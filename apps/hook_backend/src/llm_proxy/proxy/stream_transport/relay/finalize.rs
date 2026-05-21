use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};

use super::StreamRelay;
use crate::llm_proxy::{
    LlmProxyError,
    proxy::{
        response_model::rewrite_response_model_value,
        stream_transport::{
            estimated_usage::{ESTIMATED_REQUEST_USAGE_SOURCE, ESTIMATED_USAGE_SOURCE},
            event::render_stream_event,
            record::{StreamFailureRecordInput, StreamSuccessRecordInput, failure_record, record_stream_attempt, success_record},
            status::StreamEndReason,
        },
    },
};

impl StreamRelay {
    pub(super) async fn finish_success(&mut self) -> Result<(), LlmProxyError> {
        if self.finished {
            return Ok(());
        }
        if let Err(error) = self.finish_usage_parsers() {
            self.record_scanner_error(&error).await?;
            return Err(error);
        }
        if let Err(error) = self.flush_output().await {
            self.record_handler_stop_error(&error).await?;
            return Err(error);
        }
        self.stream_status.set_end_reason(self.finish_stream_end_reason(), None);
        self.record_success().await?;
        self.finished = true;
        Ok(())
    }

    pub(super) async fn handle_protocol_completion(&mut self) -> Result<(), LlmProxyError> {
        self.protocol_completed = true;
        if self.yielded_any || self.pending.is_empty() {
            self.finish_after_protocol_completion().await?;
        }
        Ok(())
    }

    pub(super) async fn finish_after_protocol_completion(&mut self) -> Result<(), LlmProxyError> {
        if self.finished {
            return Ok(());
        }
        if let Err(error) = self.estimate_missing_usage() {
            self.record_scanner_error(&error).await?;
            return Err(error);
        }
        if let Err(error) = self.flush_output().await {
            self.record_handler_stop_error(&error).await?;
            return Err(error);
        }
        self.stream_status.set_end_reason(StreamEndReason::Done, None);
        self.record_success().await?;
        self.finished = true;
        Ok(())
    }

    pub(super) async fn record_success(&mut self) -> Result<(), LlmProxyError> {
        if self.recorded_terminal {
            return Ok(());
        }
        let record = success_record(StreamSuccessRecordInput {
            context: &self.context,
            usage: self.usage,
            first_byte_time_ms: self.first_byte_time_ms,
            status: &self.stream_status,
            bodies: self.terminal_response_bodies(),
        });
        record_stream_attempt(record).await?;
        self.recorded_terminal = true;
        self.log_stream_status();
        Ok(())
    }

    pub(super) async fn record_failure(&mut self, error_type: &'static str, error_message: &str) -> Result<(), LlmProxyError> {
        if self.recorded_terminal {
            return Ok(());
        }
        record_stream_attempt(failure_record(StreamFailureRecordInput {
            context: &self.context,
            first_byte_time_ms: self.first_byte_time_ms,
            error_type,
            error_message,
            status: &self.stream_status,
            bodies: self.terminal_response_bodies(),
        }))
        .await?;
        self.recorded_terminal = true;
        self.log_stream_status();
        Ok(())
    }

    async fn flush_output(&mut self) -> Result<(), LlmProxyError> {
        if self.needs_conversion || self.rewrite_model {
            self.flush_remaining_buffer().await?;
            self.flush_conversion_state()?;
            self.queue_openai_done();
        }
        Ok(())
    }

    async fn flush_remaining_buffer(&mut self) -> Result<(), LlmProxyError> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        let line = String::from_utf8(std::mem::take(&mut self.buffer)).map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
        self.consume_converted_line(&line)?;
        Ok(())
    }

    fn flush_conversion_state(&mut self) -> Result<(), LlmProxyError> {
        if !self.needs_conversion {
            return Ok(());
        }
        let converted = FormatConversionRegistry::default()
            .flush_stream(self.target_format, self.source_format, &mut self.conversion)
            .map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
        for mut event in converted {
            rewrite_response_model_value(&mut event, &self.context.candidate.requested_model_name);
            self.pending.push_back(render_stream_event(&event, self.source_format));
        }
        Ok(())
    }

    fn queue_openai_done(&mut self) {
        if matches!(self.source_format, ApiFormat::OpenAiChat) && !self.openai_done_sent {
            self.pending.push_back(req::Bytes::from_static(b"data: [DONE]\n\n"));
            self.openai_done_sent = true;
        }
    }

    fn finish_usage_parsers(&mut self) -> Result<(), LlmProxyError> {
        let usage = self.usage_parser.finish()?;
        self.merge_usage(usage.usage);
        self.protocol_completed |= usage.completed;
        self.estimate_missing_usage()
    }

    fn estimate_missing_usage(&mut self) -> Result<(), LlmProxyError> {
        self.usage_estimator.finish()?;
        self.usage = self.usage_estimator.apply_to_usage(self.usage, self.protocol_completed);
        Ok(())
    }

    fn finish_stream_end_reason(&self) -> StreamEndReason {
        if self.protocol_completed {
            return StreamEndReason::Done;
        }
        if matches!(
            self.usage.and_then(|usage| usage.usage_source),
            Some(ESTIMATED_USAGE_SOURCE | ESTIMATED_REQUEST_USAGE_SOURCE)
        ) && matches!(
            self.target_format,
            ApiFormat::OpenAiChat | ApiFormat::OpenAiResponses | ApiFormat::ClaudeChat | ApiFormat::GeminiChat
        ) {
            return StreamEndReason::UpstreamEofWithoutCompletion;
        }
        StreamEndReason::Eof
    }

    async fn record_handler_stop_error(&mut self, error: &LlmProxyError) -> Result<(), LlmProxyError> {
        let message = error.to_string();
        self.stream_status.record_error(message.clone());
        self.stream_status.set_end_reason(StreamEndReason::HandlerStop, Some(message.clone()));
        self.record_failure("response_conversion_error", &message).await
    }

    async fn record_scanner_error(&mut self, error: &LlmProxyError) -> Result<(), LlmProxyError> {
        let message = error.to_string();
        self.stream_status.record_error(message.clone());
        self.stream_status.set_end_reason(StreamEndReason::ScannerError, Some(message.clone()));
        self.record_failure("upstream_stream_parse_error", &message).await
    }

    fn log_stream_status(&self) {
        let summary = self.stream_status.summary();
        if self.stream_status.is_normal_end() && !self.stream_status.has_errors() {
            hook_tracing::info_with_fields!("stream ended", summary = summary);
            return;
        }
        let soft_errors = self.stream_status.total_error_count();
        let received = self.stream_status.received_response_count();
        hook_tracing::warn_with_fields!("stream ended abnormally", summary = summary, soft_errors = soft_errors, received = received);
    }
}

#[cfg(test)]
mod tests {
    use proxy::format_conversion::ApiFormat;

    use crate::llm_proxy::proxy::stream_transport::estimated_usage::StreamUsageEstimator;

    #[test]
    fn openai_responses_can_estimate_usage_before_completed_eof() {
        let request = serde_json::json!({"input":[{"role":"user","content":"hello world"}]});
        let mut estimator = StreamUsageEstimator::new(ApiFormat::OpenAiResponses, &request, "gpt-5.5");

        estimator
            .consume(br#"data: {"type":"response.output_text.delta","delta":"hello there"}"#)
            .unwrap();
        estimator.finish().unwrap();

        let usage = estimator.estimated_usage().expect("usage should be estimated");

        assert!(usage.prompt_tokens.expect("prompt tokens") > 0);
        assert!(usage.completion_tokens.expect("completion tokens") > 0);
        assert_eq!(usage.usage_source, Some("estimated_from_stream_delta"));
        assert_eq!(usage.usage_semantic, Some("responses"));
    }

    #[test]
    fn openai_chat_can_estimate_usage_before_usage_chunk_eof() {
        let request = serde_json::json!({"messages":[{"role":"user","content":"hello"}]});
        let mut estimator = StreamUsageEstimator::new(ApiFormat::OpenAiChat, &request, "gpt-5.5");

        estimator
            .consume(br#"data: {"choices":[{"delta":{"content":"world"},"finish_reason":null}]}"#)
            .unwrap();
        estimator.finish().unwrap();

        let usage = estimator.estimated_usage().expect("usage should be estimated");

        assert!(usage.prompt_tokens.expect("prompt tokens") > 0);
        assert!(usage.completion_tokens.expect("completion tokens") > 0);
        assert_eq!(usage.usage_source, Some("estimated_from_stream_delta"));
        assert_eq!(usage.usage_semantic, Some("openai"));
    }
}

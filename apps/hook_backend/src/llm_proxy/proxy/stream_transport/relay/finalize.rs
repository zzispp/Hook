use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use types::model::PatchField;

use super::StreamRelay;
use crate::llm_proxy::{
    LlmProxyError,
    proxy::{
        response_model::rewrite_response_model_value,
        stream_transport::{
            estimated_usage::ESTIMATED_USAGE_SOURCE,
            event::render_stream_event,
            record::{failure_record, record_stream_attempt, success_record},
        },
    },
};

const EOF_WITH_ESTIMATED_USAGE_END_REASON: &str = "upstream_eof_without_completion";

impl StreamRelay {
    pub(super) async fn finish_success(&mut self) -> Result<(), LlmProxyError> {
        if self.finished {
            return Ok(());
        }
        let usage = self.usage_parser.finish()?;
        self.merge_usage(usage.usage);
        self.protocol_completed |= usage.completed;
        self.estimate_missing_usage()?;
        self.flush_output().await?;
        self.record_success_with_stream_end_reason(self.finish_stream_end_reason()).await?;
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
        self.estimate_missing_usage()?;
        self.flush_output().await?;
        self.record_success().await?;
        self.finished = true;
        Ok(())
    }

    pub(super) async fn record_success(&mut self) -> Result<(), LlmProxyError> {
        self.record_success_with_stream_end_reason(PatchField::Null).await
    }

    async fn record_success_with_stream_end_reason(&mut self, stream_end_reason: PatchField<String>) -> Result<(), LlmProxyError> {
        if self.recorded_terminal {
            return Ok(());
        }
        let mut record = success_record(&self.context, self.usage, self.first_byte_time_ms);
        record.stream_end_reason = stream_end_reason;
        record_stream_attempt(record).await?;
        self.recorded_terminal = true;
        Ok(())
    }

    pub(super) async fn record_failure(&mut self, error_type: &'static str, error_message: &str) -> Result<(), LlmProxyError> {
        if self.recorded_terminal {
            return Ok(());
        }
        record_stream_attempt(failure_record(&self.context, self.first_byte_time_ms, error_type, error_message)).await?;
        self.recorded_terminal = true;
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
        if let Err(error) = self.consume_converted_line(&line) {
            let message = error.to_string();
            self.record_failure("response_conversion_error", &message).await?;
            return Err(error);
        }
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

    fn estimate_missing_usage(&mut self) -> Result<(), LlmProxyError> {
        self.usage_estimator.finish()?;
        self.usage = self.usage_estimator.apply_to_usage(self.usage, self.protocol_completed);
        Ok(())
    }

    fn finish_stream_end_reason(&self) -> PatchField<String> {
        if self.protocol_completed {
            return PatchField::Null;
        }
        if matches!(self.usage.and_then(|usage| usage.usage_source), Some(ESTIMATED_USAGE_SOURCE))
            && matches!(
                self.target_format,
                ApiFormat::OpenAiChat | ApiFormat::OpenAiResponses | ApiFormat::ClaudeChat | ApiFormat::GeminiChat
            )
        {
            return PatchField::Value(EOF_WITH_ESTIMATED_USAGE_END_REASON.into());
        }
        PatchField::Null
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

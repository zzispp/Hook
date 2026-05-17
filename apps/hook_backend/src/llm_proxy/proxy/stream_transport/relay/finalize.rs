use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};

use super::StreamRelay;
use crate::llm_proxy::{
    LlmProxyError,
    proxy::{
        response_model::rewrite_response_model_value,
        stream_transport::{
            event::render_stream_event,
            record::{failure_record, record_stream_attempt, success_record},
        },
    },
};

impl StreamRelay {
    pub(super) async fn finish_success(&mut self) -> Result<(), LlmProxyError> {
        if self.finished {
            return Ok(());
        }
        self.flush_output().await?;
        let usage = self.usage_parser.finish()?;
        self.merge_usage(usage.usage);
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
        self.flush_output().await?;
        self.record_success().await?;
        self.finished = true;
        Ok(())
    }

    pub(super) async fn record_success(&mut self) -> Result<(), LlmProxyError> {
        if self.recorded_terminal {
            return Ok(());
        }
        record_stream_attempt(success_record(&self.context, self.usage, self.first_byte_time_ms)).await?;
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
}

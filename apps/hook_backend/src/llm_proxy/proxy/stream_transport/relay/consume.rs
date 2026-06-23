use proxy::format_conversion::{ApiFormat, FormatConversionRegistry, StreamChunkConversion};
use serde_json::Value;

use super::StreamRelay;
use crate::llm_proxy::{
    LlmProxyError,
    audit::TokenUsage,
    codex_chat_history::CodexChatHistoryStore,
    proxy::{
        response_model::rewrite_response_model_value,
        stream_transport::{
            event::{render_stream_error, render_stream_event},
            status::StreamEndReason,
        },
        usage,
    },
};

impl StreamRelay {
    pub(super) async fn consume_bytes(&mut self, bytes: req::Bytes, fail_before_output: bool) -> Result<(), LlmProxyError> {
        let parsed = match self.usage_parser.consume(&bytes) {
            Ok(parsed) => parsed,
            Err(error) => return self.handle_scanner_error(error).await,
        };
        if let Err(error) = self.usage_estimator.consume(&bytes) {
            return self.handle_scanner_error(error).await;
        }
        if let Err(error) = self.detect_client_output_start(&bytes) {
            return self.handle_scanner_error(error).await;
        }
        self.merge_usage(parsed.usage);
        if !self.needs_conversion && !self.rewrite_model {
            self.pending.push_back(bytes);
            if parsed.completed {
                self.handle_protocol_completion().await?;
            }
            return Ok(());
        }
        self.buffer.extend_from_slice(&bytes);
        while let Some(line) = match self.next_line() {
            Ok(line) => line,
            Err(error) => return self.handle_conversion_error(error, fail_before_output).await,
        } {
            if let Err(error) = self.consume_converted_line(&line).await {
                return self.handle_conversion_error(error, fail_before_output).await;
            }
        }
        if parsed.completed {
            self.handle_protocol_completion().await?;
        }
        Ok(())
    }

    fn detect_client_output_start(&mut self, bytes: &[u8]) -> Result<(), LlmProxyError> {
        if self.client_output_started {
            return Ok(());
        }
        let started = self
            .output_start_detector
            .consume(bytes)
            .map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
        if started {
            self.first_output_time_ms = Some(self.context.started.elapsed().as_millis().try_into().unwrap_or(i64::MAX));
        }
        self.client_output_started = started;
        Ok(())
    }

    pub(super) async fn consume_converted_line(&mut self, line: &str) -> Result<(), LlmProxyError> {
        let Some(payload) = line.trim_end_matches(['\r', '\n']).strip_prefix("data:") else {
            return Ok(());
        };
        let payload = payload.trim();
        if payload.is_empty() {
            return Ok(());
        }
        if payload == "[DONE]" {
            self.queue_done();
            return Ok(());
        }
        let mut chunk = serde_json::from_str::<Value>(payload).map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
        self.stream_status.record_response();
        self.merge_usage(usage::from_stream_chunk(&chunk, self.target_format));
        if !self.needs_conversion {
            rewrite_response_model_value(&mut chunk, &self.context.candidate.requested_model_name);
            record_client_event_for_history(self.context.state.codex_chat_history().clone(), self.source_format, &chunk).await?;
            self.pending.push_back(render_stream_event(&chunk, self.source_format));
            return Ok(());
        }
        self.push_converted_chunk(&chunk).await
    }

    pub(super) fn merge_usage(&mut self, incoming: Option<TokenUsage>) {
        if let Some(usage) = incoming {
            self.usage = usage::merge(self.usage, usage);
        }
    }

    pub(super) fn queue_done(&mut self) {
        if matches!(self.source_format, ApiFormat::OpenAiChat) && !self.openai_done_sent {
            self.pending.push_back(req::Bytes::from_static(b"data: [DONE]\n\n"));
            self.openai_done_sent = true;
        }
    }

    async fn handle_scanner_error(&mut self, error: LlmProxyError) -> Result<(), LlmProxyError> {
        let error_message = error.to_string();
        self.stream_status.record_error(error_message.clone());
        self.stream_status.set_end_reason(StreamEndReason::ScannerError, Some(error_message.clone()));
        self.record_failure("upstream_stream_parse_error", &error_message).await?;
        self.finished = true;
        Err(error)
    }

    async fn handle_conversion_error(&mut self, error: LlmProxyError, fail_before_output: bool) -> Result<(), LlmProxyError> {
        let error_message = error.to_string();
        self.stream_status.record_error(error_message.clone());
        self.stream_status.set_end_reason(StreamEndReason::HandlerStop, Some(error_message.clone()));
        if fail_before_output && !self.yielded_any && self.pending.is_empty() {
            self.record_failure("response_conversion_error", &error_message).await?;
            self.finished = true;
            return Err(error);
        }
        self.pending.push_back(render_stream_error(self.source_format));
        self.queue_done();
        self.record_failure("response_conversion_error", &error_message).await?;
        self.finished = true;
        Ok(())
    }

    fn next_line(&mut self) -> Result<Option<String>, LlmProxyError> {
        let Some(position) = self.buffer.iter().position(|byte| *byte == b'\n') else {
            return Ok(None);
        };
        let line = self.buffer.drain(..=position).collect::<Vec<_>>();
        String::from_utf8(line)
            .map(Some)
            .map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))
    }

    async fn push_converted_chunk(&mut self, chunk: &Value) -> Result<(), LlmProxyError> {
        let converted = FormatConversionRegistry
            .convert_stream_chunk(StreamChunkConversion {
                chunk,
                source: self.target_format,
                target: self.source_format,
                state: &mut self.conversion,
            })
            .map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
        for mut event in converted {
            rewrite_response_model_value(&mut event, &self.context.candidate.requested_model_name);
            record_client_event_for_history(self.context.state.codex_chat_history().clone(), self.source_format, &event).await?;
            self.pending.push_back(render_stream_event(&event, self.source_format));
        }
        Ok(())
    }
}

pub(super) async fn record_client_event_for_history(history: CodexChatHistoryStore, source_format: ApiFormat, event: &Value) -> Result<(), LlmProxyError> {
    if !matches!(source_format, ApiFormat::OpenAiResponses | ApiFormat::OpenAiResponsesCompact) {
        return Ok(());
    }
    history.record_stream_event(event).await?;
    Ok(())
}

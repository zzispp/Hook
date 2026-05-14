use std::collections::VecDeque;

use axum::body::Bytes;
use proxy::format_conversion::{ApiFormat, FormatConversionRegistry, StreamChunkConversion, StreamConversionState};
use serde_json::Value;
use types::model::PatchField;

use super::{
    StreamAttemptContext, UpstreamStream,
    event::{parse_stream_values, render_stream_error, render_stream_event, stream_error},
    record::{StreamAttemptRecord, record_stream_attempt, response_read_error_type},
};
use crate::llm_proxy::{
    LlmProxyError,
    audit::TokenUsage,
    proxy::{response_model::rewrite_response_model_value, transport, usage},
};

type DownstreamItem = Result<Bytes, std::io::Error>;

pub(super) struct StreamRelay {
    context: StreamAttemptContext,
    upstream: UpstreamStream,
    source_format: ApiFormat,
    target_format: ApiFormat,
    needs_conversion: bool,
    rewrite_model: bool,
    conversion: StreamConversionState,
    buffer: Vec<u8>,
    pending: VecDeque<Bytes>,
    usage: Option<TokenUsage>,
    first_byte_time_ms: Option<i64>,
    yielded_any: bool,
    finished: bool,
    recorded_terminal: bool,
    openai_done_sent: bool,
}

pub(super) async fn next_body_item(mut relay: StreamRelay) -> Option<(DownstreamItem, StreamRelay)> {
    let item = relay.next_item().await?;
    Some((item, relay))
}

impl StreamRelay {
    pub(super) fn new(context: StreamAttemptContext, upstream: UpstreamStream, source_format: ApiFormat, target_format: ApiFormat) -> Self {
        let needs_conversion = context.candidate.trace.needs_conversion;
        let rewrite_model = context.candidate.provider_model_name != context.candidate.requested_model_name;
        Self {
            context,
            upstream,
            source_format,
            target_format,
            needs_conversion,
            rewrite_model,
            conversion: StreamConversionState::default(),
            buffer: Vec::new(),
            pending: VecDeque::new(),
            usage: None,
            first_byte_time_ms: None,
            yielded_any: false,
            finished: false,
            recorded_terminal: false,
            openai_done_sent: false,
        }
    }

    pub(super) async fn prefetch(&mut self) -> Result<(), LlmProxyError> {
        while self.pending.is_empty() && !self.finished {
            let Some(item) = futures_util::StreamExt::next(&mut self.upstream).await else {
                self.finish_success().await?;
                break;
            };
            self.handle_upstream_item(item, true).await?;
        }
        Ok(())
    }

    async fn next_item(&mut self) -> Option<DownstreamItem> {
        loop {
            if let Some(bytes) = self.pending.pop_front() {
                return Some(self.mark_first_byte(bytes).await);
            }
            if self.finished {
                return None;
            }
            let Some(item) = futures_util::StreamExt::next(&mut self.upstream).await else {
                return self.finish_success_item().await;
            };
            if let Err(error) = self.handle_upstream_item(item, false).await {
                return Some(Err(stream_error(error.to_string())));
            }
        }
    }

    async fn handle_upstream_item(&mut self, item: Result<Bytes, reqwest::Error>, fail_before_output: bool) -> Result<(), LlmProxyError> {
        match item {
            Ok(bytes) => self.consume_bytes(bytes, fail_before_output).await,
            Err(error) => self.record_read_error(&error).await.and_then(|()| Err(error.into())),
        }
    }

    async fn consume_bytes(&mut self, bytes: Bytes, fail_before_output: bool) -> Result<(), LlmProxyError> {
        self.collect_usage_from_bytes(&bytes);
        if !self.needs_conversion && !self.rewrite_model {
            self.pending.push_back(bytes);
            return Ok(());
        }
        self.buffer.extend_from_slice(&bytes);
        while let Some(line) = self.next_line()? {
            if let Err(error) = self.consume_converted_line(&line) {
                return self.handle_conversion_error(error, fail_before_output).await;
            }
        }
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

    fn consume_converted_line(&mut self, line: &str) -> Result<(), LlmProxyError> {
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
        self.merge_usage(usage::from_stream_chunk(&chunk, self.target_format));
        if !self.needs_conversion {
            rewrite_response_model_value(&mut chunk, &self.context.candidate.requested_model_name);
            self.pending.push_back(render_stream_event(&chunk, self.source_format));
            return Ok(());
        }
        let converted = FormatConversionRegistry::default()
            .convert_stream_chunk(StreamChunkConversion {
                chunk: &chunk,
                source: self.target_format,
                target: self.source_format,
                state: &mut self.conversion,
            })
            .map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
        for mut event in converted {
            rewrite_response_model_value(&mut event, &self.context.candidate.requested_model_name);
            self.pending.push_back(render_stream_event(&event, self.source_format));
        }
        Ok(())
    }

    async fn handle_conversion_error(&mut self, error: LlmProxyError, fail_before_output: bool) -> Result<(), LlmProxyError> {
        let error_message = error.to_string();
        self.record_failure("response_conversion_error", &error_message).await?;
        self.finished = true;
        if fail_before_output && !self.yielded_any && self.pending.is_empty() {
            return Err(error);
        }
        self.pending.push_back(render_stream_error(self.source_format));
        self.queue_done();
        Ok(())
    }

    async fn mark_first_byte(&mut self, bytes: Bytes) -> DownstreamItem {
        if self.yielded_any {
            return Ok(bytes);
        }
        let first_byte = transport::elapsed_ms(self.context.started);
        let record = self.first_byte_record(first_byte);
        if let Err(error) = record_stream_attempt(record).await {
            return Err(stream_error(error.to_string()));
        }
        self.first_byte_time_ms = Some(first_byte);
        self.yielded_any = true;
        Ok(bytes)
    }

    async fn finish_success_item(&mut self) -> Option<DownstreamItem> {
        match self.finish_success().await {
            Ok(()) => self.pending.pop_front().map(Ok),
            Err(error) => Some(Err(stream_error(error.to_string()))),
        }
    }

    async fn finish_success(&mut self) -> Result<(), LlmProxyError> {
        if self.finished {
            return Ok(());
        }
        if self.needs_conversion || self.rewrite_model {
            self.flush_remaining_buffer().await?;
            if matches!(self.source_format, ApiFormat::OpenAiChat) && !self.openai_done_sent {
                self.pending.push_back(Bytes::from_static(b"data: [DONE]\n\n"));
                self.openai_done_sent = true;
            }
        }
        self.finished = true;
        self.record_success().await
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

    fn first_byte_record(&self, first_byte_elapsed: i64) -> StreamAttemptRecord {
        StreamAttemptRecord {
            state: self.context.state.clone(),
            request_id: self.context.request_id.clone(),
            candidate: self.context.candidate.clone(),
            retry_index: self.context.retry_index,
            status: "streaming",
            status_code: Some(self.context.status.as_u16() as i32),
            usage: None,
            latency_ms: None,
            first_byte_time_ms: Some(first_byte_elapsed),
            error_type: None,
            error_message: None,
            client_response_body: PatchField::Missing,
            termination_origin: PatchField::Missing,
            termination_reason: PatchField::Missing,
            stream_end_reason: PatchField::Missing,
            finished: false,
        }
    }

    async fn record_success(&mut self) -> Result<(), LlmProxyError> {
        if self.recorded_terminal {
            return Ok(());
        }
        self.recorded_terminal = true;
        record_stream_attempt(StreamAttemptRecord {
            state: self.context.state.clone(),
            request_id: self.context.request_id.clone(),
            candidate: self.context.candidate.clone(),
            retry_index: self.context.retry_index,
            status: "success",
            status_code: Some(self.context.status.as_u16() as i32),
            usage: self.usage,
            latency_ms: Some(transport::elapsed_ms(self.context.started)),
            first_byte_time_ms: self.first_byte_time_ms,
            error_type: None,
            error_message: None,
            client_response_body: PatchField::Missing,
            termination_origin: PatchField::Null,
            termination_reason: PatchField::Null,
            stream_end_reason: PatchField::Null,
            finished: true,
        })
        .await
    }

    async fn record_failure(&mut self, error_type: &'static str, error_message: &str) -> Result<(), LlmProxyError> {
        if self.recorded_terminal {
            return Ok(());
        }
        self.recorded_terminal = true;
        record_stream_attempt(StreamAttemptRecord {
            state: self.context.state.clone(),
            request_id: self.context.request_id.clone(),
            candidate: self.context.candidate.clone(),
            retry_index: self.context.retry_index,
            status: "failed",
            status_code: Some(self.context.status.as_u16() as i32),
            usage: None,
            latency_ms: Some(transport::elapsed_ms(self.context.started)),
            first_byte_time_ms: self.first_byte_time_ms,
            error_type: Some(error_type),
            error_message: Some(error_message.to_owned()),
            client_response_body: PatchField::Missing,
            termination_origin: PatchField::Null,
            termination_reason: PatchField::Null,
            stream_end_reason: PatchField::Null,
            finished: true,
        })
        .await
    }

    async fn record_read_error(&mut self, error: &reqwest::Error) -> Result<(), LlmProxyError> {
        let error_message = error.to_string();
        self.record_failure(response_read_error_type(error), &error_message).await
    }

    fn collect_usage_from_bytes(&mut self, bytes: &[u8]) {
        if self.needs_conversion {
            return;
        }
        for chunk in parse_stream_values(bytes) {
            self.merge_usage(usage::from_stream_chunk(&chunk, self.target_format));
        }
    }

    fn merge_usage(&mut self, incoming: Option<TokenUsage>) {
        if let Some(usage) = incoming {
            self.usage = usage::merge(self.usage, usage);
        }
    }

    fn queue_done(&mut self) {
        if matches!(self.source_format, ApiFormat::OpenAiChat) && !self.openai_done_sent {
            self.pending.push_back(Bytes::from_static(b"data: [DONE]\n\n"));
            self.openai_done_sent = true;
        }
    }

    fn cancelled_record(&self) -> StreamAttemptRecord {
        StreamAttemptRecord {
            state: self.context.state.clone(),
            request_id: self.context.request_id.clone(),
            candidate: self.context.candidate.clone(),
            retry_index: self.context.retry_index,
            status: "cancelled",
            status_code: Some(499),
            usage: self.usage,
            latency_ms: Some(transport::elapsed_ms(self.context.started)),
            first_byte_time_ms: self.first_byte_time_ms,
            error_type: Some("client_disconnected"),
            error_message: Some("client disconnected before stream completed".into()),
            client_response_body: PatchField::Missing,
            termination_origin: PatchField::Value("client".into()),
            termination_reason: PatchField::Value("disconnected".into()),
            stream_end_reason: PatchField::Value("client_disconnected".into()),
            finished: true,
        }
    }
}

impl Drop for StreamRelay {
    fn drop(&mut self) {
        if self.recorded_terminal || self.finished {
            return;
        }
        let record = self.cancelled_record();
        tokio::spawn(async move {
            if let Err(error) = record_stream_attempt(record).await {
                hook_tracing::warn_with_fields!("failed to record cancelled streaming request candidate", error = error);
            }
        });
    }
}

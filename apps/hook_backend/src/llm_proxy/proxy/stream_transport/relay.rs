mod finalize;

use std::{collections::VecDeque, time::Duration};

use proxy::format_conversion::{ApiFormat, FormatConversionRegistry, StreamChunkConversion, StreamConversionState};
use serde_json::Value;

use super::{
    StreamAttemptContext, UpstreamStream,
    estimated_usage::StreamUsageEstimator,
    event::{render_keepalive, render_stream_error, render_stream_event, stream_error},
    record::{cancelled_record, record_stream_attempt, response_read_error_type, streaming_record},
    usage_parser::StreamUsageParser,
};
use crate::llm_proxy::{
    LlmProxyError,
    audit::TokenUsage,
    proxy::{response_model::rewrite_response_model_value, usage},
};

type DownstreamItem = Result<req::Bytes, std::io::Error>;
const SSE_KEEPALIVE_INTERVAL_SECS: u64 = 10;

pub(super) struct StreamRelay {
    context: StreamAttemptContext,
    upstream: UpstreamStream,
    source_format: ApiFormat,
    target_format: ApiFormat,
    needs_conversion: bool,
    rewrite_model: bool,
    conversion: StreamConversionState,
    buffer: Vec<u8>,
    pending: VecDeque<req::Bytes>,
    usage_parser: StreamUsageParser,
    usage_estimator: StreamUsageEstimator,
    usage: Option<TokenUsage>,
    first_byte_time_ms: Option<i64>,
    yielded_any: bool,
    finished: bool,
    protocol_completed: bool,
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
        let usage_estimator = StreamUsageEstimator::new(target_format, &context.provider_request_body, &context.candidate.provider_model_name);
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
            usage_parser: StreamUsageParser::new(target_format),
            usage_estimator,
            usage: None,
            first_byte_time_ms: None,
            yielded_any: false,
            finished: false,
            protocol_completed: false,
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

    pub(super) async fn record_first_byte_timeout(&mut self) -> Result<(), LlmProxyError> {
        self.record_failure("upstream_timeout", "stream first byte timeout").await?;
        self.finished = true;
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
            match self.next_upstream_item().await {
                NextUpstreamItem::Chunk(item) => {
                    if let Err(error) = self.handle_upstream_item(item, false).await {
                        return Some(Err(stream_error(error.to_string())));
                    }
                }
                NextUpstreamItem::Keepalive(bytes) => return Some(Ok(bytes)),
                NextUpstreamItem::End => return self.finish_success_item().await,
            }
        }
    }

    async fn handle_upstream_item(&mut self, item: Result<req::Bytes, req::ClientError>, fail_before_output: bool) -> Result<(), LlmProxyError> {
        match item {
            Ok(bytes) => self.consume_bytes(bytes, fail_before_output).await,
            Err(error) => self.record_read_error(&error).await.and_then(|()| Err(error.into())),
        }
    }

    async fn next_upstream_item(&mut self) -> NextUpstreamItem {
        loop {
            match tokio::time::timeout(
                Duration::from_secs(SSE_KEEPALIVE_INTERVAL_SECS),
                futures_util::StreamExt::next(&mut self.upstream),
            )
            .await
            {
                Ok(Some(item)) => return NextUpstreamItem::Chunk(item),
                Ok(None) => return NextUpstreamItem::End,
                Err(_) => return NextUpstreamItem::Keepalive(render_keepalive()),
            }
        }
    }

    async fn consume_bytes(&mut self, bytes: req::Bytes, fail_before_output: bool) -> Result<(), LlmProxyError> {
        let parsed = self.usage_parser.consume(&bytes)?;
        self.usage_estimator.consume(&bytes)?;
        self.merge_usage(parsed.usage);
        if !self.needs_conversion && !self.rewrite_model {
            self.pending.push_back(bytes);
            if parsed.completed {
                self.handle_protocol_completion().await?;
            }
            return Ok(());
        }
        self.buffer.extend_from_slice(&bytes);
        while let Some(line) = self.next_line()? {
            if let Err(error) = self.consume_converted_line(&line) {
                return self.handle_conversion_error(error, fail_before_output).await;
            }
        }
        if parsed.completed {
            self.handle_protocol_completion().await?;
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

    async fn mark_first_byte(&mut self, bytes: req::Bytes) -> DownstreamItem {
        if self.yielded_any {
            return Ok(bytes);
        }
        let first_byte = self.context.started.elapsed().as_millis().try_into().unwrap_or(i64::MAX);
        let record = streaming_record(&self.context, first_byte);
        if let Err(error) = record_stream_attempt(record).await {
            return Err(stream_error(error.to_string()));
        }
        self.first_byte_time_ms = Some(first_byte);
        self.yielded_any = true;
        if self.protocol_completed && !self.finished {
            if let Err(error) = self.finish_after_protocol_completion().await {
                return Err(stream_error(error.to_string()));
            }
        }
        Ok(bytes)
    }

    async fn finish_success_item(&mut self) -> Option<DownstreamItem> {
        match self.finish_success().await {
            Ok(()) => self.pending.pop_front().map(Ok),
            Err(error) => Some(Err(stream_error(error.to_string()))),
        }
    }

    async fn record_read_error(&mut self, error: &req::ClientError) -> Result<(), LlmProxyError> {
        let error_message = error.to_string();
        self.record_failure(response_read_error_type(error), &error_message).await
    }

    fn merge_usage(&mut self, incoming: Option<TokenUsage>) {
        if let Some(usage) = incoming {
            self.usage = usage::merge(self.usage, usage);
        }
    }

    fn queue_done(&mut self) {
        if matches!(self.source_format, ApiFormat::OpenAiChat) && !self.openai_done_sent {
            self.pending.push_back(req::Bytes::from_static(b"data: [DONE]\n\n"));
            self.openai_done_sent = true;
        }
    }
}

enum NextUpstreamItem {
    Chunk(Result<req::Bytes, req::ClientError>),
    Keepalive(req::Bytes),
    End,
}

impl Drop for StreamRelay {
    fn drop(&mut self) {
        if self.recorded_terminal || self.finished {
            return;
        }
        let record = cancelled_record(&self.context, self.usage, self.first_byte_time_ms);
        tokio::spawn(async move {
            if let Err(error) = record_stream_attempt(record).await {
                hook_tracing::warn_with_fields!("failed to record cancelled streaming request candidate", error = error);
            }
        });
    }
}

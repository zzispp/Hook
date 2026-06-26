mod body;
mod consume;
mod drop_record;
mod finalize;
mod timing;

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use axum::http::{HeaderMap, HeaderValue};
use proxy::format_conversion::{ApiFormat, StreamConversionState};

use super::{
    StreamAttemptContext, StreamPreOutputFailure, UpstreamStream,
    body_capture::StreamBodyCapture,
    estimated_usage::StreamUsageEstimator,
    event::stream_error,
    output_start::StreamOutputStartDetector,
    preflight::inspect_provider_error,
    record::{record_stream_attempt, response_read_error_type},
    status::{StreamEndReason, StreamStatus},
    terminal::StreamClientFailure,
    usage_parser::StreamUsageParser,
};
use crate::llm_proxy::{LlmProxyError, audit::TokenUsage};

type DownstreamItem = Result<req::Bytes, std::io::Error>;

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
    output_start_detector: StreamOutputStartDetector,
    usage: Option<TokenUsage>,
    first_sse_event_time_ms: Option<i64>,
    first_output_time_ms: Option<i64>,
    first_byte_time_ms: Option<i64>,
    last_upstream_item_at: Instant,
    stream_idle_timeout: Option<Duration>,
    yielded_any: bool,
    client_output_started: bool,
    finished: bool,
    protocol_completed: bool,
    recorded_terminal: bool,
    openai_done_sent: bool,
    stream_status: StreamStatus,
    body_capture: StreamBodyCapture,
    client_failure: Option<StreamClientFailure>,
}

pub(super) async fn next_body_item(mut relay: StreamRelay) -> Option<(DownstreamItem, StreamRelay)> {
    let item = relay.next_item().await?;
    Some((item, relay))
}

impl StreamRelay {
    pub(super) fn new(context: StreamAttemptContext, upstream: UpstreamStream, source_format: ApiFormat, target_format: ApiFormat) -> Self {
        let needs_conversion = source_format != target_format;
        let rewrite_model = context.candidate.provider_model_name != context.candidate.requested_model_name;
        let usage_estimator = StreamUsageEstimator::new(target_format, &context.provider_request_body, &context.candidate.provider_model_name);
        let stream_idle_timeout = super::super::timeout::proxy_timeouts(&context.candidate).stream_idle;
        let last_upstream_item_at = context.started;
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
            output_start_detector: StreamOutputStartDetector::new(target_format),
            usage: None,
            first_sse_event_time_ms: None,
            first_output_time_ms: None,
            first_byte_time_ms: None,
            last_upstream_item_at,
            stream_idle_timeout,
            yielded_any: false,
            client_output_started: false,
            finished: false,
            protocol_completed: false,
            recorded_terminal: false,
            openai_done_sent: false,
            stream_status: StreamStatus::default(),
            body_capture: StreamBodyCapture::default(),
            client_failure: None,
        }
    }

    pub(super) async fn prefetch(&mut self) -> Result<(), LlmProxyError> {
        while !self.ready_to_commit() && !self.finished {
            let Some(item) = futures_util::StreamExt::next(&mut self.upstream).await else {
                self.finish_success().await?;
                break;
            };
            self.handle_upstream_item(item, true).await?;
        }
        Ok(())
    }

    pub(super) fn pre_output_failure(&self) -> Result<Option<StreamPreOutputFailure>, LlmProxyError> {
        if self.finished && self.client_failure.is_some() && !self.client_output_started {
            return self.pre_output_failure_input().map(Some);
        }
        Ok(None)
    }

    pub(super) async fn record_first_byte_timeout(&mut self) -> Result<(), LlmProxyError> {
        self.stream_status
            .set_end_reason(StreamEndReason::Timeout, Some("stream first byte timeout".into()));
        self.record_failure("first_byte_timeout", "stream first byte timeout").await?;
        self.finished = true;
        Ok(())
    }

    pub(super) async fn record_streaming_started(&mut self, upstream_headers: HeaderMap, content_type: Option<&HeaderValue>) -> Result<(), LlmProxyError> {
        if !super::should_record_streaming_started_after_prefetch(self.finished, self.recorded_terminal) {
            return Ok(());
        }
        super::record_stream_headers(
            &self.context,
            upstream_headers,
            content_type,
            self.context.response_headers_time_ms,
            self.first_sse_event_time_ms,
            self.first_output_time_ms,
            self.compat_first_byte_time_ms(),
        )
        .await
    }

    pub(super) fn prefetch_failure_response(&self) -> Result<Option<axum::response::Response>, LlmProxyError> {
        if self.finished && self.client_failure.is_some() && !self.yielded_any {
            return self.failure_response().map(Some);
        }
        Ok(None)
    }

    pub(super) fn failure_response(&self) -> Result<axum::response::Response, LlmProxyError> {
        let Some(failure) = &self.client_failure else {
            return Err(LlmProxyError::Upstream("stream preflight failed without terminal client response".into()));
        };
        super::super::transport::response_builder(failure.status, Some(crate::llm_proxy::client_error::json_content_type()))
            .body(axum::body::Body::from(
                failure.body().map_err(|error| LlmProxyError::Infrastructure(error.to_string()))?,
            ))
            .map_err(super::super::transport::response_error)
    }

    fn pre_output_failure_input(&self) -> Result<StreamPreOutputFailure, LlmProxyError> {
        let Some(failure) = &self.client_failure else {
            return Err(LlmProxyError::Upstream("stream pre-output failure missing terminal client failure".into()));
        };
        Ok(StreamPreOutputFailure {
            status: failure.status,
            error_type: failure.error_type,
            message: failure.message.clone(),
            advance_candidate: failure.error_type == "first_byte_timeout",
        })
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
                NextUpstreamItem::Keepalive(bytes) => return Some(self.client_item(bytes)),
                NextUpstreamItem::IdleTimeout => return Some(self.record_idle_timeout().await),
                NextUpstreamItem::End => return self.finish_success_item().await,
            }
        }
    }

    async fn handle_upstream_item(&mut self, item: Result<req::Bytes, req::ClientError>, fail_before_output: bool) -> Result<(), LlmProxyError> {
        match item {
            Ok(bytes) => {
                self.last_upstream_item_at = Instant::now();
                self.record_first_byte();
                self.record_first_sse_event();
                self.record_provider_body(&bytes);
                if fail_before_output && let Some(error) = inspect_provider_error(&bytes) {
                    self.stream_status.set_end_reason(StreamEndReason::ScannerError, Some(error.message.clone()));
                    self.record_failure(error.error_type, &error.message).await?;
                    self.finished = true;
                    return Err(LlmProxyError::Upstream(error.message));
                }
                self.consume_bytes(bytes, fail_before_output).await
            }
            Err(error) => {
                self.record_read_error(&error).await?;
                self.finished = true;
                Err(error.into())
            }
        }
    }

    async fn record_idle_timeout(&mut self) -> DownstreamItem {
        let message = "stream idle timeout";
        self.stream_status.record_error(message);
        self.stream_status.set_end_reason(StreamEndReason::Timeout, Some(message.into()));
        if let Err(error) = self.record_failure("stream_idle_timeout", message).await {
            return Err(stream_error(error.to_string()));
        }
        self.finished = true;
        Err(stream_error(message.into()))
    }

    async fn mark_first_byte(&mut self, bytes: req::Bytes) -> DownstreamItem {
        if self.yielded_any {
            return self.client_item(bytes);
        }
        self.yielded_any = true;
        self.body_capture.record_client_sent(&bytes);
        if self.protocol_completed
            && !self.finished
            && let Err(error) = self.finish_after_protocol_completion().await
        {
            return Err(stream_error(error.to_string()));
        }
        Ok(bytes)
    }

    async fn finish_success_item(&mut self) -> Option<DownstreamItem> {
        match self.finish_success().await {
            Ok(()) => self.pop_client_item(),
            Err(error) => Some(Err(stream_error(error.to_string()))),
        }
    }

    async fn record_read_error(&mut self, error: &req::ClientError) -> Result<(), LlmProxyError> {
        let error_message = error.to_string();
        let reason = if matches!(error, req::ClientError::Timeout) {
            StreamEndReason::Timeout
        } else {
            StreamEndReason::ScannerError
        };
        self.stream_status.record_error(error_message.clone());
        self.stream_status.set_end_reason(reason, Some(error_message.clone()));
        self.record_failure(response_read_error_type(error), &error_message).await
    }

    fn ready_to_commit(&self) -> bool {
        self.client_output_started && !self.pending.is_empty()
    }
}

pub(super) enum NextUpstreamItem {
    Chunk(Result<req::Bytes, req::ClientError>),
    Keepalive(req::Bytes),
    IdleTimeout,
    End,
}

pub(super) struct UpstreamWaitTimeout {
    wait: Duration,
    idle_deadline: bool,
}

impl Drop for StreamRelay {
    fn drop(&mut self) {
        if self.recorded_terminal || self.finished {
            return;
        }
        let record = drop_record::drop_terminal_record(self);
        tokio::spawn(async move {
            if let Err(error) = record_stream_attempt(record).await {
                hook_tracing::warn_with_fields!("failed to record dropped streaming request candidate", error = error);
            }
        });
    }
}

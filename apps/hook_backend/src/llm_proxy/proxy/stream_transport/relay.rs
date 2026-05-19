mod consume;
mod finalize;

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use proxy::format_conversion::{ApiFormat, StreamConversionState};

use super::{
    StreamAttemptContext, UpstreamStream,
    estimated_usage::StreamUsageEstimator,
    event::{render_keepalive, stream_error},
    record::{cancelled_record, record_stream_attempt, response_read_error_type, streaming_record},
    status::{StreamEndReason, StreamStatus},
    usage_parser::StreamUsageParser,
};
use crate::llm_proxy::{LlmProxyError, audit::TokenUsage};

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
    last_upstream_item_at: Instant,
    stream_idle_timeout: Option<Duration>,
    yielded_any: bool,
    finished: bool,
    protocol_completed: bool,
    recorded_terminal: bool,
    openai_done_sent: bool,
    stream_status: StreamStatus,
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
            usage: None,
            first_byte_time_ms: None,
            last_upstream_item_at,
            stream_idle_timeout,
            yielded_any: false,
            finished: false,
            protocol_completed: false,
            recorded_terminal: false,
            openai_done_sent: false,
            stream_status: StreamStatus::default(),
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
        self.stream_status
            .set_end_reason(StreamEndReason::Timeout, Some("stream first byte timeout".into()));
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
                NextUpstreamItem::IdleTimeout => return Some(self.record_idle_timeout().await),
                NextUpstreamItem::End => return self.finish_success_item().await,
            }
        }
    }

    async fn handle_upstream_item(&mut self, item: Result<req::Bytes, req::ClientError>, fail_before_output: bool) -> Result<(), LlmProxyError> {
        match item {
            Ok(bytes) => {
                self.last_upstream_item_at = Instant::now();
                self.consume_bytes(bytes, fail_before_output).await
            }
            Err(error) => self.record_read_error(&error).await.and_then(|()| Err(error.into())),
        }
    }

    async fn next_upstream_item(&mut self) -> NextUpstreamItem {
        let timeout = next_upstream_wait_timeout(self.last_upstream_item_at, self.stream_idle_timeout);
        match tokio::time::timeout(timeout.wait, futures_util::StreamExt::next(&mut self.upstream)).await {
            Ok(Some(item)) => NextUpstreamItem::Chunk(item),
            Ok(None) => NextUpstreamItem::End,
            Err(_) if timeout.idle_deadline => NextUpstreamItem::IdleTimeout,
            Err(_) => NextUpstreamItem::Keepalive(render_keepalive()),
        }
    }

    async fn record_idle_timeout(&mut self) -> DownstreamItem {
        let message = "stream idle timeout";
        self.stream_status.record_error(message);
        self.stream_status.set_end_reason(StreamEndReason::Timeout, Some(message.into()));
        if let Err(error) = self.record_failure("upstream_timeout", message).await {
            return Err(stream_error(error.to_string()));
        }
        self.finished = true;
        Err(stream_error(message.into()))
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
            Ok(()) => self.pending.pop_front().map(Ok),
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
}

enum NextUpstreamItem {
    Chunk(Result<req::Bytes, req::ClientError>),
    Keepalive(req::Bytes),
    IdleTimeout,
    End,
}

struct UpstreamWaitTimeout {
    wait: Duration,
    idle_deadline: bool,
}

fn next_upstream_wait_timeout(last_upstream_item_at: Instant, idle_timeout: Option<Duration>) -> UpstreamWaitTimeout {
    let keepalive = Duration::from_secs(SSE_KEEPALIVE_INTERVAL_SECS);
    let Some(idle_timeout) = idle_timeout else {
        return UpstreamWaitTimeout {
            wait: keepalive,
            idle_deadline: false,
        };
    };
    let elapsed = last_upstream_item_at.elapsed();
    if elapsed >= idle_timeout {
        return UpstreamWaitTimeout {
            wait: Duration::ZERO,
            idle_deadline: true,
        };
    }
    let remaining = idle_timeout - elapsed;
    UpstreamWaitTimeout {
        wait: remaining.min(keepalive),
        idle_deadline: remaining <= keepalive,
    }
}

impl Drop for StreamRelay {
    fn drop(&mut self) {
        if self.recorded_terminal || self.finished {
            return;
        }
        self.stream_status
            .set_end_reason(StreamEndReason::ClientGone, Some("client disconnected before stream completed".into()));
        let record = cancelled_record(&self.context, self.usage, self.first_byte_time_ms, &self.stream_status);
        tokio::spawn(async move {
            if let Err(error) = record_stream_attempt(record).await {
                hook_tracing::warn_with_fields!("failed to record cancelled streaming request candidate", error = error);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{SSE_KEEPALIVE_INTERVAL_SECS, next_upstream_wait_timeout};
    use std::time::{Duration, Instant};

    #[test]
    fn upstream_wait_uses_keepalive_when_idle_timeout_is_missing() {
        let timeout = next_upstream_wait_timeout(Instant::now(), None);

        assert_eq!(timeout.wait, Duration::from_secs(SSE_KEEPALIVE_INTERVAL_SECS));
        assert!(!timeout.idle_deadline);
    }

    #[test]
    fn upstream_wait_marks_idle_deadline_before_next_keepalive() {
        let timeout = next_upstream_wait_timeout(Instant::now() - Duration::from_secs(25), Some(Duration::from_secs(30)));

        assert!(timeout.wait <= Duration::from_secs(5));
        assert!(timeout.idle_deadline);
    }

    #[test]
    fn upstream_wait_keeps_ping_before_later_idle_deadline() {
        let timeout = next_upstream_wait_timeout(Instant::now(), Some(Duration::from_secs(30)));

        assert_eq!(timeout.wait, Duration::from_secs(SSE_KEEPALIVE_INTERVAL_SECS));
        assert!(!timeout.idle_deadline);
    }
}

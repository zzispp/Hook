use std::time::{Duration, Instant};

use super::super::event::render_keepalive;
use super::{NextUpstreamItem, StreamRelay, UpstreamWaitTimeout};

const SSE_KEEPALIVE_INTERVAL_SECS: u64 = 10;

pub(super) fn next_upstream_wait_timeout(last_upstream_item_at: Instant, idle_timeout: Option<Duration>) -> UpstreamWaitTimeout {
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

impl StreamRelay {
    pub(super) async fn next_upstream_item(&mut self) -> NextUpstreamItem {
        let timeout = next_upstream_wait_timeout(self.last_upstream_item_at, self.stream_idle_timeout);
        match tokio::time::timeout(timeout.wait, futures_util::StreamExt::next(&mut self.upstream)).await {
            Ok(Some(item)) => NextUpstreamItem::Chunk(item),
            Ok(None) => NextUpstreamItem::End,
            Err(_) if timeout.idle_deadline => NextUpstreamItem::IdleTimeout,
            Err(_) => NextUpstreamItem::Keepalive(render_keepalive()),
        }
    }

    pub(super) fn record_first_sse_event(&mut self, bytes: &[u8]) {
        if self.first_sse_event_time_ms.is_some() {
            return;
        }
        if !self.first_sse_event_detector.consume(bytes) {
            return;
        }
        self.first_sse_event_time_ms = Some(self.context.started.elapsed().as_millis().try_into().unwrap_or(i64::MAX));
    }

    pub(super) fn record_first_byte(&mut self) {
        if self.first_byte_time_ms.is_some() {
            return;
        }
        self.first_byte_time_ms = Some(self.context.started.elapsed().as_millis().try_into().unwrap_or(i64::MAX));
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{SSE_KEEPALIVE_INTERVAL_SECS, next_upstream_wait_timeout};

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

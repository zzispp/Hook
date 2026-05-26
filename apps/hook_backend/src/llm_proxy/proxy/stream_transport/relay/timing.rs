use std::time::{Duration, Instant};

use super::UpstreamWaitTimeout;

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

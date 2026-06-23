use std::time::{Duration, Instant};

use crate::llm_proxy::candidate::ProxyCandidate;

const STREAM_CANDIDATE_WATCHDOG_SLACK: Duration = Duration::from_secs(1);
const STREAM_HEDGE_DELAY_RATIO: f64 = 0.25;
const STREAM_HEDGE_DELAY_MIN: Duration = Duration::from_millis(250);
const STREAM_HEDGE_DELAY_MAX: Duration = Duration::from_millis(1_500);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ProxyTimeouts {
    pub(super) request: Option<Duration>,
    pub(super) stream_first_byte: Option<Duration>,
    pub(super) stream_idle: Option<Duration>,
}

pub(super) fn proxy_timeouts(candidate: &ProxyCandidate) -> ProxyTimeouts {
    ProxyTimeouts {
        request: candidate.request_timeout_seconds.and_then(timeout_duration),
        stream_first_byte: candidate.stream_first_byte_timeout_seconds.and_then(timeout_duration),
        stream_idle: candidate.stream_idle_timeout_seconds.and_then(timeout_duration),
    }
}

pub(super) fn non_stream_total_timeout(candidate: &ProxyCandidate, is_stream: bool) -> Option<Duration> {
    if is_stream {
        return None;
    }
    Some(proxy_timeouts(candidate).request.unwrap_or_else(req::default_timeout))
}

pub(super) fn response_start_timeout(candidate: &ProxyCandidate, is_stream: bool) -> Option<Duration> {
    if is_stream {
        return proxy_timeouts(candidate).stream_first_byte;
    }
    non_stream_total_timeout(candidate, false)
}

pub(super) fn remaining_stream_first_byte_timeout(started: Instant, candidate: &ProxyCandidate) -> Option<Duration> {
    remaining_stream_first_byte_timeout_after(started.elapsed(), candidate)
}

pub(super) fn remaining_stream_first_byte_timeout_after(elapsed: Duration, candidate: &ProxyCandidate) -> Option<Duration> {
    proxy_timeouts(candidate)
        .stream_first_byte
        .map(|timeout| remaining_timeout_after(elapsed, timeout))
}

pub(super) fn remaining_timeout(started: Instant, total_timeout: Duration) -> Duration {
    remaining_timeout_after(started.elapsed(), total_timeout)
}

pub(super) fn remaining_timeout_after(elapsed: Duration, total_timeout: Duration) -> Duration {
    total_timeout.saturating_sub(elapsed)
}

pub(super) fn stream_candidate_watchdog_timeout(candidate: &ProxyCandidate) -> Option<Duration> {
    proxy_timeouts(candidate)
        .stream_first_byte
        .map(|timeout| timeout.saturating_add(STREAM_CANDIDATE_WATCHDOG_SLACK))
}

pub(super) fn stream_hedge_delay(candidate: &ProxyCandidate) -> Duration {
    let budget = proxy_timeouts(candidate)
        .stream_first_byte
        .unwrap_or(STREAM_HEDGE_DELAY_MAX);
    let scaled = Duration::from_secs_f64((budget.as_secs_f64() * STREAM_HEDGE_DELAY_RATIO).max(0.0));
    scaled.max(STREAM_HEDGE_DELAY_MIN).min(STREAM_HEDGE_DELAY_MAX)
}

pub(super) fn timeout_duration(seconds: f64) -> Option<Duration> {
    (seconds.is_finite() && seconds > 0.0).then(|| Duration::from_secs_f64(seconds))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rust_decimal::Decimal;
    use types::model::TieredPricingConfig;

    use super::{
        non_stream_total_timeout, proxy_timeouts, remaining_stream_first_byte_timeout_after, remaining_timeout_after, response_start_timeout,
        stream_candidate_watchdog_timeout, stream_hedge_delay,
    };
    use crate::llm_proxy::candidate::{CandidateRoute, CandidateTrace, ProxyCandidate};

    #[test]
    fn stream_keeps_request_timeout_separate_from_first_byte_timeout() {
        let candidate = candidate();

        let timeouts = proxy_timeouts(&candidate);

        assert_eq!(timeouts.request, Some(Duration::from_secs(300)));
        assert_eq!(timeouts.stream_first_byte, Some(Duration::from_secs(30)));
        assert_eq!(timeouts.stream_idle, Some(Duration::from_secs(30)));
    }

    #[test]
    fn non_stream_uses_provider_request_timeout_as_total_timeout() {
        let candidate = candidate();

        let timeout = non_stream_total_timeout(&candidate, false);

        assert_eq!(timeout, Some(Duration::from_secs(300)));
    }

    #[test]
    fn non_stream_uses_default_total_timeout_when_provider_timeout_is_missing() {
        let mut candidate = candidate();
        candidate.request_timeout_seconds = None;

        let timeout = non_stream_total_timeout(&candidate, false);

        assert_eq!(timeout, Some(req::default_timeout()));
    }

    #[test]
    fn stream_has_no_total_request_timeout() {
        let candidate = candidate();

        let timeout = non_stream_total_timeout(&candidate, true);

        assert_eq!(timeout, None);
    }

    #[test]
    fn stream_uses_first_byte_timeout_while_waiting_for_response_start() {
        let candidate = candidate();

        let timeout = response_start_timeout(&candidate, true);

        assert_eq!(timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn non_stream_uses_request_timeout_while_waiting_for_response_start() {
        let candidate = candidate();

        let timeout = response_start_timeout(&candidate, false);

        assert_eq!(timeout, Some(Duration::from_secs(300)));
    }

    #[test]
    fn stream_prefetch_timeout_uses_remaining_first_byte_budget() {
        let candidate = candidate();

        let timeout = remaining_stream_first_byte_timeout_after(Duration::from_secs(2), &candidate);

        assert_eq!(timeout, Some(Duration::from_secs(28)));
    }

    #[test]
    fn stream_prefetch_timeout_saturates_when_budget_is_spent() {
        let candidate = candidate();

        let timeout = remaining_stream_first_byte_timeout_after(Duration::from_secs(31), &candidate);

        assert_eq!(timeout, Some(Duration::ZERO));
    }

    #[test]
    fn stream_candidate_watchdog_adds_handoff_slack_to_first_byte_budget() {
        let candidate = candidate();

        let timeout = stream_candidate_watchdog_timeout(&candidate);

        assert_eq!(timeout, Some(Duration::from_secs(31)));
    }

    #[test]
    fn stream_hedge_delay_uses_bounded_fraction_of_first_byte_budget() {
        let candidate = candidate();

        let timeout = stream_hedge_delay(&candidate);

        assert_eq!(timeout, Duration::from_millis(1_500));
    }

    #[test]
    fn remaining_timeout_never_underflows() {
        let timeout = remaining_timeout_after(Duration::from_secs(2), Duration::from_secs(1));

        assert_eq!(timeout, Duration::ZERO);
    }

    fn candidate() -> ProxyCandidate {
        ProxyCandidate {
            trace: trace(),
            requested_model_name: "gpt-5.5".into(),
            api_key: "secret".into(),
            base_url: "https://example.com".into(),
            custom_path: None,
            upstream_url: "https://example.com/v1/responses".into(),
            provider_model_name: "gpt-5.5".into(),
            reasoning_effort: None,
            header_rules: None,
            body_rules: None,
            format_acceptance_config: None,
            key_supports_image_generation: false,
            price_per_request: None,
            tiered_pricing: TieredPricingConfig { tiers: Vec::new() },
            billing_multiplier: Decimal::ONE,
            max_retries: 0,
            request_timeout_seconds: Some(300.0),
            stream_first_byte_timeout_seconds: Some(30.0),
            stream_idle_timeout_seconds: Some(30.0),
            cache_ttl_minutes: 5,
            key_rpm_limit: None,
            is_cached: false,
            route: CandidateRoute { options: Vec::new() },
        }
    }

    fn trace() -> CandidateTrace {
        CandidateTrace {
            token_id: Some("token-1".into()),
            user_id_snapshot: Some("user-1".into()),
            username_snapshot: Some("alice".into()),
            token_name_snapshot: Some("token".into()),
            token_prefix_snapshot: Some("sk-test".into()),
            group_code: Some("default".into()),
            global_model_id: "model-1".into(),
            provider_model_id: "provider-model-1".into(),
            model_name_snapshot: "gpt-5.5".into(),
            provider_id: "provider-1".into(),
            provider_name_snapshot: "Provider".into(),
            endpoint_id: "endpoint-1".into(),
            endpoint_name_snapshot: "endpoint".into(),
            key_id: "key-1".into(),
            key_name_snapshot: "Key".into(),
            key_preview_snapshot: "***test".into(),
            client_api_format: "openai:cli".into(),
            provider_api_format: "openai:cli".into(),
            needs_conversion: false,
            is_stream: true,
            is_cached: false,
            routing_profile_id: types::provider::RoutingProfileId::Balanced,
            routing_profile_ema_alpha: types::provider::default_ema_alpha(),
            routing_context_key: "group=default|model=model-1|format=openai:cli|stream=true|size=unknown|cap=none".into(),
            route_config_fingerprint: "route-fingerprint".into(),
            price_config_fingerprint: "price-fingerprint".into(),
            candidate_index: 0,
        }
    }
}

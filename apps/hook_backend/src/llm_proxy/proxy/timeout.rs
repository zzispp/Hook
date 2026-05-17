use std::time::Duration;

use crate::llm_proxy::candidate::ProxyCandidate;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ProxyTimeouts {
    pub(super) request: Option<Duration>,
    pub(super) stream_first_byte: Option<Duration>,
}

pub(super) fn proxy_timeouts(candidate: &ProxyCandidate) -> ProxyTimeouts {
    ProxyTimeouts {
        request: candidate.request_timeout_seconds.and_then(timeout_duration),
        stream_first_byte: candidate.stream_first_byte_timeout_seconds.and_then(timeout_duration),
    }
}

pub(super) fn timeout_duration(seconds: f64) -> Option<Duration> {
    (seconds.is_finite() && seconds > 0.0).then(|| Duration::from_secs_f64(seconds))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rust_decimal::Decimal;
    use types::model::TieredPricingConfig;

    use super::proxy_timeouts;
    use crate::llm_proxy::candidate::{CandidateRoute, CandidateTrace, ProxyCandidate};

    #[test]
    fn stream_keeps_request_timeout_separate_from_first_byte_timeout() {
        let candidate = candidate();

        let timeouts = proxy_timeouts(&candidate);

        assert_eq!(timeouts.request, Some(Duration::from_secs(300)));
        assert_eq!(timeouts.stream_first_byte, Some(Duration::from_secs(30)));
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
            price_per_request: None,
            tiered_pricing: TieredPricingConfig { tiers: Vec::new() },
            billing_multiplier: Decimal::ONE,
            max_retries: 0,
            request_timeout_seconds: Some(300.0),
            stream_first_byte_timeout_seconds: Some(30.0),
            cache_ttl_minutes: 5,
            key_rpm_limit: None,
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
            model_name_snapshot: "gpt-5.5".into(),
            provider_id: "provider-1".into(),
            provider_name_snapshot: "Provider".into(),
            endpoint_id: "endpoint-1".into(),
            endpoint_name_snapshot: "endpoint".into(),
            key_id: "key-1".into(),
            key_name_snapshot: "Key".into(),
            key_preview_snapshot: "***test".into(),
            client_api_format: "openai_cli".into(),
            provider_api_format: "openai_cli".into(),
            needs_conversion: false,
            is_stream: true,
            candidate_index: 0,
        }
    }
}

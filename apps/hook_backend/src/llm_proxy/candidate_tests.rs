use rust_decimal::Decimal;
use types::model::TieredPricingConfig;

use super::{CandidateEndpointOption, CandidateKeyOption, CandidateRoute, CandidateRouteOption, CandidateTrace, ProxyCandidate};

#[test]
fn proxy_candidate_materializes_route_options_by_retry_index() {
    let candidate = route_candidate();

    let first = candidate.for_attempt(0);
    let second = candidate.for_attempt(1);
    let third = candidate.for_attempt(2);
    let cycled = candidate.for_attempt(9);

    assert_eq!(first.trace.endpoint_id, "endpoint-openai");
    assert_eq!(first.trace.key_id, "key-a-1");
    assert_eq!(second.trace.endpoint_id, "endpoint-openai");
    assert_eq!(second.trace.key_id, "key-a-2");
    assert_eq!(third.trace.endpoint_id, "endpoint-gemini");
    assert_eq!(third.trace.key_id, "key-a-1");
    assert_eq!(cycled.trace.endpoint_id, "endpoint-openai");
    assert_eq!(cycled.trace.key_id, "key-a-2");
}

#[test]
fn route_retry_floor_covers_non_cached_provider_key_options() {
    let candidate = route_candidate();

    assert!(!candidate.is_cached);
    assert_eq!(candidate.route_retry_floor(), 3);
}

#[test]
fn non_cached_candidate_attempts_each_route_option_once() {
    let candidate = route_candidate();

    assert_eq!(candidate.max_attempt_index(), 3);
}

#[test]
fn cached_candidate_keeps_configured_retries_after_route_floor() {
    let candidate = ProxyCandidate {
        is_cached: true,
        max_retries: 5,
        ..route_candidate()
    };

    assert_eq!(candidate.max_attempt_index(), 5);
}

fn route_candidate() -> ProxyCandidate {
    ProxyCandidate {
        trace: trace(),
        requested_model_name: "gpt-5.5".into(),
        api_key: "key-1-secret".into(),
        base_url: "https://openai.example.com".into(),
        custom_path: None,
        upstream_url: "https://openai.example.com/v1/chat/completions".into(),
        provider_model_name: "upstream-model".into(),
        reasoning_effort: None,
        header_rules: None,
        body_rules: None,
        format_acceptance_config: None,
        key_supports_image_generation: false,
        price_per_request: None,
        tiered_pricing: TieredPricingConfig { tiers: Vec::new() },
        billing_multiplier: Decimal::ONE,
        max_retries: 3,
        request_timeout_seconds: None,
        stream_first_byte_timeout_seconds: None,
        stream_first_output_timeout_seconds: None,
        stream_idle_timeout_seconds: None,
        cache_ttl_minutes: 5,
        key_rpm_limit: None,
        is_cached: false,
        route: route(),
    }
}

fn route() -> CandidateRoute {
    CandidateRoute {
        options: vec![
            route_option(endpoint("endpoint-openai", "openai:chat", false), key("key-a-1", "key-1-secret")),
            route_option(endpoint("endpoint-openai", "openai:chat", false), key("key-a-2", "key-2-secret")),
            route_option(endpoint("endpoint-gemini", "gemini:chat", true), key("key-a-1", "key-1-secret")),
            route_option(endpoint("endpoint-gemini", "gemini:chat", true), key("key-a-2", "key-2-secret")),
        ],
    }
}

fn trace() -> CandidateTrace {
    CandidateTrace {
        token_id: Some("token-a".into()),
        user_id_snapshot: Some("user-a".into()),
        username_snapshot: Some("alice".into()),
        token_name_snapshot: Some("token-a-name".into()),
        token_prefix_snapshot: Some("sk-a".into()),
        group_code: Some("default".into()),
        global_model_id: "model-a".into(),
        provider_model_id: "provider-model-a".into(),
        model_name_snapshot: "model-a".into(),
        provider_id: "provider-a".into(),
        provider_name_snapshot: "provider-a-name".into(),
        endpoint_id: "endpoint-openai".into(),
        endpoint_name_snapshot: "openai:chat".into(),
        key_id: "key-a-1".into(),
        key_name_snapshot: "key-a-1-name".into(),
        key_preview_snapshot: "***cret".into(),
        client_api_format: "openai:chat".into(),
        provider_api_format: "openai:chat".into(),
        needs_conversion: false,
        is_stream: false,
        is_cached: false,
        routing_profile_id: types::provider::RoutingProfileId::Balanced,
        routing_profile_ema_alpha: types::provider::default_ema_alpha(),
        routing_context_key: "group=default|model=model-a|format=openai:chat|stream=false|size=unknown|cap=none".into(),
        route_config_fingerprint: "route-fingerprint".into(),
        price_config_fingerprint: "price-fingerprint".into(),
        candidate_index: 0,
    }
}

fn route_option(endpoint: CandidateEndpointOption, key: CandidateKeyOption) -> CandidateRouteOption {
    CandidateRouteOption { endpoint, key }
}

fn endpoint(id: &str, provider_api_format: &str, needs_conversion: bool) -> CandidateEndpointOption {
    CandidateEndpointOption {
        id: id.into(),
        name: provider_api_format.into(),
        provider_api_format: provider_api_format.into(),
        base_url: format!("https://{id}.example.com"),
        custom_path: None,
        upstream_url: format!("https://{id}.example.com/v1/chat/completions"),
        max_retries: None,
        header_rules: None,
        body_rules: None,
        format_acceptance_config: None,
        needs_conversion,
    }
}

fn key(id: &str, api_key: &str) -> CandidateKeyOption {
    CandidateKeyOption {
        id: id.into(),
        name: format!("{id}-name"),
        key_preview: "***cret".into(),
        api_key: api_key.into(),
        supports_image_generation: false,
        cache_ttl_minutes: 5,
        rpm_limit: None,
    }
}

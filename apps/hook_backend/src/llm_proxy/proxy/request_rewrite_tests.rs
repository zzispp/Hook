use proxy::format_conversion::ApiFormat;
use rust_decimal::Decimal;
use serde_json::json;
use types::model::TieredPricingConfig;

use super::{apply_reasoning_effort, rewrite_upstream_body};
use crate::llm_proxy::candidate::{CandidateRoute, CandidateTrace, ProxyCandidate};

#[test]
fn reasoning_effort_override_sets_openai_chat_field() {
    let mut body = json!({"model": "gpt-5.5"});

    apply_reasoning_effort(&mut body, &candidate("openai:chat"), ApiFormat::OpenAiChat).unwrap();

    assert_eq!(body["reasoning_effort"], "high");
}

#[test]
fn reasoning_effort_override_sets_openai_responses_nested_field() {
    let mut body = json!({"model": "gpt-5.5"});

    apply_reasoning_effort(&mut body, &candidate("openai:cli"), ApiFormat::OpenAiResponses).unwrap();

    assert_eq!(body["reasoning"]["effort"], "high");
}

#[test]
fn openai_chat_stream_requests_include_usage() {
    let mut body = json!({
        "model": "gpt-5.5",
        "messages": [{"role": "user", "content": "hello"}],
        "stream": true
    });

    rewrite_upstream_body(&mut body, &candidate("openai:chat"), false, ApiFormat::OpenAiChat).unwrap();

    assert_eq!(body["stream_options"]["include_usage"], true);
}

#[test]
fn openai_image_default_stream_request_removes_upstream_stream_flag() {
    let mut body = json!({
        "model": "gpt-image-2",
        "prompt": "draw a green square",
        "stream": true,
        "partial_images": 1
    });

    rewrite_upstream_body(&mut body, &candidate("openai_image"), false, ApiFormat::OpenAiImage).unwrap();

    assert!(body.get("stream").is_none());
    assert!(body.get("partial_images").is_none());
    assert_eq!(body["model"], "upstream-model");
    assert!(body.get("stream_options").is_none());
}

#[test]
fn openai_image_native_stream_request_keeps_stream_flag() {
    let mut body = json!({
        "model": "gpt-image-2",
        "prompt": "draw a green square",
        "stream": true,
        "partial_images": 1
    });
    let mut candidate = candidate("openai_image");
    candidate.format_acceptance_config = Some(json!({"upstream_image_stream_mode": "native_stream"}));

    rewrite_upstream_body(&mut body, &candidate, false, ApiFormat::OpenAiImage).unwrap();

    assert_eq!(body["stream"], true);
    assert_eq!(body["partial_images"], 1);
    assert_eq!(body["model"], "upstream-model");
    assert!(body.get("stream_options").is_none());
}

#[test]
fn openai_image_force_non_stream_removes_stream_flag() {
    let mut body = json!({
        "model": "gpt-image-2",
        "prompt": "draw a green square",
        "stream": true,
        "partial_images": 1
    });
    let mut candidate = candidate("openai_image");
    candidate.format_acceptance_config = Some(json!({"upstream_image_stream_mode": "native_stream"}));

    rewrite_upstream_body(&mut body, &candidate, true, ApiFormat::OpenAiImage).unwrap();

    assert!(body.get("stream").is_none());
    assert!(body.get("partial_images").is_none());
    assert_eq!(body["model"], "upstream-model");
    assert!(body.get("stream_options").is_none());
}

fn candidate(provider_api_format: &str) -> ProxyCandidate {
    ProxyCandidate {
        trace: CandidateTrace {
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
            endpoint_name_snapshot: provider_api_format.into(),
            key_id: "key-1".into(),
            key_name_snapshot: "Key".into(),
            key_preview_snapshot: "***test".into(),
            client_api_format: "openai:chat".into(),
            provider_api_format: provider_api_format.into(),
            needs_conversion: false,
            is_stream: false,
            is_cached: false,
            routing_profile_id: types::provider::RoutingProfileId::Balanced,
            routing_profile_ema_alpha: types::provider::default_ema_alpha(),
            routing_context_key: "group=default|model=model-1|format=openai:chat|stream=false|size=unknown|cap=none".into(),
            route_config_fingerprint: "route-fingerprint".into(),
            price_config_fingerprint: "price-fingerprint".into(),
            candidate_index: 0,
        },
        requested_model_name: "gpt-5.5".into(),
        api_key: "secret".into(),
        base_url: "https://example.com".into(),
        custom_path: None,
        upstream_url: "https://example.com/v1/chat/completions".into(),
        provider_model_name: "upstream-model".into(),
        reasoning_effort: Some("high".into()),
        header_rules: None,
        body_rules: None,
        format_acceptance_config: None,
        key_supports_image_generation: false,
        price_per_request: None,
        tiered_pricing: TieredPricingConfig { tiers: Vec::new() },
        billing_multiplier: Decimal::ONE,
        max_retries: 0,
        request_timeout_seconds: None,
        stream_first_byte_timeout_seconds: None,
        stream_idle_timeout_seconds: None,
        cache_ttl_minutes: 5,
        key_rpm_limit: None,
        is_cached: false,
        route: CandidateRoute { options: Vec::new() },
    }
}

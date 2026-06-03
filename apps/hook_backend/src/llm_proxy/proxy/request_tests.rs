use rust_decimal::Decimal;
use serde_json::json;
use types::model::TieredPricingConfig;

use super::{apply_reasoning_effort, bridge_openai_chat_image_body, openai_request_explicitly_selects_image_generation, rewrite_upstream_body};
use crate::llm_proxy::candidate::{CandidateRoute, CandidateTrace, ProxyCandidate};
use proxy::format_conversion::ApiFormat;

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
fn image_generation_intent_requires_explicit_tool_choice() {
    let body = json!({
        "model": "gpt-5.5",
        "input": "draw",
        "tools": [{"type": "image_generation"}]
    });

    assert!(!openai_request_explicitly_selects_image_generation("openai:cli", &body));
}

#[test]
fn image_generation_intent_accepts_object_tool_choice() {
    let body = json!({
        "model": "gpt-5.5",
        "input": "draw",
        "tools": [{"type": "image_generation"}],
        "tool_choice": {"type": "image_generation"}
    });

    assert!(openai_request_explicitly_selects_image_generation("openai:cli", &body));
}

#[test]
fn chat_image_bridge_preserves_image_tool_declaration() {
    let body = json!({
        "model": "gpt-5.5",
        "messages": [{"role": "user", "content": "draw a cat"}],
        "tools": [{"type": "image_generation", "size": "1024x1024"}],
        "tool_choice": {"type": "image_generation"}
    });

    let bridged = bridge_openai_chat_image_body(body).unwrap();

    assert_eq!(bridged["input"][0]["content"], "draw a cat");
    assert_eq!(bridged["tools"][0]["type"], "image_generation");
    assert_eq!(bridged["tools"][0]["size"], "1024x1024");
    assert_eq!(bridged["tool_choice"]["type"], "image_generation");
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

use rust_decimal::Decimal;
use serde_json::json;
use types::model::TieredPricingConfig;

use super::super::{request_image::bridge_openai_chat_image_body, request_tools::openai_request_explicitly_selects_image_generation};
use super::{AttemptContext, attempt_payload, has_openai_responses_custom_tool_items};
use crate::llm_proxy::{
    OPENAI_CHAT_FORMAT, OPENAI_CLI_FORMAT,
    candidate::{CandidateRoute, CandidateTrace, ProxyCandidate},
    codex_chat_history::{CodexChatHistoryStore, test_support::test_history},
};

#[tokio::test]
async fn openai_cli_to_chat_body_rule_can_drop_stream_options() {
    let body = json!({
        "model": "gpt-5.5",
        "input": [{
            "type": "message",
            "role": "user",
            "content": [{"type": "input_text", "text": "hello"}]
        }],
        "stream": true
    });
    let mut candidate = candidate("openai:chat");
    candidate.trace.client_api_format = "openai:cli".into();
    candidate.trace.needs_conversion = true;
    candidate.trace.is_stream = true;
    candidate.reasoning_effort = None;
    candidate.body_rules = Some(json!([
        {"action": "drop", "path": "stream_options"}
    ]));

    let history = test_history().await;
    let payload = attempt_payload(attempt_context(&history), body, &candidate, false).await.unwrap();

    assert_eq!(payload.body["stream"], true);
    assert!(payload.body.get("stream_options").is_none());
    assert_eq!(payload.body["messages"][0]["role"], "user");
}

#[tokio::test]
async fn openai_cli_prunes_image_generation_tool_when_key_lacks_capability() {
    let body = json!({
        "model": "gpt-5.5",
        "input": "hello",
        "tool_choice": "auto",
        "tools": [
            {"type": "image_generation"},
            {"type": "function", "name": "lookup"}
        ]
    });
    let mut candidate = candidate("openai:cli");
    candidate.trace.client_api_format = "openai:cli".into();
    candidate.reasoning_effort = None;

    let history = test_history().await;
    let payload = attempt_payload(attempt_context(&history), body, &candidate, false).await.unwrap();

    assert_eq!(payload.body["tools"], json!([{"type": "function", "name": "lookup"}]));
    assert_eq!(payload.body["tool_choice"], "auto");
}

#[tokio::test]
async fn openai_cli_keeps_image_generation_tool_when_key_has_capability() {
    let body = json!({
        "model": "gpt-5.5",
        "input": "hello",
        "tools": [{"type": "image_generation"}]
    });
    let mut candidate = candidate("openai:cli");
    candidate.trace.client_api_format = "openai:cli".into();
    candidate.reasoning_effort = None;
    candidate.key_supports_image_generation = true;

    let history = test_history().await;
    let payload = attempt_payload(attempt_context(&history), body, &candidate, false).await.unwrap();

    assert_eq!(payload.body["tools"], json!([{"type": "image_generation"}]));
}

#[tokio::test]
async fn openai_cli_keeps_explicit_image_generation_tool_choice() {
    let body = json!({
        "model": "gpt-5.5",
        "input": "draw",
        "tools": [{"type": "image_generation"}],
        "tool_choice": {"type": "image_generation"}
    });
    let mut candidate = candidate("openai:cli");
    candidate.trace.client_api_format = "openai:cli".into();
    candidate.reasoning_effort = None;

    let history = test_history().await;
    let payload = attempt_payload(attempt_context(&history), body, &candidate, false).await.unwrap();

    assert_eq!(payload.body["tools"], json!([{"type": "image_generation"}]));
    assert_eq!(payload.body["tool_choice"]["type"], "image_generation");
}

#[tokio::test]
async fn openai_cli_removes_tools_field_when_only_image_generation_was_pruned() {
    let body = json!({
        "model": "gpt-5.5",
        "input": "hello",
        "tools": [{"type": "image_generation"}]
    });
    let mut candidate = candidate("openai:cli");
    candidate.trace.client_api_format = "openai:cli".into();
    candidate.reasoning_effort = None;

    let history = test_history().await;
    let payload = attempt_payload(attempt_context(&history), body, &candidate, false).await.unwrap();

    assert!(payload.body.get("tools").is_none());
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
fn responses_custom_tool_feature_detects_custom_tool_input_items() {
    let body = json!({
        "model": "gpt-test",
        "input": [
            { "type": "message", "role": "user", "content": "hello" },
            { "type": "custom_tool_call", "call_id": "call_1", "name": "apply_patch" }
        ]
    });

    assert!(has_openai_responses_custom_tool_items(OPENAI_CLI_FORMAT, &body));
}

#[test]
fn responses_custom_tool_feature_ignores_image_generation_call_input_items() {
    let body = json!({
        "model": "gpt-test",
        "input": [
            { "type": "message", "role": "user", "content": "continue" },
            { "type": "image_generation_call", "id": "ig_1", "result": "aGVsbG8=" }
        ]
    });

    assert!(!has_openai_responses_custom_tool_items(OPENAI_CLI_FORMAT, &body));
}

#[test]
fn responses_custom_tool_feature_ignores_non_responses_requests() {
    let body = json!({
        "model": "gpt-test",
        "input": [{ "type": "custom_tool_call_output", "call_id": "call_1" }]
    });

    assert!(!has_openai_responses_custom_tool_items(OPENAI_CHAT_FORMAT, &body));
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

fn attempt_context(history: &CodexChatHistoryStore) -> AttemptContext<'_> {
    AttemptContext { codex_chat_history: history }
}

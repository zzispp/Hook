use rust_decimal::Decimal;
use serde_json::json;
use types::model::TieredPricingConfig;

use super::super::request::{AttemptContext, attempt_payload};
use crate::llm_proxy::{
    LlmProxyError,
    candidate::{CandidateRoute, CandidateTrace, ProxyCandidate},
    codex_chat_history::{CodexChatHistoryStore, test_support::test_history},
};

#[tokio::test]
async fn openai_cli_to_chat_enriches_tool_output_from_history() {
    let history = test_history().await;
    history
        .record_response(&json!({
            "id": "resp_1",
            "output": [{
                "type": "function_call",
                "call_id": "call_1",
                "name": "read_file",
                "arguments": "{\"path\":\"README.md\"}"
            }]
        }))
        .await
        .unwrap();
    let mut candidate = candidate("openai:chat");
    candidate.trace.client_api_format = "openai:cli".into();
    candidate.trace.needs_conversion = true;
    candidate.reasoning_effort = None;

    let payload = attempt_payload(attempt_context(&history), tool_output_body(), &candidate, false).await.unwrap();

    assert_eq!(payload.body["messages"][0]["role"], "assistant");
    assert_eq!(payload.body["messages"][0]["tool_calls"][0]["id"], "call_1");
    assert_eq!(payload.body["messages"][1]["role"], "tool");
    assert_eq!(payload.body["messages"][1]["tool_call_id"], "call_1");
}

#[tokio::test]
async fn openai_cli_to_chat_errors_when_history_is_missing() {
    let history = test_history().await;
    let mut candidate = candidate("openai:chat");
    candidate.trace.client_api_format = "openai:cli".into();
    candidate.trace.needs_conversion = true;

    let error = match attempt_payload(attempt_context(&history), tool_output_body(), &candidate, false).await {
        Ok(_) => panic!("request should fail when Codex chat history is missing"),
        Err(error) => error,
    };

    assert!(matches!(error, LlmProxyError::InvalidRequest(message) if message.contains("missing Codex chat history")));
}

fn tool_output_body() -> serde_json::Value {
    json!({
        "model": "gpt-5.5",
        "previous_response_id": "resp_1",
        "input": [{
            "type": "function_call_output",
            "call_id": "call_1",
            "output": "ok"
        }]
    })
}

fn attempt_context(history: &CodexChatHistoryStore) -> AttemptContext<'_> {
    AttemptContext { codex_chat_history: history }
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

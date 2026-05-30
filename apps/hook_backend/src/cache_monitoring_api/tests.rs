use std::collections::HashMap;

use axum::{body::to_bytes, response::IntoResponse};
use serde_json::Value;
use types::api_token::ApiTokenOwnerResponse;
use types::cache_monitoring::CacheAffinityItem;

use super::{CacheMonitoringApiError, filter_items, normalized_search, resolve_owner};

fn cache_item() -> CacheAffinityItem {
    CacheAffinityItem {
        affinity_key: "token-1".into(),
        user_id: Some("user-1".into()),
        username: Some("alice".into()),
        user_email: Some("alice@example.com".into()),
        token_name: Some("demo token".into()),
        token_prefix: Some("hk-demo".into()),
        provider_id: "provider-1".into(),
        provider_name: Some("OpenAI Proxy".into()),
        endpoint_id: "endpoint-1".into(),
        endpoint_base_url: Some("https://api.example.com".into()),
        endpoint_api_format: Some("openai:chat".into()),
        provider_key_id: "key-1".into(),
        provider_key_name: Some("primary".into()),
        model_id: "model-1".into(),
        model_name: Some("gpt-4.1".into()),
        api_format: "openai:chat".into(),
        ttl_seconds: 120,
        request_count: 4,
    }
}

#[test]
fn filter_items_matches_keyword() {
    let filtered = filter_items(vec![cache_item()], normalized_search(Some("alice@example.com")));

    assert_eq!(filtered.len(), 1);
}

#[test]
fn resolve_owner_uses_system_owner() {
    let system_owner = Some((
        "system-user".into(),
        ApiTokenOwnerResponse {
            username: "codex".into(),
            email: "codex@example.com".into(),
            group_codes: vec![constants::user_group::DEFAULT_USER_GROUP_CODE.into()],
        },
    ));

    let owner = resolve_owner(Some("system-user"), &HashMap::new(), system_owner.as_ref()).unwrap();

    assert_eq!(owner.username, "codex");
    assert_eq!(owner.email, "codex@example.com");
}

#[tokio::test]
async fn infrastructure_error_maps_to_503() {
    let response = CacheMonitoringApiError::from(crate::llm_proxy::LlmProxyError::Infrastructure("redis down".into())).into_response();

    assert_eq!(response.status(), axum::http::StatusCode::SERVICE_UNAVAILABLE);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload["message"], "redis down");
}

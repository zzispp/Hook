use types::api_token::ApiTokenType;

use super::helpers::{api_token, provider_with_endpoints_and_keys, snapshot_with_provider, user_access};
use crate::llm_proxy::{
    LlmProxyError,
    cache::snapshot::SchedulingSnapshot,
    model_access::{token_user_for_snapshot, user_access_for_token},
};

#[test]
fn token_user_snapshot_keeps_independent_token_owner() {
    let snapshot = SchedulingSnapshot {
        users: vec![user_access("user-a", "alice", Vec::new())],
        ..snapshot_with_provider(provider_with_endpoints_and_keys())
    };
    let token = api_token(ApiTokenType::Independent, Some("user-a"));

    let token_user = token_user_for_snapshot(&snapshot, &token).unwrap();

    assert_eq!(token_user.map(|user| user.username.as_str()), Some("alice"));
    assert!(user_access_for_token(&token, token_user).is_none());
}

#[test]
fn token_user_snapshot_rejects_orphaned_user_token() {
    let snapshot = snapshot_with_provider(provider_with_endpoints_and_keys());
    let token = api_token(ApiTokenType::User, Some("missing-user"));

    let error = token_user_for_snapshot(&snapshot, &token).unwrap_err();

    assert!(matches!(error, LlmProxyError::CodedForbidden { code: "hook_api_error", .. }));
}

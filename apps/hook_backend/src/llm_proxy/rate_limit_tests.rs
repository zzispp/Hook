use rust_decimal::Decimal;
use types::{
    api_token::{ApiToken, ApiTokenType, ModelAccessMode},
    provider::ProviderSchedulingMode,
    system_setting::RequestRecordLevel,
};

use super::{CachedUserAccess, RateLimitScope, SchedulingSnapshot, provider_key_probe_slot_command, request_scopes, token_scope};

#[test]
fn provider_key_probe_throttle_uses_atomic_set_nx_ex() {
    let command = provider_key_probe_slot_command("hook", "key-1", 2);

    let packed = String::from_utf8(command.get_packed_command()).unwrap();

    assert_eq!(
        packed,
        "*6\r\n$3\r\nSET\r\n$44\r\nhook:llm_proxy:model_status_probe_slot:key-1\r\n$1\r\n1\r\n$2\r\nNX\r\n$2\r\nEX\r\n$1\r\n2\r\n"
    );
}

#[test]
fn token_limit_follows_system_when_configured_zero() {
    let snapshot = snapshot(7, None);
    let scope = token_scope(&snapshot, &token(ApiTokenType::Independent, None, Some(0)));

    assert_eq!(scope, Some(RateLimitScope::token("token-1", 7)));
}

#[test]
fn token_limit_uses_smaller_of_system_and_token() {
    let snapshot = snapshot(7, None);
    let scope = token_scope(&snapshot, &token(ApiTokenType::Independent, None, Some(3)));

    assert_eq!(scope, Some(RateLimitScope::token("token-1", 3)));
}

#[test]
fn token_limit_uses_configured_value_when_system_unlimited() {
    let snapshot = snapshot(0, None);
    let scope = token_scope(&snapshot, &token(ApiTokenType::Independent, None, Some(3)));

    assert_eq!(scope, Some(RateLimitScope::token("token-1", 3)));
}

#[test]
fn request_scopes_include_user_and_token_for_user_tokens() {
    let snapshot = snapshot(7, Some(2));
    let scopes = request_scopes(&snapshot, &token(ApiTokenType::User, Some("user-1"), Some(5)));

    assert_eq!(scopes, vec![RateLimitScope::user("user-1", 2), RateLimitScope::token("token-1", 5)]);
}

#[test]
fn request_scopes_skip_user_limit_for_independent_tokens() {
    let snapshot = snapshot(7, Some(2));
    let scopes = request_scopes(&snapshot, &token(ApiTokenType::Independent, None, Some(5)));

    assert_eq!(scopes, vec![RateLimitScope::token("token-1", 5)]);
}

fn snapshot(default_rate_limit_rpm: i64, user_rate_limit_rpm: Option<i64>) -> SchedulingSnapshot {
    SchedulingSnapshot {
        default_rate_limit_rpm,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        provider_priority_mode: types::provider::ProviderPriorityMode::Provider,
        cache_affinity_ttl_minutes: 5,
        client_request_record_level: RequestRecordLevel::Basic,
        client_record_request_headers: true,
        client_record_request_body: true,
        client_record_response_headers: true,
        client_record_response_body: true,
        client_max_request_body_size_kb: 1024,
        client_max_response_body_size_kb: 1024,
        client_sensitive_request_headers: String::new(),
        provider_request_record_level: RequestRecordLevel::Basic,
        provider_record_request_headers: true,
        provider_record_request_body: true,
        provider_record_response_headers: true,
        provider_record_response_body: true,
        provider_max_request_body_size_kb: 1024,
        provider_max_response_body_size_kb: 1024,
        provider_sensitive_request_headers: String::new(),
        provider_cooldown_policy: Default::default(),
        models: Vec::new(),
        groups: Vec::new(),
        active_user_group_codes: vec!["default".into()],
        users: vec![CachedUserAccess {
            id: "user-1".into(),
            username: "alice".into(),
            group_codes: vec!["default".into()],
            is_active: true,
            allowed_model_ids: Vec::new(),
            allowed_provider_ids: Vec::new(),
            quota_mode: "wallet".into(),
            rate_limit_rpm: user_rate_limit_rpm,
        }],
        providers: Vec::new(),
    }
}

fn token(token_type: ApiTokenType, user_id: Option<&str>, rate_limit_rpm: Option<i64>) -> ApiToken {
    ApiToken {
        id: "token-1".into(),
        user_id: user_id.map(str::to_owned),
        token_type,
        name: "token".into(),
        token_value: String::new(),
        token_hash: String::new(),
        token_prefix: "sk-test".into(),
        group_code: "default".into(),
        expires_at: None,
        model_access_mode: ModelAccessMode::All,
        allowed_model_ids: Vec::new(),
        rate_limit_rpm,
        quota_limit: None,
        used_quota: Decimal::ZERO,
        request_count: 0,
        is_active: true,
        last_used_at: None,
        created_at: String::new(),
        updated_at: String::new(),
    }
}

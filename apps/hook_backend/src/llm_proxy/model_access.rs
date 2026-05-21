use types::api_token::ApiToken;

use super::{
    LlmProxyError,
    cache::snapshot::{CachedBillingGroup, CachedGlobalModel, CachedProvider, CachedUserAccess, SchedulingSnapshot},
    client_error,
};

pub(crate) fn visible_models_for_token<'a>(snapshot: &'a SchedulingSnapshot, token: &ApiToken) -> Result<Vec<&'a CachedGlobalModel>, LlmProxyError> {
    let token_user = token_user_for_snapshot(snapshot, token)?;
    let user_access = user_access_for_token(token, token_user);
    let group = active_group(snapshot, token)?;
    Ok(snapshot
        .models
        .iter()
        .filter(|model| model.is_active)
        .filter(|model| ensure_token_allows_model(token, &model.id).is_ok())
        .filter(|model| ensure_group_allows_model(group, &model.id).is_ok())
        .filter(|model| ensure_user_allows_model(user_access, &model.id).is_ok())
        .collect())
}

pub(crate) fn visible_model_for_token<'a>(snapshot: &'a SchedulingSnapshot, token: &ApiToken, value: &str) -> Result<&'a CachedGlobalModel, LlmProxyError> {
    visible_models_for_token(snapshot, token)?
        .into_iter()
        .find(|model| model.id == value || model.name == value)
        .ok_or_else(|| LlmProxyError::NotFound(format!("model not found: {value}")))
}

pub(crate) fn active_group<'a>(snapshot: &'a SchedulingSnapshot, token: &ApiToken) -> Result<&'a CachedBillingGroup, LlmProxyError> {
    let group = snapshot
        .groups
        .iter()
        .find(|group| group.code == token.group_code)
        .ok_or_else(|| LlmProxyError::Forbidden(format!("billing group not found: {}", token.group_code)))?;
    if !group.is_active {
        return Err(LlmProxyError::Forbidden(format!("billing group is inactive: {}", group.code)));
    }
    Ok(group)
}

pub(crate) fn ensure_token_allows_model(token: &ApiToken, model_id: &str) -> Result<(), LlmProxyError> {
    if token.model_access_mode == types::api_token::ModelAccessMode::All || token.allowed_model_ids.iter().any(|id| id == model_id) {
        return Ok(());
    }
    Err(LlmProxyError::Forbidden(format!("model is not allowed by token: {model_id}")))
}

pub(crate) fn ensure_group_allows_model(group: &CachedBillingGroup, model_id: &str) -> Result<(), LlmProxyError> {
    if ids_allow(&group.allowed_model_ids, model_id) {
        return Ok(());
    }
    Err(LlmProxyError::Forbidden(format!(
        "model is not allowed by billing group {}: {model_id}",
        group.code
    )))
}

pub(crate) fn ensure_user_allows_model(access: Option<&CachedUserAccess>, model_id: &str) -> Result<(), LlmProxyError> {
    if access.is_none_or(|access| ids_allow(&access.allowed_model_ids, model_id)) {
        return Ok(());
    }
    Err(LlmProxyError::Forbidden(format!("model is not allowed by user: {model_id}")))
}

pub(crate) fn provider_allowed(group: &CachedBillingGroup, user_access: Option<&CachedUserAccess>, provider: &CachedProvider) -> bool {
    provider.is_active
        && ids_allow(&group.allowed_provider_ids, &provider.id)
        && user_access.is_none_or(|access| ids_allow(&access.allowed_provider_ids, &provider.id))
}

pub(crate) fn token_user_for_snapshot<'a>(snapshot: &'a SchedulingSnapshot, token: &ApiToken) -> Result<Option<&'a CachedUserAccess>, LlmProxyError> {
    let Some(user_id) = token.user_id.as_ref() else {
        if token.token_type == types::api_token::ApiTokenType::User {
            return Err(LlmProxyError::Forbidden(format!("user token missing user id: {}", token.id)));
        }
        return Ok(None);
    };
    let user = snapshot.users.iter().find(|user| user.id == *user_id);
    if token.token_type == types::api_token::ApiTokenType::User && user.is_none() {
        return Err(LlmProxyError::hook_api_forbidden(
            "user is disabled or unavailable",
            client_error::HOOK_API_ERROR_TYPE,
        ));
    }
    if token.token_type == types::api_token::ApiTokenType::User && user.is_some_and(|user| !user.is_active) {
        return Err(LlmProxyError::hook_api_forbidden(
            "user is disabled or unavailable",
            client_error::HOOK_API_ERROR_TYPE,
        ));
    }
    Ok(user)
}

pub(crate) fn user_access_for_token<'a>(token: &ApiToken, token_user: Option<&'a CachedUserAccess>) -> Option<&'a CachedUserAccess> {
    if token.token_type != types::api_token::ApiTokenType::User {
        return None;
    }
    token_user
}

pub(crate) fn ids_allow(ids: &[String], id: &str) -> bool {
    ids.is_empty() || ids.iter().any(|item| item == id)
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use types::{
        api_token::{ApiToken, ApiTokenType, ModelAccessMode},
        model::TieredPricingConfig,
        provider::{ProviderModelMapping, ProviderSchedulingMode},
        system_setting::RequestRecordLevel,
    };

    use super::*;
    use crate::llm_proxy::cache::snapshot::{CachedEndpoint, CachedModelBinding, CachedProviderKey};

    #[test]
    fn visible_models_for_token_returns_intersection_of_group_token_and_user_scope() {
        let snapshot = snapshot();
        let token = token(
            ApiTokenType::User,
            "group-a",
            ModelAccessMode::Limited,
            vec!["global-model-a".into(), "global-model-b".into()],
        );

        let models = visible_models_for_token(&snapshot, &token).unwrap();

        assert_eq!(models.iter().map(|model| model.name.as_str()).collect::<Vec<_>>(), vec!["model-a"]);
    }

    #[test]
    fn visible_models_for_token_keeps_visible_models_without_provider_binding() {
        let mut snapshot = snapshot();
        snapshot.providers[0].models.clear();
        let token = token(ApiTokenType::Independent, "group-a", ModelAccessMode::All, Vec::new());

        let models = visible_models_for_token(&snapshot, &token).unwrap();

        assert_eq!(models.iter().map(|model| model.name.as_str()).collect::<Vec<_>>(), vec!["model-a", "model-c"]);
    }

    #[test]
    fn visible_model_for_token_matches_by_name_or_id() {
        let snapshot = snapshot();
        let token = token(ApiTokenType::Independent, "group-a", ModelAccessMode::All, Vec::new());

        let by_name = visible_model_for_token(&snapshot, &token, "model-a").unwrap();
        let by_id = visible_model_for_token(&snapshot, &token, "global-model-a").unwrap();

        assert_eq!(by_name.id, "global-model-a");
        assert_eq!(by_id.name, "model-a");
    }

    #[test]
    fn visible_model_for_token_returns_not_found_for_invisible_model() {
        let snapshot = snapshot();
        let token = token(ApiTokenType::Independent, "group-a", ModelAccessMode::Limited, vec!["global-model-a".into()]);

        let error = visible_model_for_token(&snapshot, &token, "model-c").unwrap_err();

        assert!(matches!(error, LlmProxyError::NotFound(message) if message == "model not found: model-c"));
    }

    #[test]
    fn visible_model_for_token_returns_group_visible_model_without_provider_binding() {
        let mut snapshot = snapshot();
        snapshot.providers[0].models.clear();
        let token = token(ApiTokenType::Independent, "group-a", ModelAccessMode::All, Vec::new());

        let model = visible_model_for_token(&snapshot, &token, "model-c").unwrap();

        assert_eq!(model.id, "global-model-c");
    }

    fn snapshot() -> SchedulingSnapshot {
        SchedulingSnapshot {
            default_rate_limit_rpm: 0,
            scheduling_mode: ProviderSchedulingMode::FixedOrder,
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
            models: vec![
                model("global-model-a", "model-a"),
                model("global-model-b", "model-b"),
                model("global-model-c", "model-c"),
            ],
            groups: vec![CachedBillingGroup {
                code: "group-a".into(),
                billing_multiplier: Decimal::ONE,
                allowed_model_ids: vec!["global-model-a".into(), "global-model-c".into()],
                allowed_provider_ids: Vec::new(),
                is_active: true,
            }],
            users: vec![CachedUserAccess {
                id: "user-a".into(),
                username: "alice".into(),
                is_active: true,
                allowed_model_ids: vec!["global-model-a".into()],
                allowed_provider_ids: Vec::new(),
                quota_mode: "wallet".into(),
                rate_limit_rpm: None,
            }],
            providers: vec![CachedProvider {
                id: "provider-a".into(),
                name: "Provider A".into(),
                max_retries: Some(2),
                request_timeout_seconds: None,
                stream_first_byte_timeout_seconds: None,
                stream_idle_timeout_seconds: None,
                priority: 0,
                keep_priority_on_conversion: false,
                enable_format_conversion: true,
                is_active: true,
                endpoints: vec![CachedEndpoint {
                    id: "endpoint-a".into(),
                    provider_id: "provider-a".into(),
                    api_format: "openai_chat".into(),
                    base_url: "https://example.com".into(),
                    custom_path: None,
                    max_retries: None,
                    is_active: true,
                    format_acceptance_config: None,
                    header_rules: None,
                    body_rules: None,
                }],
                keys: vec![CachedProviderKey {
                    id: "key-a".into(),
                    provider_id: "provider-a".into(),
                    name: "Key A".into(),
                    api_formats: vec!["openai_chat".into()],
                    allowed_model_ids: Vec::new(),
                    key_preview: "sk-***".into(),
                    encrypted_api_key: "encrypted".into(),
                    internal_priority: 0,
                    rpm_limit: None,
                    cache_ttl_minutes: 0,
                    time_range_enabled: false,
                    time_range_start_minute: None,
                    time_range_end_minute: None,
                    is_active: true,
                }],
                models: vec![CachedModelBinding {
                    id: "binding-a".into(),
                    provider_id: "provider-a".into(),
                    global_model_id: "global-model-a".into(),
                    provider_model_name: "provider-model-a".into(),
                    provider_model_mapping: Some(ProviderModelMapping {
                        name: "provider-model-a".into(),
                        reasoning_effort: None,
                    }),
                    is_active: true,
                    price_per_request: None,
                    tiered_pricing: None,
                }],
            }],
        }
    }

    fn model(id: &str, name: &str) -> CachedGlobalModel {
        CachedGlobalModel {
            id: id.into(),
            name: name.into(),
            is_active: true,
            default_price_per_request: None,
            default_tiered_pricing: TieredPricingConfig { tiers: Vec::new() },
        }
    }

    fn token(token_type: ApiTokenType, group_code: &str, mode: ModelAccessMode, allowed_model_ids: Vec<String>) -> ApiToken {
        ApiToken {
            id: "token-a".into(),
            user_id: Some("user-a".into()),
            token_type,
            name: "Token A".into(),
            token_value: "sk-test".into(),
            token_hash: "hash".into(),
            token_prefix: "sk".into(),
            group_code: group_code.into(),
            expires_at: None,
            model_access_mode: mode,
            allowed_model_ids,
            rate_limit_rpm: None,
            quota_limit: None,
            used_quota: Decimal::ZERO,
            request_count: 0,
            is_active: true,
            last_used_at: None,
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: "2026-01-01T00:00:00Z".into(),
        }
    }
}

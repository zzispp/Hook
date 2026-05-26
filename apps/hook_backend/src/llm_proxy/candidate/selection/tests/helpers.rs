use rust_decimal::Decimal;
use types::{
    api_token::{ApiToken, ApiTokenType, ModelAccessMode},
    model::TieredPricingConfig,
    provider::{ProviderModelMapping, ProviderSchedulingMode},
    system_setting::RequestRecordLevel,
};

use crate::llm_proxy::{
    cache::snapshot::{
        CachedBillingGroup, CachedEndpoint, CachedGlobalModel, CachedModelBinding, CachedProvider, CachedProviderKey, CachedUserAccess, SchedulingSnapshot,
    },
    candidate::CandidateRequest,
};

pub(super) fn snapshot_with_provider(provider: CachedProvider) -> SchedulingSnapshot {
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
        models: vec![CachedGlobalModel {
            id: "model-a".into(),
            name: "gpt-test".into(),
            is_active: true,
            default_price_per_request: None,
            default_tiered_pricing: TieredPricingConfig { tiers: Vec::new() },
        }],
        groups: vec![CachedBillingGroup {
            code: "default".into(),
            billing_multiplier: Decimal::ONE,
            allowed_model_ids: Vec::new(),
            allowed_provider_ids: Vec::new(),
            is_active: true,
        }],
        users: Vec::new(),
        providers: vec![provider],
    }
}

pub(super) fn user_access(id: &str, username: &str, allowed_provider_ids: Vec<String>) -> CachedUserAccess {
    CachedUserAccess {
        id: id.into(),
        username: username.into(),
        is_active: true,
        allowed_model_ids: Vec::new(),
        allowed_provider_ids,
        quota_mode: "wallet".into(),
        rate_limit_rpm: None,
    }
}

pub(super) fn provider_with_endpoints_and_keys() -> CachedProvider {
    CachedProvider {
        id: "provider-a".into(),
        name: "Provider A".into(),
        max_retries: Some(2),
        request_timeout_seconds: None,
        stream_first_byte_timeout_seconds: None,
        stream_idle_timeout_seconds: None,
        priority: 10,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active: true,
        endpoints: vec![
            endpoint("endpoint-gemini", "gemini:chat"),
            endpoint("endpoint-openai", "openai:chat"),
            endpoint("endpoint-image", "openai_image"),
        ],
        keys: vec![key("key-a-2", 20), key("key-a-1", 10)],
        models: vec![CachedModelBinding {
            id: "binding-a".into(),
            provider_id: "provider-a".into(),
            global_model_id: "model-a".into(),
            provider_model_name: "upstream-model".into(),
            provider_model_mapping: Some(ProviderModelMapping {
                name: "mapped-upstream-model".into(),
                reasoning_effort: Some("high".into()),
            }),
            is_active: true,
        }],
    }
}

pub(super) fn provider_key(id: &str, internal_priority: i32, api_formats: Vec<&str>) -> CachedProviderKey {
    let mut output = key(id, internal_priority);
    output.api_formats = api_formats.into_iter().map(str::to_owned).collect();
    output
}

pub(super) fn provider_key_for_models(id: &str, internal_priority: i32, api_formats: Vec<&str>, model_ids: Vec<&str>) -> CachedProviderKey {
    let mut output = provider_key(id, internal_priority, api_formats);
    output.allowed_model_ids = model_ids.into_iter().map(str::to_owned).collect();
    output
}

pub(super) fn provider_key_with_time_range(id: &str, internal_priority: i32, start_minute: u16, end_minute: u16) -> CachedProviderKey {
    let mut output = provider_key(id, internal_priority, vec!["openai:chat"]);
    output.time_range_enabled = true;
    output.time_range_start_minute = Some(start_minute);
    output.time_range_end_minute = Some(end_minute);
    output
}

pub(super) fn provider_with_keys(keys: Vec<CachedProviderKey>) -> CachedProvider {
    CachedProvider {
        keys,
        ..provider_with_endpoints_and_keys()
    }
}

pub(super) fn provider_b() -> CachedProvider {
    CachedProvider {
        id: "provider-b".into(),
        name: "Provider B".into(),
        priority: 20,
        endpoints: vec![CachedEndpoint {
            provider_id: "provider-b".into(),
            ..endpoint("endpoint-b-openai", "openai:chat")
        }],
        keys: vec![CachedProviderKey {
            provider_id: "provider-b".into(),
            ..key("key-b-1", 10)
        }],
        models: vec![CachedModelBinding {
            id: "binding-b".into(),
            provider_id: "provider-b".into(),
            global_model_id: "model-a".into(),
            provider_model_name: "provider-b-model".into(),
            provider_model_mapping: None,
            is_active: true,
        }],
        ..provider_with_endpoints_and_keys()
    }
}

pub(super) fn request() -> CandidateRequest<'static> {
    CandidateRequest {
        api_format: "openai:chat",
        model_name: "gpt-test",
        is_stream: false,
    }
}

pub(super) const fn minute_of_day(hour: u16, minute: u16) -> u16 {
    hour * 60 + minute
}

pub(super) fn api_token(token_type: ApiTokenType, user_id: Option<&str>) -> ApiToken {
    ApiToken {
        id: "token-a".into(),
        user_id: user_id.map(str::to_owned),
        token_type,
        name: "Token A".into(),
        token_value: String::new(),
        token_hash: String::new(),
        token_prefix: "sk-test".into(),
        group_code: "default".into(),
        expires_at: None,
        model_access_mode: ModelAccessMode::All,
        allowed_model_ids: Vec::new(),
        rate_limit_rpm: None,
        quota_limit: None,
        used_quota: Decimal::ZERO,
        request_count: 0,
        is_active: true,
        last_used_at: None,
        created_at: String::new(),
        updated_at: String::new(),
    }
}

pub(super) fn endpoint(id: &str, api_format: &str) -> CachedEndpoint {
    CachedEndpoint {
        id: id.into(),
        provider_id: "provider-a".into(),
        api_format: api_format.into(),
        base_url: "https://example.com".into(),
        custom_path: None,
        max_retries: None,
        is_active: true,
        format_acceptance_config: Some(serde_json::json!({ "enabled": true })),
        header_rules: None,
        body_rules: None,
    }
}

fn key(id: &str, internal_priority: i32) -> CachedProviderKey {
    CachedProviderKey {
        id: id.into(),
        provider_id: "provider-a".into(),
        name: format!("{id}-name"),
        api_formats: vec![
            "openai:chat".into(),
            "gemini:chat".into(),
            "openai_image".into(),
            "openai_image_edit".into(),
            "openai:compact".into(),
        ],
        allowed_model_ids: Vec::new(),
        key_preview: format!("{id}-name"),
        encrypted_api_key: "encrypted".into(),
        internal_priority,
        rpm_limit: None,
        cache_ttl_minutes: 5,
        time_range_enabled: false,
        time_range_start_minute: None,
        time_range_end_minute: None,
        is_active: true,
    }
}

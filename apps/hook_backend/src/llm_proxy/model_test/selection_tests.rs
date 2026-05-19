use types::{model::TieredPricingConfig, provider::ProviderSchedulingMode, system_setting::RequestRecordLevel};

use super::*;

#[test]
fn fixed_parts_uses_selected_endpoint_as_client_format_and_routes_to_compatible_openai_key() {
    let snapshot = snapshot(provider(vec![
        endpoint("endpoint-gemini", "gemini_cli"),
        endpoint("endpoint-openai", "openai_chat"),
    ]));

    let parts = fixed_parts(&snapshot, "provider-a", "binding-a", "endpoint-gemini", true).unwrap();

    assert_eq!(parts.client_api_format, "gemini_cli");
    assert_eq!(parts.endpoints.len(), 1);
    assert_eq!(parts.endpoints[0].id, "endpoint-openai");
    assert_eq!(parts.keys.len(), 1);
    assert_eq!(parts.keys[0].id, "key-openai");
    assert!(parts.effective_stream);
}

#[test]
fn fixed_parts_excludes_compact_endpoint_from_stream_responses_test_route() {
    let snapshot = snapshot(provider_with_keys_and_endpoints(
        vec![endpoint("endpoint-responses", "openai_cli"), endpoint("endpoint-compact", "openai_compact")],
        vec![key("key-responses", vec!["openai_cli"]), key("key-compact", vec!["openai_compact"])],
    ));

    let parts = fixed_parts(&snapshot, "provider-a", "binding-a", "endpoint-responses", true).unwrap();

    assert!(parts.effective_stream);
    assert_eq!(parts.endpoints.len(), 1);
    assert_eq!(parts.endpoints[0].api_format, "openai_cli");
    assert_eq!(parts.keys.len(), 1);
    assert_eq!(parts.keys[0].id, "key-responses");
}

fn snapshot(provider: CachedProvider) -> SchedulingSnapshot {
    SchedulingSnapshot {
        default_rate_limit_rpm: 0,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        client_request_record_level: RequestRecordLevel::Basic,
        client_max_request_body_size_kb: 1024,
        client_max_response_body_size_kb: 1024,
        client_sensitive_request_headers: String::new(),
        provider_request_record_level: RequestRecordLevel::Basic,
        provider_max_request_body_size_kb: 1024,
        provider_max_response_body_size_kb: 1024,
        provider_sensitive_request_headers: String::new(),
        provider_cooldown_policy: Default::default(),
        models: vec![CachedGlobalModel {
            id: "global-model-a".into(),
            name: "gpt-test".into(),
            is_active: true,
            default_price_per_request: None,
            default_tiered_pricing: TieredPricingConfig { tiers: Vec::new() },
        }],
        groups: Vec::new(),
        users: Vec::new(),
        providers: vec![provider],
    }
}

fn provider(endpoints: Vec<CachedEndpoint>) -> CachedProvider {
    provider_with_keys_and_endpoints(endpoints, vec![key("key-openai", vec!["openai_chat"])])
}

fn provider_with_keys_and_endpoints(endpoints: Vec<CachedEndpoint>, keys: Vec<CachedProviderKey>) -> CachedProvider {
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
        endpoints,
        keys,
        models: vec![CachedModelBinding {
            id: "binding-a".into(),
            provider_id: "provider-a".into(),
            global_model_id: "global-model-a".into(),
            provider_model_name: "upstream-model".into(),
            provider_model_mapping: None,
            is_active: true,
            price_per_request: None,
            tiered_pricing: None,
        }],
    }
}

fn endpoint(id: &str, api_format: &str) -> CachedEndpoint {
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

fn key(id: &str, api_formats: Vec<&str>) -> CachedProviderKey {
    CachedProviderKey {
        id: id.into(),
        provider_id: "provider-a".into(),
        name: format!("{id}-name"),
        api_formats: api_formats.into_iter().map(str::to_owned).collect(),
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
    }
}

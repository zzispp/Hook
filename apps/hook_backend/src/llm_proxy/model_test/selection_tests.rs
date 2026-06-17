use types::{model::TieredPricingConfig, provider::ProviderSchedulingMode, system_setting::RequestRecordLevel};

use super::*;

#[test]
fn fixed_parts_uses_selected_endpoint_as_client_format_and_routes_to_compatible_openai_key() {
    let snapshot = snapshot(provider(vec![
        endpoint("endpoint-gemini", "gemini:cli"),
        endpoint("endpoint-openai", "openai:cli"),
    ]));

    let parts = fixed_parts(&snapshot, "provider-a", "binding-a", "endpoint-gemini", "key-openai", true).unwrap();

    assert_eq!(parts.client_api_format, "gemini:cli");
    assert_eq!(parts.endpoints.len(), 1);
    assert_eq!(parts.endpoints[0].id, "endpoint-openai");
    assert_eq!(parts.keys.len(), 1);
    assert_eq!(parts.keys[0].id, "key-openai");
    assert!(parts.effective_stream);
}

#[test]
fn fixed_parts_excludes_compact_endpoint_from_stream_responses_test_route() {
    let snapshot = snapshot(provider_with_keys_and_endpoints(
        vec![endpoint("endpoint-responses", "openai:cli"), endpoint("endpoint-compact", "openai:compact")],
        vec![key("key-responses", vec!["openai:cli"]), key("key-compact", vec!["openai:compact"])],
    ));

    let parts = fixed_parts(&snapshot, "provider-a", "binding-a", "endpoint-responses", "key-responses", true).unwrap();

    assert!(parts.effective_stream);
    assert_eq!(parts.endpoints.len(), 1);
    assert_eq!(parts.endpoints[0].api_format, "openai:cli");
    assert_eq!(parts.keys.len(), 1);
    assert_eq!(parts.keys[0].id, "key-responses");
}

#[test]
fn fixed_parts_supports_openai_image_edit_provider_tests() {
    let snapshot = snapshot(provider_with_keys_and_endpoints(
        vec![endpoint("endpoint-image-edit", "openai_image_edit")],
        vec![key("key-image-edit", vec!["openai_image_edit"])],
    ));

    let parts = fixed_parts(&snapshot, "provider-a", "binding-a", "endpoint-image-edit", "key-image-edit", false).unwrap();

    assert_eq!(parts.client_api_format, "openai_image_edit");
    assert_eq!(parts.endpoints.len(), 1);
    assert_eq!(parts.endpoints[0].api_format, "openai_image_edit");
    assert_eq!(parts.keys.len(), 1);
    assert_eq!(parts.keys[0].id, "key-image-edit");
}

#[test]
fn fixed_parts_uses_selected_key_for_compatible_test_route() {
    let snapshot = snapshot(provider_with_keys_and_endpoints(
        vec![endpoint("endpoint-openai", "openai:cli")],
        vec![key("key-a", vec!["openai:cli"]), key("key-b", vec!["openai:cli"])],
    ));

    let parts = fixed_parts(&snapshot, "provider-a", "binding-a", "endpoint-openai", "key-b", true).unwrap();

    assert_eq!(parts.keys.len(), 1);
    assert_eq!(parts.keys[0].id, "key-b");
}

#[test]
fn fixed_parts_allows_inactive_provider_for_manual_test() {
    let snapshot = snapshot(CachedProvider {
        is_active: false,
        ..provider_with_keys_and_endpoints(vec![endpoint("endpoint-openai", "openai:cli")], vec![key("key-openai", vec!["openai:cli"])])
    });

    let parts = fixed_parts(&snapshot, "provider-a", "binding-a", "endpoint-openai", "key-openai", true).unwrap();

    assert_eq!(parts.provider.id, "provider-a");
    assert!(!parts.provider.is_active);
    assert_eq!(parts.endpoints.len(), 1);
    assert_eq!(parts.keys[0].id, "key-openai");
}

#[test]
fn fixed_parts_rejects_selected_key_that_does_not_support_test_route() {
    let snapshot = snapshot(provider_with_keys_and_endpoints(
        vec![endpoint("endpoint-openai", "openai:cli")],
        vec![key("key-claude", vec!["claude:chat"])],
    ));

    let error = fixed_parts_error(&snapshot, "key-claude");

    assert!(
        error
            .to_string()
            .contains("selected API key does not support a compatible test endpoint format")
    );
}

#[test]
fn fixed_parts_rejects_inactive_selected_key() {
    let snapshot = snapshot(provider_with_keys_and_endpoints(
        vec![endpoint("endpoint-openai", "openai:cli")],
        vec![inactive_key("key-inactive", vec!["openai:cli"])],
    ));

    let error = fixed_parts_error(&snapshot, "key-inactive");

    assert!(error.to_string().contains("selected API key is inactive"));
}

#[test]
fn fixed_parts_rejects_selected_key_that_does_not_allow_model() {
    let snapshot = snapshot(provider_with_keys_and_endpoints(
        vec![endpoint("endpoint-openai", "openai:cli")],
        vec![model_scoped_key("key-scoped", vec!["openai:cli"], vec!["other-model"])],
    ));

    let error = fixed_parts_error(&snapshot, "key-scoped");

    assert!(error.to_string().contains("selected API key does not allow this model"));
}

#[test]
fn fixed_parts_rejects_selected_key_outside_time_range() {
    let snapshot = snapshot(provider_with_keys_and_endpoints(
        vec![endpoint("endpoint-openai", "openai:cli")],
        vec![out_of_range_key("key-window", vec!["openai:cli"])],
    ));

    let error = fixed_parts_error(&snapshot, "key-window");

    assert!(error.to_string().contains("selected API key is outside its active time range"));
}

fn snapshot(provider: CachedProvider) -> SchedulingSnapshot {
    SchedulingSnapshot {
        default_rate_limit_rpm: 0,
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
        models: vec![CachedGlobalModel {
            id: "global-model-a".into(),
            name: "gpt-test".into(),
            is_active: true,
            supported_capabilities: None,
            default_price_per_request: None,
            default_tiered_pricing: TieredPricingConfig { tiers: Vec::new() },
            routing_profile_id: None,
        }],
        groups: Vec::new(),
        active_user_group_codes: Vec::new(),
        users: Vec::new(),
        providers: vec![provider],
    }
}

fn provider(endpoints: Vec<CachedEndpoint>) -> CachedProvider {
    provider_with_keys_and_endpoints(endpoints, vec![key("key-openai", vec!["openai:cli"])])
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
        }],
    }
}

fn fixed_parts_error(snapshot: &SchedulingSnapshot, key_id: &str) -> LlmProxyError {
    match fixed_parts(snapshot, "provider-a", "binding-a", "endpoint-openai", key_id, true) {
        Ok(_) => panic!("fixed_parts should reject key {key_id}"),
        Err(error) => error,
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
        supports_image_generation: false,
        key_preview: "sk-***".into(),
        encrypted_api_key: "encrypted".into(),
        internal_priority: 0,
        global_priority_by_format: std::collections::BTreeMap::from([("openai:chat".to_owned(), 0)]),
        rpm_limit: None,
        cache_ttl_minutes: 0,
        time_range_enabled: false,
        time_range_start_minute: None,
        time_range_end_minute: None,
        is_active: true,
    }
}

fn inactive_key(id: &str, api_formats: Vec<&str>) -> CachedProviderKey {
    CachedProviderKey {
        is_active: false,
        ..key(id, api_formats)
    }
}

fn model_scoped_key(id: &str, api_formats: Vec<&str>, model_ids: Vec<&str>) -> CachedProviderKey {
    CachedProviderKey {
        allowed_model_ids: model_ids.into_iter().map(str::to_owned).collect(),
        ..key(id, api_formats)
    }
}

fn out_of_range_key(id: &str, api_formats: Vec<&str>) -> CachedProviderKey {
    let current_minute = current_utc_minute();
    let start = (current_minute + 1) % 1440;
    let end = (current_minute + 2) % 1440;
    CachedProviderKey {
        time_range_enabled: true,
        time_range_start_minute: Some(start),
        time_range_end_minute: Some(end),
        ..key(id, api_formats)
    }
}

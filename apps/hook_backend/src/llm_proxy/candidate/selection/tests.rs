use rust_decimal::Decimal;
use types::{model::TieredPricingConfig, provider::ProviderSchedulingMode};

use super::*;

#[test]
fn matching_candidate_parts_compacts_endpoint_key_product_into_provider_route() {
    let snapshot = snapshot_with_provider(provider_with_endpoints_and_keys());
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(
        &snapshot,
        group,
        None,
        "model-a",
        request(),
        None,
        ProviderSchedulingMode::FixedOrder,
        "request-1",
    );

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].provider.id, "provider-a");
    assert_eq!(parts[0].endpoints.len(), 3);
    assert_eq!(parts[0].keys.len(), 2);
    assert_eq!(parts[0].endpoints[0].api_format, "openai_chat");
    assert_eq!(parts[0].endpoints[1].api_format, "gemini_chat");
    assert_eq!(parts[0].keys[0].id, "key-a-1");
}

#[test]
fn matching_candidate_parts_promotes_affinity_key_inside_route() {
    let snapshot = snapshot_with_provider(provider_with_endpoints_and_keys());
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(
        &snapshot,
        group,
        None,
        "model-a",
        request(),
        Some("key-a-2"),
        ProviderSchedulingMode::CacheAffinity,
        "request-1",
    );

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].keys[0].id, "key-a-2");
    assert_eq!(parts[0].keys[1].id, "key-a-1");
}

#[test]
fn matching_candidate_parts_keeps_all_provider_routes_without_silent_budget() {
    let snapshot = SchedulingSnapshot {
        providers: vec![provider_with_endpoints_and_keys(), provider_b()],
        ..snapshot_with_provider(provider_with_endpoints_and_keys())
    };
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(
        &snapshot,
        group,
        None,
        "model-a",
        request(),
        None,
        ProviderSchedulingMode::FixedOrder,
        "request-1",
    );

    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].provider.id, "provider-a");
    assert_eq!(parts[1].provider.id, "provider-b");
}

#[test]
fn matching_candidate_parts_filters_by_user_provider_access() {
    let snapshot = SchedulingSnapshot {
        providers: vec![provider_with_endpoints_and_keys(), provider_b()],
        ..snapshot_with_provider(provider_with_endpoints_and_keys())
    };
    let group = &snapshot.groups[0];
    let user_access = CachedUserAccess {
        id: "user-a".into(),
        allowed_model_ids: Vec::new(),
        allowed_provider_ids: vec!["provider-b".into()],
    };

    let parts = matching_candidate_parts(
        &snapshot,
        group,
        Some(&user_access),
        "model-a",
        request(),
        None,
        ProviderSchedulingMode::FixedOrder,
        "request-1",
    );

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].provider.id, "provider-b");
}

fn snapshot_with_provider(provider: CachedProvider) -> SchedulingSnapshot {
    SchedulingSnapshot {
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
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

fn provider_with_endpoints_and_keys() -> CachedProvider {
    CachedProvider {
        id: "provider-a".into(),
        name: "Provider A".into(),
        max_retries: Some(2),
        request_timeout_seconds: None,
        stream_first_byte_timeout_seconds: None,
        priority: 10,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active: true,
        endpoints: vec![
            endpoint("endpoint-gemini", "gemini_chat"),
            endpoint("endpoint-openai", "openai_chat"),
            endpoint("endpoint-compact", "openai_compact"),
        ],
        keys: vec![key("key-a-2", 20), key("key-a-1", 10)],
        models: vec![CachedModelBinding {
            id: "binding-a".into(),
            provider_id: "provider-a".into(),
            global_model_id: "model-a".into(),
            provider_model_name: "upstream-model".into(),
            is_active: true,
            price_per_request: None,
            tiered_pricing: None,
        }],
    }
}

fn provider_b() -> CachedProvider {
    CachedProvider {
        id: "provider-b".into(),
        name: "Provider B".into(),
        priority: 20,
        endpoints: vec![CachedEndpoint {
            provider_id: "provider-b".into(),
            ..endpoint("endpoint-b-openai", "openai_chat")
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
            is_active: true,
            price_per_request: None,
            tiered_pricing: None,
        }],
        ..provider_with_endpoints_and_keys()
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
    }
}

fn key(id: &str, internal_priority: i32) -> CachedProviderKey {
    CachedProviderKey {
        id: id.into(),
        provider_id: "provider-a".into(),
        encrypted_api_key: "encrypted".into(),
        internal_priority,
        cache_ttl_minutes: 5,
        is_active: true,
    }
}

fn request() -> CandidateRequest<'static> {
    CandidateRequest {
        api_format: "openai_chat",
        model_name: "gpt-test",
        is_stream: false,
    }
}

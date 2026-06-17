use sea_orm::{DatabaseBackend, MockDatabase};
use storage::{
    Database,
    model::provider_models,
    provider::{
        record::provider_key_model_mappings,
        record::{provider_api_keys, provider_endpoints, providers},
    },
};

use super::load_providers;

#[tokio::test]
async fn load_providers_keeps_disabled_providers_for_admin_model_tests() {
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([provider_records()])
            .append_query_results([key_records("provider-active")])
            .append_query_results([key_model_mapping_records("provider-active")])
            .append_query_results([model_records("provider-active")])
            .append_query_results([endpoint_records("provider-active")])
            .append_query_results([model_records("provider-active")])
            .append_query_results([key_records("provider-disabled")])
            .append_query_results([key_model_mapping_records("provider-disabled")])
            .append_query_results([Vec::<provider_models::Model>::new()])
            .append_query_results([endpoint_records("provider-disabled")])
            .append_query_results([model_records("provider-disabled")])
            .into_connection(),
    );

    let providers = load_providers(&database).await.unwrap();

    assert_eq!(
        providers.iter().map(|provider| provider.id.as_str()).collect::<Vec<_>>(),
        vec!["provider-active", "provider-disabled"]
    );
    assert!(providers[0].is_active);
    assert!(!providers[1].is_active);
    assert_eq!(providers[1].endpoints[0].provider_id, "provider-disabled");
    assert_eq!(providers[1].keys[0].provider_id, "provider-disabled");
    assert!(providers[1].keys[0].model_mappings.is_empty());
    assert_eq!(providers[1].models[0].provider_id, "provider-disabled");
}

fn provider_records() -> Vec<providers::Model> {
    vec![
        provider_record("provider-active", "Active Provider", true, 1),
        provider_record("provider-disabled", "Disabled Provider", false, 2),
    ]
}

fn provider_record(id: &str, name: &str, is_active: bool, priority: i32) -> providers::Model {
    providers::Model {
        id: id.into(),
        name: name.into(),
        provider_type: "custom".into(),
        provider_origin: "manual".into(),
        max_retries: Some(2),
        request_timeout_seconds: Some(300.0),
        stream_first_byte_timeout_seconds: Some(60.0),
        stream_idle_timeout_seconds: Some(30.0),
        priority,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active,
        created_at: now(),
        updated_at: now(),
    }
}

fn endpoint_records(provider_id: &str) -> Vec<provider_endpoints::Model> {
    vec![provider_endpoints::Model {
        id: format!("{provider_id}-endpoint"),
        provider_id: provider_id.into(),
        api_format: "openai:cli".into(),
        base_url: "https://example.test".into(),
        custom_path: None,
        max_retries: None,
        is_active: true,
        format_acceptance_config: None,
        header_rules: None,
        body_rules: None,
        created_at: now(),
        updated_at: now(),
    }]
}

fn key_records(provider_id: &str) -> Vec<provider_api_keys::Model> {
    vec![provider_api_keys::Model {
        id: format!("{provider_id}-key"),
        provider_id: provider_id.into(),
        name: "Test Key".into(),
        api_formats: "[\"openai:cli\"]".into(),
        allowed_model_ids: "[]".into(),
        capabilities: None,
        encrypted_api_key: "encrypted".into(),
        note: None,
        internal_priority: 0,
        global_priority_by_format: "{}".into(),
        rpm_limit: None,
        learned_rpm_limit: None,
        cache_ttl_minutes: 0,
        max_probe_interval_minutes: 30,
        time_range_enabled: false,
        time_range_start: None,
        time_range_end: None,
        health_by_format: None,
        circuit_breaker_by_format: None,
        is_active: true,
        created_at: now(),
        updated_at: now(),
    }]
}

fn model_records(provider_id: &str) -> Vec<provider_models::Model> {
    vec![provider_models::Model {
        id: format!("{provider_id}-model"),
        provider_id: provider_id.into(),
        global_model_id: "global-model".into(),
        is_active: true,
        config: None,
        created_at: now(),
        updated_at: now(),
    }]
}

fn key_model_mapping_records(provider_id: &str) -> Vec<provider_key_model_mappings::Model> {
    if provider_id != "provider-active" {
        return Vec::new();
    }
    vec![provider_key_model_mappings::Model {
        id: format!("{provider_id}-mapping"),
        provider_id: provider_id.into(),
        key_id: format!("{provider_id}-key"),
        provider_model_id: format!("{provider_id}-model"),
        upstream_model_name: "upstream-model".into(),
        reasoning_effort: Some("high".into()),
        created_at: now(),
        updated_at: now(),
    }]
}

fn now() -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::June, 8)
        .unwrap()
        .with_hms(12, 0, 0)
        .unwrap()
        .assume_utc()
}

use std::collections::BTreeMap;

use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, DbErr, MockDatabase};
use storage::{
    Database,
    model::provider_models,
    provider::{
        ProviderQuickImportApiKeyRecordInput, ProviderQuickImportEndpointRecordInput, ProviderQuickImportModelCostRecordInput,
        ProviderQuickImportModelRecordInput, ProviderQuickImportRecordInput, ProviderRecordInput, ProviderStore,
        record::{provider_api_keys, provider_endpoints, provider_model_costs, providers},
    },
};
use types::provider::{ProviderModelCostMode, ProviderModelMapping, ProviderOrigin};

#[tokio::test]
async fn create_quick_import_commits_complete_resource_set() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[provider_record()]])
        .append_query_results([[endpoint_record()]])
        .append_query_results([[model_record()]])
        .append_query_results([[key_record()]])
        .append_query_results([[cost_record()]])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let output = store.create_quick_import(quick_import_input()).await.unwrap();

    assert_eq!(output.provider.id, "provider-a");
    assert_eq!(output.provider.provider_origin, ProviderOrigin::QuickImport);
    assert_eq!(output.endpoints.len(), 1);
    assert_eq!(output.model_bindings.len(), 1);
    assert_eq!(output.api_keys.len(), 1);
    assert_eq!(output.model_costs.len(), 1);
    assert_committed_all_tables(sql_statements(connection));
}

#[tokio::test]
async fn create_quick_import_rolls_back_when_late_insert_fails() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[provider_record()]])
        .append_query_results([[endpoint_record()]])
        .append_query_results([[model_record()]])
        .append_query_results([[key_record()]])
        .append_query_errors([DbErr::Custom("cost insert failed".into())])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let error = store.create_quick_import(quick_import_input()).await.unwrap_err();

    assert!(error.to_string().contains("cost insert failed"));
    let statements = sql_statements(connection);
    assert_eq!(statements.iter().filter(|sql| sql.contains("BEGIN")).count(), 1);
    assert_eq!(statements.iter().filter(|sql| sql.contains("ROLLBACK")).count(), 1);
    assert_eq!(statements.iter().filter(|sql| sql.contains("COMMIT")).count(), 0);
    assert_inserted_tables(&statements);
}

fn quick_import_input() -> ProviderQuickImportRecordInput {
    ProviderQuickImportRecordInput {
        provider: provider_input(),
        endpoints: vec![endpoint_input()],
        api_keys: vec![key_input()],
        model_bindings: vec![model_input()],
        model_costs: vec![cost_input()],
    }
}

fn provider_input() -> ProviderRecordInput {
    ProviderRecordInput {
        name: "Provider A".into(),
        provider_type: "custom".into(),
        provider_origin: ProviderOrigin::Manual,
        provider_group_id: None,
        max_retries: Some(2),
        request_timeout_seconds: Some(300.0),
        stream_first_byte_timeout_seconds: Some(60.0),
        stream_idle_timeout_seconds: Some(300.0),
        priority: 100,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active: true,
    }
}

fn endpoint_input() -> ProviderQuickImportEndpointRecordInput {
    ProviderQuickImportEndpointRecordInput {
        api_format: "openai".into(),
        base_url: "https://newapi.example".into(),
        custom_path: None,
        max_retries: None,
        is_active: true,
        format_acceptance_config: None,
        header_rules: None,
        body_rules: None,
    }
}

fn model_input() -> ProviderQuickImportModelRecordInput {
    ProviderQuickImportModelRecordInput {
        global_model_id: "global-model-a".into(),
        provider_model_name: "gpt-5".into(),
        provider_model_mapping: Some(ProviderModelMapping {
            name: "upstream-gpt-5".into(),
            reasoning_effort: None,
        }),
        is_active: true,
        config: None,
    }
}

fn key_input() -> ProviderQuickImportApiKeyRecordInput {
    ProviderQuickImportApiKeyRecordInput {
        upstream_token_id: "1209".into(),
        name: "codex".into(),
        api_formats: vec!["openai".into()],
        allowed_model_ids: vec!["global-model-a".into()],
        encrypted_api_key: "encrypted".into(),
        note: Some("Imported from newapi group: plus".into()),
        internal_priority: 10,
        global_priority_by_format: BTreeMap::new(),
        rpm_limit: None,
        cache_ttl_minutes: 5,
        max_probe_interval_minutes: 32,
        time_range_enabled: false,
        time_range_start: None,
        time_range_end: None,
        is_active: true,
    }
}

fn cost_input() -> ProviderQuickImportModelCostRecordInput {
    ProviderQuickImportModelCostRecordInput {
        upstream_token_id: "1209".into(),
        global_model_id: "global-model-a".into(),
        cost_mode: ProviderModelCostMode::PerToken,
        price_per_request: None,
        input_price_per_million: Some(Decimal::new(1, 2)),
        output_price_per_million: Some(Decimal::new(2, 2)),
        cache_creation_price_per_million: Some(Decimal::new(125, 4)),
        cache_read_price_per_million: Some(Decimal::new(1, 3)),
    }
}

fn assert_committed_all_tables(statements: Vec<String>) {
    assert_eq!(statements.iter().filter(|sql| sql.contains("BEGIN")).count(), 1);
    assert_eq!(statements.iter().filter(|sql| sql.contains("COMMIT")).count(), 1);
    assert_eq!(statements.iter().filter(|sql| sql.contains("ROLLBACK")).count(), 0);
    assert_inserted_tables(&statements);
}

fn assert_inserted_tables(statements: &[String]) {
    for table in [
        "providers",
        "provider_endpoints",
        "provider_models",
        "provider_api_keys",
        "provider_model_costs",
    ] {
        assert!(statements.iter().any(|sql| sql.contains(&format!("INSERT INTO \"{table}\""))), "{statements:?}");
    }
}

fn provider_record() -> providers::Model {
    providers::Model {
        id: "provider-a".into(),
        name: "Provider A".into(),
        provider_type: "custom".into(),
        provider_origin: "quick_import".into(),
        max_retries: Some(2),
        request_timeout_seconds: Some(300.0),
        stream_first_byte_timeout_seconds: Some(60.0),
        stream_idle_timeout_seconds: Some(300.0),
        priority: 100,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active: true,
        created_at: now(),
        updated_at: now(),
    }
}

fn endpoint_record() -> provider_endpoints::Model {
    provider_endpoints::Model {
        id: "endpoint-a".into(),
        provider_id: "provider-a".into(),
        api_format: "openai".into(),
        base_url: "https://newapi.example".into(),
        custom_path: None,
        max_retries: None,
        is_active: true,
        format_acceptance_config: None,
        header_rules: None,
        body_rules: None,
        created_at: now(),
        updated_at: now(),
    }
}

fn model_record() -> provider_models::Model {
    provider_models::Model {
        id: "provider-model-a".into(),
        provider_id: "provider-a".into(),
        global_model_id: "global-model-a".into(),
        provider_model_name: "gpt-5".into(),
        provider_model_mappings: Some(r#"{"name":"upstream-gpt-5","reasoning_effort":null}"#.into()),
        is_active: true,
        config: None,
        created_at: now(),
        updated_at: now(),
    }
}

fn key_record() -> provider_api_keys::Model {
    provider_api_keys::Model {
        id: "key-a".into(),
        provider_id: "provider-a".into(),
        name: "codex".into(),
        api_formats: r#"["openai"]"#.into(),
        allowed_model_ids: r#"["global-model-a"]"#.into(),
        encrypted_api_key: "encrypted".into(),
        note: Some("Imported from newapi group: plus".into()),
        internal_priority: 10,
        global_priority_by_format: "{}".into(),
        rpm_limit: None,
        learned_rpm_limit: None,
        cache_ttl_minutes: 5,
        max_probe_interval_minutes: 32,
        time_range_enabled: false,
        time_range_start: None,
        time_range_end: None,
        health_by_format: None,
        circuit_breaker_by_format: None,
        is_active: true,
        created_at: now(),
        updated_at: now(),
    }
}

fn cost_record() -> provider_model_costs::Model {
    provider_model_costs::Model {
        id: "cost-a".into(),
        provider_id: "provider-a".into(),
        key_id: "key-a".into(),
        provider_model_id: "provider-model-a".into(),
        cost_mode: "per_token".into(),
        price_per_request: None,
        input_price_per_million: Some(Decimal::new(1, 2)),
        output_price_per_million: Some(Decimal::new(2, 2)),
        cache_creation_price_per_million: Some(Decimal::new(125, 4)),
        cache_read_price_per_million: Some(Decimal::new(1, 3)),
        created_at: now(),
        updated_at: now(),
    }
}

fn sql_statements(connection: sea_orm::DatabaseConnection) -> Vec<String> {
    connection
        .into_transaction_log()
        .into_iter()
        .flat_map(|transaction| transaction.statements().iter().map(|statement| statement.sql.clone()).collect::<Vec<_>>())
        .collect()
}

fn now() -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::May, 11)
        .unwrap()
        .with_hms(12, 0, 0)
        .unwrap()
        .assume_utc()
}

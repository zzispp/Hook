use std::collections::BTreeMap;

use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, DbErr, MockDatabase, MockExecResult};
use storage::{
    Database,
    model::provider_models,
    provider::{
        ProviderQuickImportApiKeyRecordInput, ProviderQuickImportBindRecordInput, ProviderQuickImportBoundApiKeyRecordInput,
        ProviderQuickImportEndpointRecordInput, ProviderQuickImportKeyModelRecordInput, ProviderQuickImportModelCostRecordInput,
        ProviderQuickImportModelRecordInput, ProviderQuickImportRecordInput, ProviderQuickImportSourceRecordInput, ProviderRecordInput, ProviderStore,
        record::{
            provider_api_keys, provider_endpoints, provider_key_model_mappings, provider_model_costs, provider_quick_import_keys,
            provider_quick_import_sources, providers,
        },
    },
};
use types::provider::{ProviderModelCostMode, ProviderOrigin, ProviderQuickImportSyncConfig};

#[tokio::test]
async fn create_quick_import_commits_complete_resource_set() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[provider_record()]])
        .append_query_results([[endpoint_record()]])
        .append_query_results([[model_record()]])
        .append_query_results([[key_record()]])
        .append_query_results([[cost_record()]])
        .append_query_results([[sync_source_record()]])
        .append_query_results([[sync_key_record()]])
        .append_query_results([[sync_key_model_record()]])
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
    assert_core_inserted_tables(&statements);
}

#[tokio::test]
async fn bind_quick_import_converts_provider_and_rebuilds_resources() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[manual_provider_record()]])
        .append_query_results([vec![existing_key_record("key-a"), existing_key_record("key-b")]])
        .append_query_results([vec![existing_key_record("key-a"), existing_key_record("key-b")]])
        .append_exec_results(exec_results(7))
        .append_query_results([[endpoint_record()]])
        .append_query_results([[model_record()]])
        .append_query_results([[existing_key_record("key-a")]])
        .append_query_results([[updated_key_record()]])
        .append_query_results([[cost_record()]])
        .append_query_results([[sync_source_record()]])
        .append_query_results([[sync_key_record()]])
        .append_query_results([[sync_key_model_record()]])
        .append_query_results([[quick_import_provider_record()]])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let output = store.bind_quick_import(bind_input()).await.unwrap();

    assert_eq!(output.provider.provider_origin, ProviderOrigin::QuickImport);
    assert_eq!(output.created_key_count, 0);
    assert_eq!(output.reused_key_count, 1);
    assert_eq!(output.deleted_key_count, 1);
    assert_eq!(output.api_keys[0].id, "key-a");
    assert_eq!(output.api_keys[0].name, "codex");
    assert_eq!(output.api_keys[0].allowed_model_ids, vec!["global-model-a"]);
    assert_bind_rebuilt_tables(sql_statements(connection));
}

#[tokio::test]
async fn bind_quick_import_rolls_back_when_late_insert_fails() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[manual_provider_record()]])
        .append_query_results([vec![existing_key_record("key-a"), existing_key_record("key-b")]])
        .append_query_results([vec![existing_key_record("key-a"), existing_key_record("key-b")]])
        .append_exec_results(exec_results(7))
        .append_query_results([[endpoint_record()]])
        .append_query_results([[model_record()]])
        .append_query_results([[existing_key_record("key-a")]])
        .append_query_results([[updated_key_record()]])
        .append_query_errors([DbErr::Custom("cost insert failed".into())])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let error = store.bind_quick_import(bind_input()).await.unwrap_err();

    assert!(error.to_string().contains("cost insert failed"));
    let statements = sql_statements(connection);
    assert_eq!(statements.iter().filter(|sql| sql.contains("BEGIN")).count(), 1);
    assert_eq!(statements.iter().filter(|sql| sql.contains("ROLLBACK")).count(), 1);
    assert_eq!(statements.iter().filter(|sql| sql.contains("COMMIT")).count(), 0);
    assert!(statements.iter().any(|sql| sql.contains("DELETE FROM \"provider_api_keys\"")));
    assert!(statements.iter().any(|sql| sql.contains("UPDATE \"provider_api_keys\"")));
}

fn quick_import_input() -> ProviderQuickImportRecordInput {
    ProviderQuickImportRecordInput {
        provider: provider_input(),
        sync_source: Some(sync_source_input()),
        endpoints: vec![endpoint_input()],
        api_keys: vec![key_input()],
        model_bindings: vec![model_input()],
        model_costs: vec![cost_input()],
    }
}

fn bind_input() -> ProviderQuickImportBindRecordInput {
    ProviderQuickImportBindRecordInput {
        provider_id: "provider-a".into(),
        sync_source: sync_source_input(),
        endpoints: vec![endpoint_input()],
        api_keys: vec![ProviderQuickImportBoundApiKeyRecordInput {
            local_key_id: Some("key-a".into()),
            input: key_input(),
        }],
        model_bindings: vec![model_input()],
        model_costs: vec![cost_input()],
    }
}

fn provider_input() -> ProviderRecordInput {
    ProviderRecordInput {
        name: "Provider A".into(),
        provider_type: "custom".into(),
        provider_origin: ProviderOrigin::Manual,
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
        is_active: true,
        config: None,
    }
}

fn key_input() -> ProviderQuickImportApiKeyRecordInput {
    ProviderQuickImportApiKeyRecordInput {
        upstream_token_id: "1209".into(),
        upstream_token_name: "codex".into(),
        upstream_masked_key: "c7mE****9pAG".into(),
        upstream_group: Some("plus".into()),
        upstream_group_ratio: Decimal::new(2, 0),
        effective_cost_multiplier: Decimal::new(2, 1),
        model_mappings: vec![ProviderQuickImportKeyModelRecordInput {
            upstream_model_name: "upstream-gpt-5".into(),
            global_model_id: "global-model-a".into(),
            reasoning_effort: None,
        }],
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

fn sync_source_input() -> ProviderQuickImportSourceRecordInput {
    ProviderQuickImportSourceRecordInput {
        source_kind: "newapi".into(),
        base_url: "https://newapi.example".into(),
        encrypted_system_access_token: "encrypted-system-token".into(),
        email: String::new(),
        encrypted_password: String::new(),
        encrypted_auth_token: String::new(),
        encrypted_refresh_token: String::new(),
        token_expires_at: None,
        user_id: "737".into(),
        recharge_multiplier: Decimal::new(10, 0),
        sync_config: ProviderQuickImportSyncConfig::default(),
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
    assert_core_inserted_tables(&statements);
    assert_sync_metadata_inserted_tables(&statements);
}

fn assert_core_inserted_tables(statements: &[String]) {
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

fn assert_sync_metadata_inserted_tables(statements: &[String]) {
    for table in ["provider_quick_import_sources", "provider_quick_import_keys", "provider_key_model_mappings"] {
        assert!(statements.iter().any(|sql| sql.contains(&format!("INSERT INTO \"{table}\""))), "{statements:?}");
    }
}

fn assert_bind_rebuilt_tables(statements: Vec<String>) {
    assert_eq!(statements.iter().filter(|sql| sql.contains("BEGIN")).count(), 1);
    assert_eq!(statements.iter().filter(|sql| sql.contains("COMMIT")).count(), 1);
    assert_eq!(statements.iter().filter(|sql| sql.contains("ROLLBACK")).count(), 0);
    for table in [
        "provider_key_model_mappings",
        "provider_quick_import_keys",
        "provider_quick_import_sources",
        "provider_model_costs",
        "provider_models",
        "provider_endpoints",
    ] {
        assert!(statements.iter().any(|sql| sql.contains(&format!("DELETE FROM \"{table}\""))), "{statements:?}");
    }
    assert!(statements.iter().any(|sql| sql.contains("DELETE FROM \"provider_api_keys\"")), "{statements:?}");
    assert!(statements.iter().any(|sql| sql.contains("UPDATE \"provider_api_keys\"")), "{statements:?}");
    assert!(statements.iter().any(|sql| sql.contains("UPDATE \"providers\"")), "{statements:?}");
    assert_core_inserted_tables_without_provider(&statements);
    assert_sync_metadata_inserted_tables(&statements);
}

fn assert_core_inserted_tables_without_provider(statements: &[String]) {
    for table in ["provider_endpoints", "provider_models", "provider_model_costs"] {
        assert!(statements.iter().any(|sql| sql.contains(&format!("INSERT INTO \"{table}\""))), "{statements:?}");
    }
}

fn manual_provider_record() -> providers::Model {
    providers::Model {
        provider_origin: "manual".into(),
        ..provider_record()
    }
}

fn quick_import_provider_record() -> providers::Model {
    providers::Model {
        provider_origin: "quick_import".into(),
        ..provider_record()
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

fn existing_key_record(id: &str) -> provider_api_keys::Model {
    provider_api_keys::Model {
        id: id.into(),
        name: format!("existing-{id}"),
        encrypted_api_key: "old-encrypted".into(),
        ..key_record()
    }
}

fn updated_key_record() -> provider_api_keys::Model {
    provider_api_keys::Model {
        id: "key-a".into(),
        encrypted_api_key: "encrypted".into(),
        ..key_record()
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

fn sync_source_record() -> provider_quick_import_sources::Model {
    provider_quick_import_sources::Model {
        id: "source-a".into(),
        provider_id: "provider-a".into(),
        source_kind: "newapi".into(),
        base_url: "https://newapi.example".into(),
        encrypted_system_access_token: "encrypted-system-token".into(),
        email: String::new(),
        encrypted_password: String::new(),
        encrypted_auth_token: String::new(),
        encrypted_refresh_token: String::new(),
        token_expires_at: None,
        user_id: "737".into(),
        recharge_multiplier: Decimal::new(10, 0),
        auto_sync_enabled: true,
        cost_sync_mode: "overwrite".into(),
        upstream_anomaly_action: "disable_key".into(),
        token_deleted_action: "disable_key".into(),
        token_disabled_action: "disable_key".into(),
        group_removed_action: "disable_key".into(),
        group_changed_action: "disable_key".into(),
        key_unavailable_action: "disable_key".into(),
        model_removed_action: "disable_key".into(),
        fetch_failure_action: "report_only".into(),
        fetch_failure_disable_threshold: 3,
        last_status: None,
        last_error: None,
        last_synced_at: None,
        consecutive_failures: 0,
        created_at: now(),
        updated_at: now(),
    }
}

fn sync_key_record() -> provider_quick_import_keys::Model {
    provider_quick_import_keys::Model {
        id: "sync-key-a".into(),
        provider_id: "provider-a".into(),
        source_id: "source-a".into(),
        key_id: "key-a".into(),
        upstream_token_id: "1209".into(),
        upstream_token_name: "codex".into(),
        upstream_masked_key: "c7mE****9pAG".into(),
        upstream_group: Some("plus".into()),
        upstream_group_ratio: Decimal::new(2, 0),
        effective_cost_multiplier: Decimal::new(2, 1),
        sync_statuses: r#"["ok"]"#.into(),
        last_sync_error: None,
        last_synced_at: None,
        created_at: now(),
        updated_at: now(),
    }
}

fn sync_key_model_record() -> provider_key_model_mappings::Model {
    provider_key_model_mappings::Model {
        id: "key-model-mapping-a".into(),
        provider_id: "provider-a".into(),
        key_id: "key-a".into(),
        provider_model_id: "provider-model-a".into(),
        upstream_model_name: "upstream-gpt-5".into(),
        reasoning_effort: None,
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

fn exec_results(count: usize) -> Vec<MockExecResult> {
    vec![exec_result(); count]
}

fn exec_result() -> MockExecResult {
    MockExecResult {
        last_insert_id: 0,
        rows_affected: 1,
    }
}

fn now() -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::May, 11)
        .unwrap()
        .with_hms(12, 0, 0)
        .unwrap()
        .assume_utc()
}

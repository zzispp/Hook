use sea_orm::{DatabaseBackend, MockDatabase};

use crate::{
    Database, StorageError,
    provider::{
        ProviderStore,
        quick_import_sync_lookup::{require_key_name, require_provider_name},
        quick_import_sync_query::list_sub2api_token_refresh_sources,
        record::{provider_quick_import_sources, providers},
    },
};

#[test]
fn missing_provider_name_is_explicit() {
    let error = require_provider_name(&std::collections::BTreeMap::new(), "provider-1").unwrap_err();
    assert_database_error(error, "quick import sync source provider missing: provider-1");
}

#[test]
fn missing_key_name_is_explicit() {
    let error = require_key_name(&std::collections::BTreeMap::new(), "provider-1", "key-1").unwrap_err();
    assert_database_error(error, "quick import sync local api key missing: provider-1/key-1");
}

#[tokio::test]
async fn list_sub2api_token_refresh_sources_filters_and_orders_candidates() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![
            token_source_record("provider-2", expires_after_minutes(10)),
            token_source_record("provider-1", expires_after_minutes(30)),
        ]])
        .append_query_results([vec![provider_record("provider-1"), provider_record("provider-2")]])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let records = list_sub2api_token_refresh_sources(&store, 20).await.unwrap();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].provider_id, "provider-2");
    assert_eq!(records[1].provider_id, "provider-1");
    let sql = logged_sql(connection);
    assert!(sql.contains("\"provider_quick_import_sources\".\"source_kind\" = $1"), "{sql}");
    assert!(sql.contains("\"provider_quick_import_sources\".\"encrypted_auth_token\" <> $2"), "{sql}");
    assert!(sql.contains("\"provider_quick_import_sources\".\"encrypted_refresh_token\" <> $3"), "{sql}");
    assert!(sql.contains("\"token_expires_at\" IS NOT NULL"), "{sql}");
    assert!(sql.contains("ORDER BY \"provider_quick_import_sources\".\"token_expires_at\" ASC"), "{sql}");
}

fn assert_database_error(error: StorageError, expected: &str) {
    match error {
        StorageError::Database(message) => assert_eq!(message, expected),
        other => panic!("expected database error, got {other:?}"),
    }
}

fn token_source_record(provider_id: &str, token_expires_at: time::OffsetDateTime) -> provider_quick_import_sources::Model {
    provider_quick_import_sources::Model {
        id: format!("source-{provider_id}"),
        provider_id: provider_id.into(),
        source_kind: "sub2api".into(),
        base_url: "https://sub2api.example".into(),
        encrypted_system_access_token: String::new(),
        email: String::new(),
        encrypted_password: String::new(),
        encrypted_auth_token: "enc:access".into(),
        encrypted_refresh_token: "enc:refresh".into(),
        token_expires_at: Some(token_expires_at),
        user_id: String::new(),
        recharge_multiplier: rust_decimal::Decimal::ONE,
        auto_sync_enabled: false,
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
        created_at: fixed_now(),
        updated_at: fixed_now(),
    }
}

fn provider_record(id: &str) -> providers::Model {
    providers::Model {
        id: id.into(),
        name: format!("Provider {id}"),
        provider_type: "custom".into(),
        provider_origin: "quick_import".into(),
        max_retries: Some(2),
        request_timeout_seconds: Some(300.0),
        stream_first_byte_timeout_seconds: Some(60.0),
        stream_first_output_timeout_seconds: Some(45.0),
        stream_idle_timeout_seconds: Some(300.0),
        priority: 100,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active: true,
        created_at: fixed_now(),
        updated_at: fixed_now(),
    }
}

fn logged_sql(connection: sea_orm::DatabaseConnection) -> String {
    connection
        .into_transaction_log()
        .iter()
        .flat_map(|entry| entry.statements())
        .map(|statement| statement.sql.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

fn expires_after_minutes(minutes: i64) -> time::OffsetDateTime {
    fixed_now() + time::Duration::minutes(minutes)
}

fn fixed_now() -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::June, 23)
        .unwrap()
        .with_hms(12, 0, 0)
        .unwrap()
        .assume_utc()
}

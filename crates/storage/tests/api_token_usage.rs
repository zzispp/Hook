use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
use storage::{
    Database, StorageError,
    api_token::{ApiTokenStore, ApiTokenUsageRecord},
};

#[tokio::test]
async fn api_token_usage_record_increments_metrics() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let store = ApiTokenStore::new(Database::new(connection.clone()));

    store.record_usage(usage_record()).await.unwrap();

    let logs = connection.into_transaction_log();
    assert_eq!(logs.len(), 1);
    let sql = &logs[0].statements()[0].sql;
    assert!(sql.contains("UPDATE \"api_tokens\" SET"), "{sql}");
    assert!(sql.contains("\"used_quota\" = \"used_quota\" +"), "{sql}");
    assert!(sql.contains("\"request_count\" = \"request_count\" + $"), "{sql}");
    assert!(sql.contains("\"last_used_at\" ="), "{sql}");
    assert!(sql.contains("WHERE \"api_tokens\".\"id\" ="), "{sql}");
}

#[tokio::test]
async fn api_token_usage_record_requires_existing_token() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 0,
        }])
        .into_connection();
    let store = ApiTokenStore::new(Database::new(connection));

    let error = store.record_usage(usage_record()).await.unwrap_err();

    assert!(matches!(error, StorageError::NotFound));
}

fn usage_record() -> ApiTokenUsageRecord {
    ApiTokenUsageRecord {
        token_id: "token-1".into(),
        cost: Decimal::new(25, 4),
        used_at: used_at(),
    }
}

fn used_at() -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::May, 12)
        .unwrap()
        .with_hms(10, 30, 0)
        .unwrap()
        .assume_utc()
}

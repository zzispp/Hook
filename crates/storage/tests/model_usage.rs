use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
use storage::{
    Database, StorageError,
    model::{GlobalModelUsageRecord, ModelStore},
    usage_flush::usage_flush_batches,
};

#[tokio::test]
async fn global_model_usage_record_increments_usage_count() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let store = ModelStore::new(Database::new(connection.clone()));

    store.record_usage(usage_record()).await.unwrap();

    let logs = connection.into_transaction_log();
    assert_eq!(logs.len(), 1);
    let sql = &logs[0].statements()[0].sql;
    assert!(sql.contains("UPDATE \"global_models\" SET"), "{sql}");
    assert!(sql.contains("\"usage_count\" = \"usage_count\" + $"), "{sql}");
    assert!(sql.contains("WHERE \"global_models\".\"id\" ="), "{sql}");
}

#[tokio::test]
async fn global_model_usage_record_requires_existing_model() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 0,
        }])
        .into_connection();
    let store = ModelStore::new(Database::new(connection));

    let error = store.record_usage(usage_record()).await.unwrap_err();

    assert!(matches!(error, StorageError::NotFound));
}

#[tokio::test]
async fn global_model_usage_batch_once_skips_existing_batch() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[batch_record("batch-1")]])
        .into_connection();
    let store = ModelStore::new(Database::new(connection.clone()));

    let report = store.record_usage_batch_once("batch-1", &[usage_record()]).await.unwrap();

    assert!(report.already_applied);
    assert_eq!(report.applied_records, 0);
    assert_eq!(report.skipped_missing_resource_ids, Vec::<String>::new());
    let statements = logged_sql(connection);
    assert!(statements.iter().any(|sql| sql.contains("SELECT")), "{statements:?}");
    assert!(!statements.iter().any(|sql| sql.contains("UPDATE \"global_models\"")), "{statements:?}");
}

#[tokio::test]
async fn global_model_usage_batch_once_marks_applied_batch() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([Vec::<usage_flush_batches::Model>::new()])
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results([[batch_record("batch-1")]])
        .into_connection();
    let store = ModelStore::new(Database::new(connection.clone()));

    let report = store.record_usage_batch_once("batch-1", &[usage_record()]).await.unwrap();

    assert!(!report.already_applied);
    assert_eq!(report.applied_records, 1);
    assert_eq!(report.skipped_missing_resource_ids, Vec::<String>::new());
    let statements = logged_sql(connection);
    assert!(statements.iter().any(|sql| sql.contains("UPDATE \"global_models\"")), "{statements:?}");
    assert!(
        statements.iter().any(|sql| sql.contains("INSERT INTO \"usage_flush_batches\"")),
        "{statements:?}"
    );
}

#[tokio::test]
async fn global_model_usage_batch_once_skips_missing_models() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([Vec::<usage_flush_batches::Model>::new()])
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 0,
        }])
        .append_query_results([[batch_record("batch-1")]])
        .into_connection();
    let store = ModelStore::new(Database::new(connection.clone()));

    let report = store.record_usage_batch_once("batch-1", &[usage_record()]).await.unwrap();

    assert!(!report.already_applied);
    assert_eq!(report.applied_records, 0);
    assert_eq!(report.skipped_missing_resource_ids, vec!["model-1"]);
    let statements = logged_sql(connection);
    assert!(statements.iter().any(|sql| sql.contains("UPDATE \"global_models\"")), "{statements:?}");
    assert!(
        statements.iter().any(|sql| sql.contains("INSERT INTO \"usage_flush_batches\"")),
        "{statements:?}"
    );
}

fn usage_record() -> GlobalModelUsageRecord {
    GlobalModelUsageRecord {
        model_id: "model-1".into(),
        count: 5,
    }
}

fn batch_record(id: &str) -> usage_flush_batches::Model {
    usage_flush_batches::Model {
        id: id.into(),
        usage_kind: "model".into(),
        record_count: 1,
        created_at: time::Date::from_calendar_date(2026, time::Month::May, 12)
            .unwrap()
            .with_hms(10, 30, 0)
            .unwrap()
            .assume_utc(),
    }
}

fn logged_sql(connection: sea_orm::DatabaseConnection) -> Vec<String> {
    connection
        .into_transaction_log()
        .iter()
        .flat_map(|entry| entry.statements())
        .map(|statement| statement.sql.clone())
        .collect()
}

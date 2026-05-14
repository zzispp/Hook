use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
use storage::{
    Database, StorageError,
    model::{GlobalModelUsageRecord, ModelStore},
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

fn usage_record() -> GlobalModelUsageRecord {
    GlobalModelUsageRecord { model_id: "model-1".into() }
}

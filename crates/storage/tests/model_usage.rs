use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
use storage::{
    Database, StorageError,
    model::{GlobalModelUsageRecord, GlobalModelUserUsageRecord, ModelStore, global_model_user_usage_counts, global_models, provider_models},
    usage_flush::usage_flush_batches,
};
use types::model::GlobalModelListRequest;

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
async fn global_model_usage_record_with_user_increments_user_model_usage_count() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            },
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            },
        ])
        .into_connection();
    let store = ModelStore::new(Database::new(connection.clone()));

    store.record_usage(user_usage_record()).await.unwrap();

    let statements = logged_sql(connection);
    assert!(statements.iter().any(|sql| sql.contains("UPDATE \"global_models\"")), "{statements:?}");
    assert!(
        statements.iter().any(|sql| sql.contains("INSERT INTO global_model_user_usage_counts")),
        "{statements:?}"
    );
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

    let report = store
        .record_usage_batch_once("batch-1", &[usage_record()], &[user_usage_batch_record()])
        .await
        .unwrap();

    assert!(report.already_applied);
    assert_eq!(report.applied_records, 0);
    assert_eq!(report.skipped_missing_resource_ids, Vec::<String>::new());
    let statements = logged_sql(connection);
    assert!(statements.iter().any(|sql| sql.contains("SELECT")), "{statements:?}");
    assert!(!statements.iter().any(|sql| sql.contains("UPDATE \"global_models\"")), "{statements:?}");
    assert!(!statements.iter().any(|sql| sql.contains("global_model_user_usage_counts")), "{statements:?}");
}

#[tokio::test]
async fn global_model_usage_batch_once_marks_applied_batch() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([Vec::<usage_flush_batches::Model>::new()])
        .append_exec_results([
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            },
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            },
        ])
        .append_query_results([[batch_record("batch-1")]])
        .into_connection();
    let store = ModelStore::new(Database::new(connection.clone()));

    let report = store
        .record_usage_batch_once("batch-1", &[usage_record()], &[user_usage_batch_record()])
        .await
        .unwrap();

    assert!(!report.already_applied);
    assert_eq!(report.applied_records, 1);
    assert_eq!(report.skipped_missing_resource_ids, Vec::<String>::new());
    let statements = logged_sql(connection);
    assert!(statements.iter().any(|sql| sql.contains("UPDATE \"global_models\"")), "{statements:?}");
    assert!(
        statements.iter().any(|sql| sql.contains("INSERT INTO global_model_user_usage_counts")),
        "{statements:?}"
    );
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

    let report = store
        .record_usage_batch_once("batch-1", &[usage_record()], &[user_usage_batch_record()])
        .await
        .unwrap();

    assert!(!report.already_applied);
    assert_eq!(report.applied_records, 0);
    assert_eq!(report.skipped_missing_resource_ids, vec!["model-1"]);
    let statements = logged_sql(connection);
    assert!(statements.iter().any(|sql| sql.contains("UPDATE \"global_models\"")), "{statements:?}");
    assert!(
        !statements.iter().any(|sql| sql.contains("INSERT INTO global_model_user_usage_counts")),
        "{statements:?}"
    );
    assert!(
        statements.iter().any(|sql| sql.contains("INSERT INTO \"usage_flush_batches\"")),
        "{statements:?}"
    );
}

#[tokio::test]
async fn model_usage_user_global_model_catalog_overlays_personal_usage_count() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![global_model_record("model-1", 99)]])
        .append_query_results([vec![user_usage_count_record("user-1", "model-1", 7)]])
        .append_query_results([vec![provider_model_record("provider-model-1", "model-1", "provider-1")]])
        .append_query_results([vec![provider_model_record("provider-model-1", "model-1", "provider-1")]])
        .into_connection();
    let store = ModelStore::new(Database::new(connection));

    let response = store.list_user_global_models("user-1", list_request()).await.unwrap();

    assert_eq!(response.total, 1);
    assert_eq!(response.models[0].usage_count, Some(7));
}

#[tokio::test]
async fn model_usage_user_global_model_catalog_returns_zero_without_personal_usage_count() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![global_model_record("model-1", 99)]])
        .append_query_results([Vec::<global_model_user_usage_counts::Model>::new()])
        .append_query_results([Vec::<provider_models::Model>::new()])
        .append_query_results([Vec::<provider_models::Model>::new()])
        .into_connection();
    let store = ModelStore::new(Database::new(connection));

    let response = store.list_user_global_models("user-1", list_request()).await.unwrap();

    assert_eq!(response.models[0].usage_count, Some(0));
}

#[tokio::test]
async fn model_usage_admin_global_model_catalog_keeps_platform_usage_count() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![global_model_record("model-1", 99)]])
        .append_query_results([Vec::<provider_models::Model>::new()])
        .append_query_results([Vec::<provider_models::Model>::new()])
        .into_connection();
    let store = ModelStore::new(Database::new(connection));

    let response = store.list_global_models(list_request()).await.unwrap();

    assert_eq!(response.models[0].usage_count, Some(99));
}

fn usage_record() -> GlobalModelUsageRecord {
    GlobalModelUsageRecord {
        model_id: "model-1".into(),
        count: 5,
        user_id: None,
    }
}

fn list_request() -> GlobalModelListRequest {
    GlobalModelListRequest {
        limit: 100,
        ..GlobalModelListRequest::default()
    }
}

fn user_usage_record() -> GlobalModelUsageRecord {
    GlobalModelUsageRecord {
        user_id: Some("user-1".into()),
        ..usage_record()
    }
}

fn user_usage_batch_record() -> GlobalModelUserUsageRecord {
    GlobalModelUserUsageRecord {
        user_id: "user-1".into(),
        model_id: "model-1".into(),
        count: 5,
    }
}

fn global_model_record(id: &str, usage_count: i64) -> global_models::Model {
    global_models::Model {
        id: id.into(),
        name: id.into(),
        display_name: id.into(),
        default_price_per_request: None,
        default_tiered_pricing: "{\"tiers\":[]}".into(),
        supported_capabilities: None,
        config: None,
        is_active: true,
        usage_count,
        created_at: now(),
        updated_at: now(),
    }
}

fn user_usage_count_record(user_id: &str, model_id: &str, usage_count: i64) -> global_model_user_usage_counts::Model {
    global_model_user_usage_counts::Model {
        user_id: user_id.into(),
        global_model_id: model_id.into(),
        usage_count,
        created_at: now(),
        updated_at: now(),
    }
}

fn provider_model_record(id: &str, model_id: &str, provider_id: &str) -> provider_models::Model {
    provider_models::Model {
        id: id.into(),
        provider_id: provider_id.into(),
        global_model_id: model_id.into(),
        provider_model_name: model_id.into(),
        provider_model_mappings: None,
        is_active: true,
        config: None,
        created_at: now(),
        updated_at: now(),
    }
}

fn batch_record(id: &str) -> usage_flush_batches::Model {
    usage_flush_batches::Model {
        id: id.into(),
        usage_kind: "model".into(),
        record_count: 1,
        created_at: now(),
    }
}

fn now() -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::May, 12)
        .unwrap()
        .with_hms(10, 30, 0)
        .unwrap()
        .assume_utc()
}

fn logged_sql(connection: sea_orm::DatabaseConnection) -> Vec<String> {
    connection
        .into_transaction_log()
        .iter()
        .flat_map(|entry| entry.statements())
        .map(|statement| statement.sql.clone())
        .collect()
}

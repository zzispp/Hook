use sea_orm::{DatabaseBackend, DbErr, MockDatabase, MockExecResult};
use storage::{
    Database,
    model::provider_models,
    provider::{
        ProviderModelRecordBatchUpdate, ProviderModelRecordInput, ProviderStore,
        record::{provider_api_keys, provider_quick_import_key_models},
    },
};

#[tokio::test]
async fn batch_update_model_bindings_commits_deletes_and_creates_in_one_transaction() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[deleted_model_record()]])
        .append_query_results([Vec::<provider_quick_import_key_models::Model>::new()])
        .append_query_results([Vec::<provider_api_keys::Model>::new()])
        .append_query_results([Vec::<provider_models::Model>::new()])
        .append_exec_results([exec_result(), exec_result(), exec_result(), exec_result()])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let bindings = store.batch_update_model_bindings(batch_update()).await.unwrap();

    assert!(bindings.is_empty());
    let statements = sql_statements(connection);
    assert_eq!(statements.iter().filter(|sql| sql.contains("BEGIN")).count(), 1);
    assert_eq!(statements.iter().filter(|sql| sql.contains("COMMIT")).count(), 1);
    assert!(statements.iter().any(|sql| sql.contains("DELETE FROM \"provider_models\"")));
    assert!(statements.iter().any(|sql| sql.contains("DELETE FROM \"provider_model_costs\"")));
    assert!(statements.iter().any(|sql| sql.contains("DELETE FROM \"provider_quick_import_key_models\"")));
    assert!(statements.iter().any(|sql| sql.contains("INSERT INTO \"provider_models\"")));
}

#[tokio::test]
async fn batch_update_model_bindings_rolls_back_when_create_fails() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[deleted_model_record()]])
        .append_query_results([Vec::<provider_quick_import_key_models::Model>::new()])
        .append_query_results([Vec::<provider_api_keys::Model>::new()])
        .append_exec_results([exec_result(), exec_result(), exec_result()])
        .append_exec_errors([DbErr::Custom("insert failed".into())])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let error = store.batch_update_model_bindings(batch_update()).await.unwrap_err();

    assert!(error.to_string().contains("insert failed"));
    let statements = sql_statements(connection);
    assert_eq!(statements.iter().filter(|sql| sql.contains("BEGIN")).count(), 1);
    assert_eq!(statements.iter().filter(|sql| sql.contains("ROLLBACK")).count(), 1);
    assert_eq!(statements.iter().filter(|sql| sql.contains("COMMIT")).count(), 0);
}

fn batch_update() -> ProviderModelRecordBatchUpdate {
    ProviderModelRecordBatchUpdate {
        provider_id: "provider-a".into(),
        create: vec![ProviderModelRecordInput {
            provider_id: "provider-a".into(),
            global_model_id: "model-b".into(),
            provider_model_name: "upstream-model-b".into(),
            provider_model_mapping: None,
            is_active: true,
            config: None,
        }],
        delete_ids: vec!["binding-a".into()],
    }
}

fn exec_result() -> MockExecResult {
    MockExecResult {
        last_insert_id: 0,
        rows_affected: 1,
    }
}

fn deleted_model_record() -> provider_models::Model {
    provider_models::Model {
        id: "binding-a".into(),
        provider_id: "provider-a".into(),
        global_model_id: "model-a".into(),
        provider_model_name: "upstream-model-a".into(),
        provider_model_mappings: None,
        is_active: true,
        config: None,
        created_at: now(),
        updated_at: now(),
    }
}

fn now() -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::May, 11)
        .unwrap()
        .with_hms(12, 0, 0)
        .unwrap()
        .assume_utc()
}

fn sql_statements(connection: sea_orm::DatabaseConnection) -> Vec<String> {
    connection
        .into_transaction_log()
        .into_iter()
        .flat_map(|transaction| transaction.statements().iter().map(|statement| statement.sql.clone()).collect::<Vec<_>>())
        .collect()
}

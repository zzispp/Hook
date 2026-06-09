use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, Value};
use std::{collections::BTreeMap, time::Duration};
use storage::{
    Database,
    provider::{ProviderStore, RequestRecordCleanupOptions},
};

#[tokio::test]
async fn request_record_cleanup_uses_limited_delete_batches() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results(timeout_exec_results(8))
        .append_query_results([vec![request_id_row("req-1"), request_id_row("req-2")]])
        .append_query_results([[deleted_candidates_count(3)]])
        .append_query_results([[deleted_records_count(2)]])
        .append_query_results([[orphan_candidate_counts(1)]])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([[orphan_candidate_counts(0)]])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let result = store.cleanup_request_records(cleanup_options()).await.unwrap();

    assert_eq!(result.deleted_records, 2);
    assert_eq!(result.deleted_candidates, 4);
    let statements = logged_statements(connection);
    let sql = sql_strings(&statements);
    assert!(sql.iter().any(|item| item.contains("set_config('statement_timeout'")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("LIMIT $2 FOR UPDATE SKIP LOCKED")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("deleted_candidates AS")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("deleted_records AS")), "{sql:?}");
    assert!(
        statement_position(&sql, "deleted_candidates AS") < statement_position(&sql, "deleted_records AS"),
        "{sql:?}"
    );
    assert!(
        sql.iter()
            .all(|item| !item.contains("DELETE FROM \"request_records\" WHERE \"request_records\".\"created_at\" <")),
        "{sql:?}"
    );
}

#[tokio::test]
async fn request_record_cleanup_uses_indexed_payload_marker() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results(timeout_exec_results(4))
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([[orphan_candidate_counts(0)]])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let result = store.cleanup_request_records(cleanup_options()).await.unwrap();

    assert_eq!(result.compressed_records, 0);
    assert_eq!(result.compressed_candidates, 0);
    let statements = logged_statements(connection);
    let sql = sql_strings(&statements);
    let string_values = bound_string_values(&statements);
    assert!(sql.iter().any(|item| item.contains("payload_compressed_at IS NULL")), "{sql:?}");
    assert!(sql.iter().all(|item| !item.contains("NOT LIKE")), "{sql:?}");
    assert!(
        string_values.iter().all(|item| !item.contains("__hook_compressed_payload__")),
        "{string_values:?}"
    );
}

#[tokio::test]
async fn request_record_cleanup_stops_when_budget_cannot_cover_statement() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let result = store
        .cleanup_request_records(RequestRecordCleanupOptions {
            max_runtime: Duration::from_secs(1),
            statement_timeout_seconds: 15,
            ..cleanup_options()
        })
        .await
        .unwrap();

    assert_eq!(result.deleted_records, 0);
    assert_eq!(result.deleted_candidates, 0);
    assert_eq!(result.compressed_records, 0);
    assert_eq!(result.compressed_candidates, 0);
    assert!(result.time_budget_exhausted);
    assert!(logged_statements(connection).is_empty());
}

fn cleanup_options() -> RequestRecordCleanupOptions {
    RequestRecordCleanupOptions {
        record_cutoff: time::OffsetDateTime::now_utc(),
        payload_cutoff: time::OffsetDateTime::now_utc(),
        delete_batch_size: 200,
        compress_batch_size: 50,
        max_runtime: Duration::from_secs(60),
        batch_sleep: Duration::ZERO,
        statement_timeout_seconds: 15,
        lock_timeout_seconds: 2,
    }
}

fn request_id_row(request_id: &str) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([("request_id", Value::from(request_id.to_owned()))])
}

fn deleted_candidates_count(deleted_candidates: i64) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([("deleted_candidates", Value::from(deleted_candidates))])
}

fn deleted_records_count(deleted_records: i64) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([("deleted_records", Value::from(deleted_records))])
}

fn orphan_candidate_counts(deleted_candidates: i64) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([("deleted_candidates", Value::from(deleted_candidates))])
}

fn timeout_exec_results(count: usize) -> Vec<MockExecResult> {
    vec![MockExecResult::default(); count]
}

fn logged_statements(connection: sea_orm::DatabaseConnection) -> Vec<sea_orm::Statement> {
    connection.into_transaction_log().iter().flat_map(|entry| entry.statements()).cloned().collect()
}

fn sql_strings(statements: &[sea_orm::Statement]) -> Vec<String> {
    statements.iter().map(|statement| statement.sql.clone()).collect()
}

fn statement_position(sql: &[String], pattern: &str) -> usize {
    sql.iter().position(|item| item.contains(pattern)).unwrap()
}

fn bound_string_values(statements: &[sea_orm::Statement]) -> Vec<String> {
    statements
        .iter()
        .filter_map(|statement| statement.values.as_ref())
        .flat_map(|values| values.0.iter())
        .filter_map(|value| match value {
            Value::String(Some(text)) => Some(text.clone()),
            _ => None,
        })
        .collect()
}

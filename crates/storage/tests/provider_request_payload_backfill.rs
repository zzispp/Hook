use std::collections::BTreeMap;

use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, Value};
use storage::{
    Database,
    provider::{ProviderStore, RequestPayloadBackfillOptions},
};

#[tokio::test]
async fn request_payload_backfill_moves_legacy_payloads_to_partitioned_store() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[legacy_record_row()]])
        .append_exec_results([exec_result()])
        .append_query_results([[payload_key_row()]])
        .append_exec_results([exec_result(), exec_result()])
        .append_query_results([[legacy_candidate_row()]])
        .append_exec_results([exec_result()])
        .append_query_results([[payload_key_row()]])
        .append_exec_results([exec_result(), exec_result()])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let result = store.backfill_legacy_request_payloads(backfill_options()).await.unwrap();

    assert_eq!(result.records_backfilled, 1);
    assert_eq!(result.candidates_backfilled, 1);
    let sql = logged_sql(connection);
    assert!(sql.iter().any(|item| item.contains("INSERT INTO request_payloads")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("UPDATE request_payloads SET status = 'stored'")), "{sql:?}");
    assert!(
        sql.iter().any(|item| item.contains("UPDATE request_records SET request_headers = NULL")),
        "{sql:?}"
    );
    assert!(
        sql.iter()
            .any(|item| item.contains("UPDATE request_candidates SET provider_request_headers = NULL")),
        "{sql:?}"
    );
}

fn backfill_options() -> RequestPayloadBackfillOptions {
    RequestPayloadBackfillOptions {
        batch_size: 10,
        minimum_created_at: ts(),
    }
}

fn legacy_record_row() -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("request_id", Value::from("req-1")),
        ("request_headers", Value::from(r#"{"authorization":"****"}"#)),
        ("request_body", Value::String(None)),
        ("client_response_headers", Value::String(None)),
        ("client_response_body", Value::String(None)),
    ])
}

fn legacy_candidate_row() -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("id", Value::from("candidate-1")),
        ("provider_request_headers", Value::String(None)),
        ("provider_request_body", Value::String(None)),
        ("provider_response_headers", Value::String(None)),
        ("provider_response_body", Value::from(r#"{"id":"resp-1"}"#)),
    ])
}

fn payload_key_row() -> BTreeMap<&'static str, Value> {
    BTreeMap::from([("created_at", Value::from(ts()))])
}

fn exec_result() -> MockExecResult {
    MockExecResult {
        rows_affected: 1,
        ..Default::default()
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

fn ts() -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::June, 9)
        .unwrap()
        .with_hms(0, 0, 0)
        .unwrap()
        .assume_utc()
}

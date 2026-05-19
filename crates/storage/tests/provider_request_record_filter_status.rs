use std::collections::BTreeMap;

use sea_orm::{DatabaseBackend, MockDatabase, Value};
use storage::{Database, provider::ProviderStore};
use types::provider::RequestRecordListRequest;

#[tokio::test]
async fn request_record_storage_filters_summary_by_failover_status() {
    let connection = filtered_request_records("failover").await;
    let logs = connection.into_transaction_log();
    let count_sql = &logs[0].statements()[0].sql;

    assert!(count_sql.contains("r.has_failover = TRUE"), "{count_sql}");
}

#[tokio::test]
async fn request_record_storage_filters_summary_by_retry_status() {
    let connection = filtered_request_records("retry").await;
    let logs = connection.into_transaction_log();
    let count_sql = &logs[0].statements()[0].sql;

    assert!(count_sql.contains("r.has_retry = TRUE"), "{count_sql}");
}

async fn filtered_request_records(status: &str) -> sea_orm::DatabaseConnection {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[count_row(0)]])
        .append_query_results([Vec::<storage::provider::record::request_records::Model>::new()])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    store
        .list_request_records(RequestRecordListRequest {
            status: Some(status.into()),
            ..Default::default()
        })
        .await
        .unwrap();

    connection
}

fn count_row(total: i64) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([("total", total.into())])
}

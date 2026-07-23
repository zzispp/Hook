use std::collections::BTreeMap;

use sea_orm::{DatabaseBackend, MockDatabase, sea_query::Value};
use storage::{Database, provider::ProviderStore};
use types::provider::RoutingMetricWindow;

#[tokio::test]
async fn routing_metric_query_filters_to_minute_buckets() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let records = store.list_routing_metrics(RoutingMetricWindow::SevenDays).await.unwrap();

    assert!(records.is_empty());
    let transactions = connection.into_transaction_log();
    assert_eq!(transactions.len(), 1);
    let sql = &transactions[0].statements()[0].sql;
    assert!(sql.contains("bucket_granularity = 'minute'"), "{sql}");
}

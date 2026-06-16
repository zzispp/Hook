use std::collections::BTreeMap;

use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, Value};
use storage::{Database, provider::ProviderStore};

#[tokio::test]
async fn route_state_query_returns_matching_ema_states() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![route_state_row()]])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let states = store
        .list_routing_route_states("model-a", "openai:chat", false)
        .await
        .expect("route state query should succeed");

    assert_eq!(states.len(), 1);
    assert_eq!(states[0].route.key_id, "key-a");
    assert_eq!(states[0].ema_success_rate, 0.75);
    assert_eq!(states[0].ema_latency_ms, Some(900.0));
    assert_eq!(states[0].ema_ttfb_ms, Some(320.0));
    assert_eq!(states[0].ema_output_tps, Some(42.0));
    assert_eq!(states[0].sample_count, 24);
    let logs = connection.into_transaction_log();
    let sql = &logs[0].statements()[0].sql;
    assert!(sql.contains("FROM routing_route_states"), "{sql}");
    assert!(sql.contains("global_model_id = $1"), "{sql}");
    assert!(sql.contains("client_api_format = $2"), "{sql}");
    assert!(sql.contains("is_stream = $3"), "{sql}");
}

fn route_state_row() -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("provider_id", Value::from("provider-a".to_owned())),
        ("key_id", Value::from("key-a".to_owned())),
        ("endpoint_id", Value::from("endpoint-a".to_owned())),
        ("global_model_id", Value::from("model-a".to_owned())),
        ("client_api_format", Value::from("openai:chat".to_owned())),
        ("provider_api_format", Value::from("openai:chat".to_owned())),
        ("is_stream", Value::from(false)),
        ("ema_success_rate", Value::from(Decimal::new(75, 2))),
        ("ema_latency_ms", Value::from(Decimal::new(900, 0))),
        ("ema_ttfb_ms", Value::from(Decimal::new(320, 0))),
        ("ema_output_tps", Value::from(Decimal::new(42, 0))),
        ("sample_count", Value::from(24_i64)),
        ("last_updated_at", Value::from(time::OffsetDateTime::now_utc())),
    ])
}

use std::collections::BTreeMap;

use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, Value};
use storage::{
    Database,
    dashboard::{DashboardCostAnalysisWindow, DashboardProviderAggregationQuery, DashboardStore},
};

#[tokio::test]
async fn provider_aggregation_returns_stage_latency_averages() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![provider_row()]])
        .into_connection();
    let store = DashboardStore::new(Database::new(connection.clone()));

    let providers = store.provider_aggregation(query()).await.unwrap();

    assert_eq!(providers[0].avg_response_headers_ms, Some(40.0));
    assert_eq!(providers[0].avg_first_sse_event_ms, Some(90.0));
    assert_eq!(providers[0].avg_first_output_ms, Some(360.0));
    assert_eq!(providers[0].avg_sse_to_output_ms, Some(270.0));
    let logs = connection.into_transaction_log();
    let sql = &logs[0].statements()[0].sql;
    assert!(sql.contains("response_headers_total_ms"), "{sql}");
    assert!(sql.contains("first_sse_event_total_ms"), "{sql}");
    assert!(sql.contains("first_output_total_ms"), "{sql}");
    assert!(sql.contains("sse_to_output_total_ms"), "{sql}");
}

fn query() -> DashboardProviderAggregationQuery {
    DashboardProviderAggregationQuery {
        window: DashboardCostAnalysisWindow {
            start_date: date(1),
            end_date: date(2),
            started_at: timestamp(1),
            ended_at: timestamp(2),
            tz_offset_minutes: 0,
        },
        limit: 10,
    }
}

fn provider_row() -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("dimension_id", Value::from("provider-a")),
        ("provider", Value::from("Provider A")),
        ("request_count", Value::from(3_i64)),
        ("success_count", Value::from(3_i64)),
        ("failed_count", Value::from(0_i64)),
        ("input_tokens", Value::from(300_i64)),
        ("output_tokens", Value::from(90_i64)),
        ("cache_creation_tokens", Value::from(0_i64)),
        ("cache_read_tokens", Value::from(0_i64)),
        ("total_tokens", Value::from(390_i64)),
        ("total_cost", Value::from(Decimal::new(123, 2))),
        ("upstream_total_cost", Value::from(Decimal::new(80, 2))),
        ("total_latency_ms", Value::from(900_i64)),
        ("latency_sample_count", Value::from(3_i64)),
        ("avg_response_headers_ms", Value::from(40.0_f64)),
        ("avg_first_sse_event_ms", Value::from(90.0_f64)),
        ("avg_first_output_ms", Value::from(360.0_f64)),
        ("avg_sse_to_output_ms", Value::from(270.0_f64)),
    ])
}

fn date(day: u8) -> time::Date {
    time::Date::from_calendar_date(2026, time::Month::May, day).unwrap()
}

fn timestamp(day: u8) -> time::OffsetDateTime {
    date(day).with_hms(0, 0, 0).unwrap().assume_utc()
}

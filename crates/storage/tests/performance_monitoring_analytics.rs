use std::collections::BTreeMap;

use sea_orm::{DatabaseBackend, MockDatabase, sea_query::Value};
use storage::{Database, performance_monitoring::PerformanceMonitoringStore};
use types::performance_monitoring::{PerformanceMonitoringAnalyticsRequest, PerformanceMonitoringRange};

#[tokio::test]
async fn analytics_maps_errors_and_upstream_rows() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![percentile_row()]])
        .append_query_results([vec![error_distribution_row("rate_limit", 3)]])
        .append_query_results([vec![error_trend_row(0, "rate_limit", 2), error_trend_row(0, "server_error", 1)]])
        .append_query_results([[upstream_summary_row()]])
        .append_query_results([vec![upstream_provider_row()]])
        .append_query_results([vec![upstream_timeline_row()]])
        .append_query_results([vec![recent_error_row("req-2"), recent_error_row("req-1")]])
        .into_connection();
    let store = PerformanceMonitoringStore::new(Database::new(connection.clone()));

    let response = store.analytics(request(), ts(3_600)).await.unwrap();

    assert_eq!(response.percentiles[0].p90_ttfb_ms, Some(80));
    assert_eq!(response.error_distribution[0].category, "rate_limit");
    assert_eq!(response.error_trend[0].total, 3);
    assert_eq!(response.error_trend[0].categories.len(), 2);
    assert_eq!(response.upstream_performance.summary.success_rate, 0.75);
    assert_eq!(response.upstream_performance.summary.error_rate, 0.25);
    assert_eq!(response.upstream_performance.summary.avg_output_tps, Some(25.5));
    assert_eq!(response.upstream_performance.providers[0].ttfb_sample_count, 3);
    assert_eq!(response.upstream_performance.timeline[0].avg_ttfb_ms, Some(45.0));
    assert_eq!(response.recent_errors[0].request_id, "req-2");

    let logs = connection.into_transaction_log();
    assert!(logs[0].statements()[0].sql.contains("p99_ttfb_ms"));
    assert!(logs[0].statements()[0].sql.contains("status = 'success'"));
    assert!(logs[3].statements()[0].sql.contains("FROM request_candidates"));
    assert!(logs[5].statements()[0].sql.contains("bucketed AS"));
}

#[tokio::test]
async fn analytics_returns_empty_upstream_without_timeline_query() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([[empty_upstream_summary_row()]])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .into_connection();
    let store = PerformanceMonitoringStore::new(Database::new(connection.clone()));

    let response = store.analytics(request(), ts(3_600)).await.unwrap();

    assert!(response.percentiles.is_empty());
    assert!(response.upstream_performance.timeline.is_empty());
    assert_eq!(response.upstream_performance.summary.request_count, 0);
    assert_eq!(response.upstream_performance.summary.avg_output_tps, None);
    assert_eq!(connection.into_transaction_log().len(), 6);
}

fn request() -> PerformanceMonitoringAnalyticsRequest {
    PerformanceMonitoringAnalyticsRequest {
        range: PerformanceMonitoringRange::Today,
        limit: Some(4),
        slow_threshold_ms: Some(1_000),
        provider_id: None,
        model: None,
        api_format: None,
        is_stream: None,
        needs_conversion: None,
    }
}

fn percentile_row() -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("bucket_started_at", Value::from(ts(0))),
        ("bucket_ended_at", Value::from(ts(3_600))),
        ("p50_latency_ms", Value::from(100_i64)),
        ("p90_latency_ms", Value::from(300_i64)),
        ("p99_latency_ms", Value::from(900_i64)),
        ("p50_ttfb_ms", Value::from(30_i64)),
        ("p90_ttfb_ms", Value::from(80_i64)),
        ("p99_ttfb_ms", Value::from(140_i64)),
    ])
}

fn error_distribution_row(category: &'static str, count: i64) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([("category", Value::from(category)), ("count", Value::from(count))])
}

fn error_trend_row(bucket: i64, category: &'static str, count: i64) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("bucket_started_at", Value::from(ts(bucket))),
        ("category", Value::from(category)),
        ("count", Value::from(count)),
    ])
}

fn upstream_summary_row() -> BTreeMap<&'static str, Value> {
    let mut row = empty_upstream_summary_row();
    row.insert("request_count", Value::from(4_i64));
    row.insert("success_count", Value::from(3_i64));
    row.insert("error_count", Value::from(1_i64));
    row.insert("avg_output_tps", Value::from(25.5_f64));
    row.insert("ttfb_sample_count", Value::from(3_i64));
    row
}

fn empty_upstream_summary_row() -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("request_count", Value::from(0_i64)),
        ("success_count", Value::from(0_i64)),
        ("error_count", Value::from(0_i64)),
        ("output_tokens", Value::from(0_i64)),
        ("avg_output_tps", Value::Double(None)),
        ("avg_ttfb_ms", Value::Double(None)),
        ("avg_latency_ms", Value::Double(None)),
        ("p90_latency_ms", Value::BigInt(None)),
        ("p99_latency_ms", Value::BigInt(None)),
        ("p90_ttfb_ms", Value::BigInt(None)),
        ("p99_ttfb_ms", Value::BigInt(None)),
        ("tps_sample_count", Value::from(0_i64)),
        ("latency_sample_count", Value::from(0_i64)),
        ("ttfb_sample_count", Value::from(0_i64)),
        ("slow_request_count", Value::from(0_i64)),
    ])
}

fn upstream_provider_row() -> BTreeMap<&'static str, Value> {
    let mut row = upstream_summary_row();
    row.insert("provider_id", Value::from("provider-a"));
    row.insert("provider_name", Value::from("Provider A"));
    row
}

fn upstream_timeline_row() -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("bucket_started_at", Value::from(ts(0))),
        ("bucket_ended_at", Value::from(ts(3_600))),
        ("provider_id", Value::from("provider-a")),
        ("provider_name", Value::from("Provider A")),
        ("request_count", Value::from(4_i64)),
        ("success_count", Value::from(3_i64)),
        ("error_count", Value::from(1_i64)),
        ("output_tokens", Value::from(240_i64)),
        ("avg_output_tps", Value::from(25.5_f64)),
        ("avg_ttfb_ms", Value::from(45.0_f64)),
        ("avg_latency_ms", Value::from(320.0_f64)),
        ("slow_request_count", Value::from(1_i64)),
    ])
}

fn recent_error_row(request_id: &'static str) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("created_at", Value::from(ts(30))),
        ("request_id", Value::from(request_id)),
        ("provider_id", Value::from("provider-a")),
        ("provider_name", Value::from("Provider A")),
        ("model", Value::from("gpt-5")),
        ("status_code", Value::from(429_i32)),
        ("error_type", Value::from("rate_limit")),
        ("error_message", Value::from("limited")),
        ("latency_ms", Value::from(800_i64)),
        ("ttfb_ms", Value::from(120_i64)),
    ])
}

fn ts(seconds: i64) -> time::OffsetDateTime {
    time::OffsetDateTime::from_unix_timestamp(seconds).unwrap()
}

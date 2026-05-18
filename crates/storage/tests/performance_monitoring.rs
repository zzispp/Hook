use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, sea_query::Value};
use std::collections::BTreeMap;
use storage::{
    Database,
    performance_monitoring::{PerformanceMonitoringStore, PerformanceSnapshotInput, record::snapshots},
};
use types::performance_monitoring::{CoreRequestMetrics, PerformanceMonitoringRange, PerformanceSnapshotMetrics, SnapshotDataStatus, SnapshotGranularity};

#[tokio::test]
async fn realtime_range_queries_minute_snapshot_buckets() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([Vec::<snapshots::Model>::new()])
        .append_query_results([[summary_row(0)]])
        .append_query_results([Vec::<BTreeMap<String, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<String, Value>>::new()])
        .into_connection();
    let store = PerformanceMonitoringStore::new(Database::new(connection.clone()));

    let response = store.overview(PerformanceMonitoringRange::Realtime, ts(600)).await.unwrap();

    assert_eq!(response.bucket_granularity, SnapshotGranularity::Minute);
    assert_eq!(response.status, SnapshotDataStatus::EmptySnapshot);
    assert_eq!(response.effective_range.started_at, "1970-01-01T00:05:00Z");
    let logs = connection.into_transaction_log();
    let sql = &logs[0].statements()[0].sql;
    assert!(sql.contains("\"bucket_granularity\" ="), "{sql}");
}

#[tokio::test]
async fn today_range_queries_hour_snapshot_buckets() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([Vec::<snapshots::Model>::new()])
        .append_query_results([[summary_row(0)]])
        .append_query_results([Vec::<BTreeMap<String, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<String, Value>>::new()])
        .into_connection();
    let store = PerformanceMonitoringStore::new(Database::new(connection.clone()));

    let response = store.overview(PerformanceMonitoringRange::Today, ts(86_400)).await.unwrap();

    assert_eq!(response.bucket_granularity, SnapshotGranularity::Hour);
    assert_eq!(response.status, SnapshotDataStatus::EmptySnapshot);
    assert_eq!(response.effective_range.started_at, "1970-01-01T00:00:00Z");
}

#[tokio::test]
async fn range_all_queries_day_snapshot_buckets_and_live_tail() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![snapshot("day-1", SnapshotGranularity::Day, 0)]])
        .append_query_results([[summary_row(0)]])
        .append_query_results([Vec::<BTreeMap<String, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<String, Value>>::new()])
        .into_connection();
    let store = PerformanceMonitoringStore::new(Database::new(connection.clone()));

    let response = store.overview(PerformanceMonitoringRange::All, ts(120_000)).await.unwrap();

    assert_eq!(response.bucket_granularity, SnapshotGranularity::Day);
    assert_eq!(response.max_series_points, types::performance_monitoring::MAX_SERIES_POINTS);
    assert_eq!(response.status, SnapshotDataStatus::Ready);
    assert_eq!(response.effective_range.started_at, "1970-01-01T00:00:00Z");
    let logs = connection.into_transaction_log();
    let sql = &logs[0].statements()[0].sql;
    assert!(sql.contains("FROM performance_monitoring_snapshots"), "{sql}");
    assert!(sql.contains("ROW_NUMBER() OVER"), "{sql}");
    assert!(sql.contains("DISTINCT ON (sample_slot)"), "{sql}");
    assert!(sql.contains("LIMIT $2"), "{sql}");
    assert!(logs.iter().skip(1).any(|log| log.statements()[0].sql.contains("FROM request_records")));
}

#[tokio::test]
async fn range_all_appends_live_request_window_without_day_snapshots() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([Vec::<snapshots::Model>::new()])
        .append_query_results([[summary_row(3)]])
        .append_query_results([Vec::<BTreeMap<String, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<String, Value>>::new()])
        .into_connection();
    let store = PerformanceMonitoringStore::new(Database::new(connection.clone()));

    let response = store.overview(PerformanceMonitoringRange::All, ts(120_000)).await.unwrap();

    assert_eq!(response.bucket_granularity, SnapshotGranularity::Day);
    assert_eq!(response.status, SnapshotDataStatus::Ready);
    assert_eq!(response.series.len(), 1);
    assert_eq!(response.series[0].metrics.core.request_count, 3);
    let logs = connection.into_transaction_log();
    assert!(logs.iter().any(|log| log.statements()[0].sql.contains("FROM request_records")));
}

#[tokio::test]
async fn thirty_day_range_uses_hour_buckets_at_point_cap() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([Vec::<snapshots::Model>::new()])
        .append_query_results([[summary_row(0)]])
        .append_query_results([Vec::<BTreeMap<String, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<String, Value>>::new()])
        .into_connection();
    let store = PerformanceMonitoringStore::new(Database::new(connection.clone()));

    let response = store.overview(PerformanceMonitoringRange::ThirtyDays, ts(2_600_000)).await.unwrap();

    assert_eq!(response.bucket_granularity, SnapshotGranularity::Hour);
    assert_eq!(response.status, SnapshotDataStatus::EmptySnapshot);
    let logs = connection.into_transaction_log();
    let sql = &logs[0].statements()[0].sql;
    assert!(sql.contains("LIMIT $"), "{sql}");
    assert!(sql.contains("\"bucket_granularity\" ="), "{sql}");
}

#[tokio::test]
async fn cleanup_deletes_snapshots_before_cutoff() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 3,
        }])
        .into_connection();
    let store = PerformanceMonitoringStore::new(Database::new(connection.clone()));

    let deleted = store.delete_snapshots_before(ts(86_400)).await.unwrap();

    assert_eq!(deleted, 3);
    let logs = connection.into_transaction_log();
    let sql = &logs[0].statements()[0].sql;
    assert!(sql.contains("DELETE FROM \"performance_monitoring_snapshots\""), "{sql}");
    assert!(sql.contains("\"bucket_started_at\" <"), "{sql}");
}

#[tokio::test]
async fn upsert_snapshot_updates_existing_bucket() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[snapshot("existing", SnapshotGranularity::Minute, 0)]])
        .append_query_results([[snapshot("existing", SnapshotGranularity::Minute, 0)]])
        .into_connection();
    let store = PerformanceMonitoringStore::new(Database::new(connection.clone()));

    store.upsert_snapshot(snapshot_input(SnapshotGranularity::Minute, 0)).await.unwrap();

    let logs = connection.into_transaction_log();
    assert!(logs[1].statements()[0].sql.contains("UPDATE \"performance_monitoring_snapshots\""));
}

fn snapshot_input(granularity: SnapshotGranularity, started_at_seconds: i64) -> PerformanceSnapshotInput {
    PerformanceSnapshotInput {
        bucket_granularity: granularity,
        bucket_started_at: ts(started_at_seconds),
        bucket_ended_at: ts(started_at_seconds + granularity.bucket_seconds()),
        metrics: metrics(1),
    }
}

fn snapshot(id: &str, granularity: SnapshotGranularity, started_at_seconds: i64) -> snapshots::Model {
    snapshots::Model {
        id: id.into(),
        bucket_granularity: granularity.as_str().into(),
        bucket_started_at: ts(started_at_seconds),
        bucket_ended_at: ts(started_at_seconds + granularity.bucket_seconds()),
        metrics: serde_json::to_string(&metrics(7)).unwrap(),
        created_at: ts(started_at_seconds),
        updated_at: ts(started_at_seconds),
    }
}

fn metrics(request_count: i64) -> PerformanceSnapshotMetrics {
    PerformanceSnapshotMetrics {
        core: CoreRequestMetrics {
            request_count,
            qps: request_count as f64 / 60.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn summary_row(request_count: i64) -> BTreeMap<String, Value> {
    BTreeMap::from([
        ("request_count".into(), Value::from(request_count)),
        ("success_count".into(), Value::from(request_count)),
        ("error_count".into(), Value::from(0_i64)),
        ("concurrent_requests".into(), Value::from(0_i64)),
        ("timeout_count".into(), Value::from(0_i64)),
        ("rate_limited_count".into(), Value::from(0_i64)),
        ("server_error_count".into(), Value::from(0_i64)),
        ("p50_latency_ms".into(), Value::BigInt(None)),
        ("p95_latency_ms".into(), Value::BigInt(None)),
        ("p99_latency_ms".into(), Value::BigInt(None)),
        ("p50_ttft_ms".into(), Value::BigInt(None)),
        ("p95_ttft_ms".into(), Value::BigInt(None)),
        ("p99_ttft_ms".into(), Value::BigInt(None)),
        ("retry_count".into(), Value::from(0_i64)),
        ("circuit_breaker_count".into(), Value::from(0_i64)),
        ("stream_request_count".into(), Value::from(0_i64)),
        ("prompt_tokens".into(), Value::from(0_i64)),
        ("completion_tokens".into(), Value::from(0_i64)),
        ("total_tokens".into(), Value::from(0_i64)),
        ("failover_count".into(), Value::from(0_i64)),
        ("cache_hit_count".into(), Value::from(0_i64)),
        ("cost".into(), Value::from(rust_decimal::Decimal::ZERO)),
        ("quota_limited_count".into(), Value::from(0_i64)),
    ])
}

fn ts(seconds: i64) -> time::OffsetDateTime {
    time::OffsetDateTime::from_unix_timestamp(seconds).unwrap()
}

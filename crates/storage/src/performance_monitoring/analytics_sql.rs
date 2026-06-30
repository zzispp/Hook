use sea_orm::Value;
use types::performance_monitoring::SnapshotGranularity;

use super::types::SnapshotQueryPlan;

pub(super) const DEFAULT_ANALYTICS_LIMIT: usize = 8;
pub(super) const MAX_ANALYTICS_LIMIT: usize = 20;
pub(super) const DEFAULT_SLOW_THRESHOLD_MS: i64 = 10_000;
pub(super) const RECENT_ERROR_LIMIT: i64 = 12;

pub(super) fn plan_values(plan: &SnapshotQueryPlan) -> Vec<Value> {
    vec![Value::from(plan.started_at), Value::from(plan.ended_at)]
}

pub(super) fn plan_bucket_values(plan: &SnapshotQueryPlan) -> Vec<Value> {
    vec![
        Value::from(plan.started_at),
        Value::from(plan.ended_at),
        Value::from(plan.granularity.as_str().to_owned()),
    ]
}

pub(super) fn percentile_sql(_granularity: SnapshotGranularity) -> String {
    format!(
        "WITH histogram_raw AS ( \
        SELECT bucket_started_at, bucket_ended_at, metric_kind, le_ms, COALESCE(SUM(sample_count), 0)::bigint AS bucket_count \
        FROM dashboard_latency_histogram_buckets \
        WHERE source_type = 'request' AND bucket_granularity = $3 AND bucket_started_at >= $1 AND bucket_started_at < $2 \
        GROUP BY bucket_started_at, bucket_ended_at, metric_kind, le_ms \
    ), histogram AS ( \
        SELECT bucket_started_at, bucket_ended_at, metric_kind, le_ms, bucket_count, \
            SUM(bucket_count) OVER (PARTITION BY bucket_started_at, metric_kind ORDER BY le_ms)::bigint AS cumulative_count \
        FROM histogram_raw \
    ), totals AS ( \
        SELECT bucket_started_at, metric_kind, SUM(bucket_count)::bigint AS total_count FROM histogram_raw GROUP BY bucket_started_at, metric_kind \
    ), buckets AS ( \
        SELECT DISTINCT bucket_started_at, bucket_ended_at FROM histogram \
    ) \
    SELECT b.bucket_started_at, b.bucket_ended_at, \
        {} \
    FROM buckets b ORDER BY b.bucket_started_at ASC",
        percentile_selects().join(", ")
    )
}

pub(super) fn error_distribution_sql() -> &'static str {
    "SELECT error_category AS category, COUNT(*)::bigint AS count \
    FROM dashboard_recent_error_snapshots \
    WHERE created_at >= $1 AND created_at < $2 \
    GROUP BY category ORDER BY count DESC, category ASC"
}

pub(super) fn error_trend_sql(granularity: SnapshotGranularity) -> String {
    format!(
        "SELECT {bucket} AS bucket_started_at, error_category AS category, COUNT(*)::bigint AS count \
        FROM dashboard_recent_error_snapshots \
        WHERE created_at >= $1 AND created_at < $2 \
        GROUP BY bucket_started_at, category ORDER BY bucket_started_at ASC, category ASC",
        bucket = bucket_expr("created_at", granularity)
    )
}

pub(super) fn recent_errors_sql() -> &'static str {
    "SELECT created_at, request_id, provider_id, provider_name, model, status_code, error_type, error_message, response_headers_ms, first_token_ms, latency_ms, first_byte_ms \
    FROM dashboard_recent_error_snapshots \
    WHERE created_at >= $1 AND created_at < $2 \
    ORDER BY created_at DESC, request_id DESC LIMIT $3"
}

fn bucket_expr(column: &str, granularity: SnapshotGranularity) -> String {
    format!("date_trunc('{}', {column})", granularity.as_str())
}

fn percentile_selects() -> Vec<String> {
    [
        ("latency", "latency"),
        ("first_byte", "first_byte"),
        ("response_headers", "response_headers"),
        ("first_token", "first_token"),
    ]
    .into_iter()
    .flat_map(|(kind, suffix)| {
        [
            percentile_expr(kind, "0.50", &format!("p50_{suffix}_ms")),
            percentile_expr(kind, "0.90", &format!("p90_{suffix}_ms")),
            percentile_expr(kind, "0.99", &format!("p99_{suffix}_ms")),
        ]
    })
    .collect()
}

fn percentile_expr(metric_kind: &str, quantile: &str, alias: &str) -> String {
    format!(
        "(SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.bucket_started_at = h.bucket_started_at AND t.metric_kind = h.metric_kind WHERE h.bucket_started_at = b.bucket_started_at AND h.metric_kind = '{metric_kind}' AND h.cumulative_count >= CEIL(t.total_count::numeric * {quantile})) AS {alias}"
    )
}

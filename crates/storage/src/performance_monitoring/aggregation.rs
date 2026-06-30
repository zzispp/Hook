use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, Value};
use types::performance_monitoring::{CoreRequestMetrics, LlmBusinessMetrics, PerformanceSnapshotMetrics};

use crate::{StorageError, StorageResult};

use super::{PerformanceMonitoringStore, PerformanceSnapshotInput, SnapshotAggregationWindow, SystemMetricsSnapshot};

pub(super) async fn aggregate_window_with_system(
    store: &PerformanceMonitoringStore,
    window: SnapshotAggregationWindow,
    system: SystemMetricsSnapshot,
) -> StorageResult<()> {
    let summary = request_summary(store, &window).await?;
    let circuit_breaker_count = circuit_breaker_count(store, &window).await?;
    let input = PerformanceSnapshotInput {
        bucket_granularity: window.granularity,
        bucket_started_at: window.started_at,
        bucket_ended_at: window.ended_at,
        metrics: metrics(summary, circuit_breaker_count, window_duration_seconds(&window), system),
    };
    store.upsert_snapshot(input).await
}

pub(super) async fn aggregate_window_point(
    store: &PerformanceMonitoringStore,
    window: SnapshotAggregationWindow,
    system: SystemMetricsSnapshot,
) -> StorageResult<PerformanceSnapshotMetrics> {
    let summary = request_summary(store, &window).await?;
    let circuit_breaker_count = circuit_breaker_count(store, &window).await?;
    Ok(metrics(summary, circuit_breaker_count, window_duration_seconds(&window), system))
}

async fn request_summary(store: &PerformanceMonitoringStore, window: &SnapshotAggregationWindow) -> StorageResult<RequestSummaryRow> {
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, summary_sql(), window_values(window));
    store
        .connection()
        .query_one_raw(statement)
        .await?
        .map(|row| RequestSummaryRow::from_query_result(&row, ""))
        .transpose()?
        .ok_or_else(|| StorageError::Database("performance summary query returned no row".into()))
}

async fn circuit_breaker_count(store: &PerformanceMonitoringStore, window: &SnapshotAggregationWindow) -> StorageResult<i64> {
    let statement = Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT COUNT(*)::bigint AS count FROM provider_cooldown_events WHERE triggered_at >= $1 AND triggered_at < $2",
        time_window_values(window),
    );
    let row = store
        .connection()
        .query_one_raw(statement)
        .await?
        .ok_or_else(|| StorageError::Database("provider cooldown event count query returned no row".into()))?;
    Ok(row.try_get::<i64>("", "count")?)
}

fn metrics(summary: RequestSummaryRow, circuit_breaker_count: i64, seconds: i64, system: SystemMetricsSnapshot) -> PerformanceSnapshotMetrics {
    let request_count = summary.request_count.unwrap_or_default();
    let error_count = summary.error_count.unwrap_or_default();
    let total_tokens = summary.total_tokens.unwrap_or_default();
    PerformanceSnapshotMetrics {
        core: core_metrics(&summary, request_count, error_count, circuit_breaker_count, seconds),
        llm: llm_metrics(summary, request_count, total_tokens, seconds),
        network: system.network,
        host: system.host,
    }
}

fn core_metrics(summary: &RequestSummaryRow, request_count: i64, error_count: i64, circuit_breaker_count: i64, seconds: i64) -> CoreRequestMetrics {
    CoreRequestMetrics {
        request_count,
        qps: rate(request_count, seconds),
        concurrent_requests: summary.concurrent_requests.unwrap_or_default(),
        error_rate: ratio(error_count, request_count),
        timeout_rate: ratio(summary.timeout_count.unwrap_or_default(), request_count),
        rate_limited_count: summary.rate_limited_count.unwrap_or_default(),
        server_error_count: summary.server_error_count.unwrap_or_default(),
        p50_latency_ms: summary.p50_latency_ms,
        p90_latency_ms: summary.p90_latency_ms,
        p95_latency_ms: summary.p95_latency_ms,
        p99_latency_ms: summary.p99_latency_ms,
        p50_first_byte_ms: summary.p50_first_byte_ms,
        p90_first_byte_ms: summary.p90_first_byte_ms,
        p95_first_byte_ms: summary.p95_first_byte_ms,
        p99_first_byte_ms: summary.p99_first_byte_ms,
        p50_response_headers_ms: summary.p50_response_headers_ms,
        p90_response_headers_ms: summary.p90_response_headers_ms,
        p95_response_headers_ms: summary.p95_response_headers_ms,
        p99_response_headers_ms: summary.p99_response_headers_ms,
        p50_first_token_ms: summary.p50_first_token_ms,
        p90_first_token_ms: summary.p90_first_token_ms,
        p95_first_token_ms: summary.p95_first_token_ms,
        p99_first_token_ms: summary.p99_first_token_ms,
        retry_count: summary.retry_count.unwrap_or_default(),
        circuit_breaker_count,
        stream_request_count: summary.stream_request_count.unwrap_or_default(),
    }
}

fn llm_metrics(summary: RequestSummaryRow, request_count: i64, total_tokens: i64, seconds: i64) -> LlmBusinessMetrics {
    let prompt_tokens = summary.prompt_tokens.unwrap_or_default();
    let cache_read_tokens = summary.cache_read_input_tokens.unwrap_or_default();
    LlmBusinessMetrics {
        prompt_tokens,
        completion_tokens: summary.completion_tokens.unwrap_or_default(),
        tokens_per_request: ratio(total_tokens, request_count),
        tokens_per_second: rate(total_tokens, seconds),
        failover_count: summary.failover_count.unwrap_or_default(),
        cache_hit_rate: ratio(cache_read_tokens, prompt_tokens + cache_read_tokens),
        quota_limited_count: summary.quota_limited_count.unwrap_or_default(),
    }
}

fn summary_sql() -> &'static str {
    "WITH metric AS ( \
        SELECT COALESCE(SUM(request_count), 0)::bigint AS request_count, \
            COALESCE(SUM(failed_count), 0)::bigint AS error_count, \
            COALESCE(SUM(active_count), 0)::bigint AS concurrent_requests, \
            COALESCE(SUM(timeout_count), 0)::bigint AS timeout_count, \
            COALESCE(SUM(rate_limited_count), 0)::bigint AS rate_limited_count, \
            COALESCE(SUM(server_error_count), 0)::bigint AS server_error_count, \
            COALESCE(SUM(retry_count), 0)::bigint AS retry_count, \
            COALESCE(SUM(request_count) FILTER (WHERE is_stream IS TRUE), 0)::bigint AS stream_request_count, \
            COALESCE(SUM(prompt_tokens), 0)::bigint AS prompt_tokens, \
            COALESCE(SUM(completion_tokens), 0)::bigint AS completion_tokens, \
            COALESCE(SUM(total_tokens), 0)::bigint AS total_tokens, \
            COALESCE(SUM(failover_count), 0)::bigint AS failover_count, \
            COALESCE(SUM(cache_read_input_tokens), 0)::bigint AS cache_read_input_tokens, \
            COALESCE(SUM(quota_limited_count), 0)::bigint AS quota_limited_count \
        FROM dashboard_request_metric_buckets \
        WHERE source_type = 'request' AND bucket_granularity = $3 AND bucket_started_at >= $1 AND bucket_started_at < $2 \
    ), histogram_raw AS ( \
        SELECT metric_kind, le_ms, COALESCE(SUM(sample_count), 0)::bigint AS bucket_count FROM dashboard_latency_histogram_buckets \
        WHERE source_type = 'request' AND bucket_granularity = $3 AND bucket_started_at >= $1 AND bucket_started_at < $2 \
        GROUP BY metric_kind, le_ms \
    ), histogram AS ( \
        SELECT metric_kind, le_ms, bucket_count, SUM(bucket_count) OVER (PARTITION BY metric_kind ORDER BY le_ms)::bigint AS cumulative_count \
        FROM histogram_raw \
    ), totals AS ( \
        SELECT metric_kind, SUM(bucket_count)::bigint AS total_count FROM histogram GROUP BY metric_kind \
    ) \
    SELECT metric.*, \
        (SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.metric_kind = h.metric_kind WHERE h.metric_kind = 'latency' AND h.cumulative_count >= CEIL(t.total_count::numeric * 0.50)) AS p50_latency_ms, \
        (SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.metric_kind = h.metric_kind WHERE h.metric_kind = 'latency' AND h.cumulative_count >= CEIL(t.total_count::numeric * 0.90)) AS p90_latency_ms, \
        (SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.metric_kind = h.metric_kind WHERE h.metric_kind = 'latency' AND h.cumulative_count >= CEIL(t.total_count::numeric * 0.95)) AS p95_latency_ms, \
        (SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.metric_kind = h.metric_kind WHERE h.metric_kind = 'latency' AND h.cumulative_count >= CEIL(t.total_count::numeric * 0.99)) AS p99_latency_ms, \
        (SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.metric_kind = h.metric_kind WHERE h.metric_kind = 'first_byte' AND h.cumulative_count >= CEIL(t.total_count::numeric * 0.50)) AS p50_first_byte_ms, \
        (SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.metric_kind = h.metric_kind WHERE h.metric_kind = 'first_byte' AND h.cumulative_count >= CEIL(t.total_count::numeric * 0.90)) AS p90_first_byte_ms, \
        (SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.metric_kind = h.metric_kind WHERE h.metric_kind = 'first_byte' AND h.cumulative_count >= CEIL(t.total_count::numeric * 0.95)) AS p95_first_byte_ms, \
        (SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.metric_kind = h.metric_kind WHERE h.metric_kind = 'first_byte' AND h.cumulative_count >= CEIL(t.total_count::numeric * 0.99)) AS p99_first_byte_ms, \
        (SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.metric_kind = h.metric_kind WHERE h.metric_kind = 'response_headers' AND h.cumulative_count >= CEIL(t.total_count::numeric * 0.50)) AS p50_response_headers_ms, \
        (SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.metric_kind = h.metric_kind WHERE h.metric_kind = 'response_headers' AND h.cumulative_count >= CEIL(t.total_count::numeric * 0.90)) AS p90_response_headers_ms, \
        (SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.metric_kind = h.metric_kind WHERE h.metric_kind = 'response_headers' AND h.cumulative_count >= CEIL(t.total_count::numeric * 0.95)) AS p95_response_headers_ms, \
        (SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.metric_kind = h.metric_kind WHERE h.metric_kind = 'response_headers' AND h.cumulative_count >= CEIL(t.total_count::numeric * 0.99)) AS p99_response_headers_ms, \
        (SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.metric_kind = h.metric_kind WHERE h.metric_kind = 'first_token' AND h.cumulative_count >= CEIL(t.total_count::numeric * 0.50)) AS p50_first_token_ms, \
        (SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.metric_kind = h.metric_kind WHERE h.metric_kind = 'first_token' AND h.cumulative_count >= CEIL(t.total_count::numeric * 0.90)) AS p90_first_token_ms, \
        (SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.metric_kind = h.metric_kind WHERE h.metric_kind = 'first_token' AND h.cumulative_count >= CEIL(t.total_count::numeric * 0.95)) AS p95_first_token_ms, \
        (SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.metric_kind = h.metric_kind WHERE h.metric_kind = 'first_token' AND h.cumulative_count >= CEIL(t.total_count::numeric * 0.99)) AS p99_first_token_ms \
    FROM metric"
}

fn time_window_values(window: &SnapshotAggregationWindow) -> Vec<Value> {
    vec![Value::from(window.started_at), Value::from(window.ended_at)]
}

fn window_values(window: &SnapshotAggregationWindow) -> Vec<Value> {
    vec![
        Value::from(window.started_at),
        Value::from(window.ended_at),
        Value::from(window.granularity.as_str().to_owned()),
    ]
}

fn window_duration_seconds(window: &SnapshotAggregationWindow) -> i64 {
    (window.ended_at - window.started_at).whole_seconds().max(1)
}

fn ratio(numerator: i64, denominator: i64) -> f64 {
    if denominator <= 0 {
        return 0.0;
    }
    numerator as f64 / denominator as f64
}

fn rate(count: i64, seconds: i64) -> f64 {
    if seconds <= 0 {
        return 0.0;
    }
    count as f64 / seconds as f64
}

#[derive(Debug, Default, FromQueryResult)]
struct RequestSummaryRow {
    request_count: Option<i64>,
    error_count: Option<i64>,
    concurrent_requests: Option<i64>,
    timeout_count: Option<i64>,
    rate_limited_count: Option<i64>,
    server_error_count: Option<i64>,
    p50_latency_ms: Option<i64>,
    p90_latency_ms: Option<i64>,
    p95_latency_ms: Option<i64>,
    p99_latency_ms: Option<i64>,
    p50_first_byte_ms: Option<i64>,
    p90_first_byte_ms: Option<i64>,
    p95_first_byte_ms: Option<i64>,
    p99_first_byte_ms: Option<i64>,
    p50_response_headers_ms: Option<i64>,
    p90_response_headers_ms: Option<i64>,
    p95_response_headers_ms: Option<i64>,
    p99_response_headers_ms: Option<i64>,
    p50_first_token_ms: Option<i64>,
    p90_first_token_ms: Option<i64>,
    p95_first_token_ms: Option<i64>,
    p99_first_token_ms: Option<i64>,
    retry_count: Option<i64>,
    stream_request_count: Option<i64>,
    prompt_tokens: Option<i64>,
    completion_tokens: Option<i64>,
    total_tokens: Option<i64>,
    failover_count: Option<i64>,
    cache_read_input_tokens: Option<i64>,
    quota_limited_count: Option<i64>,
}

#[cfg(test)]
mod tests {
    use types::performance_monitoring::{HostResourceMetrics, MetricSupportStatus, NetworkConnectionMetrics};

    use super::{RequestSummaryRow, SystemMetricsSnapshot, metrics};

    #[test]
    fn aggregate_metrics_preserves_injected_system_metrics() {
        let system = SystemMetricsSnapshot {
            network: NetworkConnectionMetrics {
                inbound_bytes: 42,
                status: MetricSupportStatus::Ready,
                ..Default::default()
            },
            host: HostResourceMetrics {
                cpu_usage_percent: Some(12.5),
                status: MetricSupportStatus::Ready,
                ..Default::default()
            },
        };

        let result = metrics(RequestSummaryRow::default(), 0, 60, system.clone());

        assert_eq!(result.network, system.network);
        assert_eq!(result.host, system.host);
    }

    #[test]
    fn aggregate_metrics_maps_first_byte_and_p90_fields() {
        let result = metrics(
            RequestSummaryRow {
                request_count: Some(2),
                p90_latency_ms: Some(900),
                p90_first_byte_ms: Some(120),
                p90_response_headers_ms: Some(40),
                p90_first_token_ms: Some(300),
                ..Default::default()
            },
            3,
            60,
            SystemMetricsSnapshot::default(),
        );

        assert_eq!(result.core.p90_latency_ms, Some(900));
        assert_eq!(result.core.p90_first_byte_ms, Some(120));
        assert_eq!(result.core.p90_response_headers_ms, Some(40));
        assert_eq!(result.core.p90_first_token_ms, Some(300));
        assert_eq!(result.core.circuit_breaker_count, 3);
    }
}

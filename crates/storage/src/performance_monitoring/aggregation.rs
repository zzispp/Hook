use rust_decimal::Decimal;
use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, Value};
use types::performance_monitoring::{CoreRequestMetrics, LlmBusinessMetrics, MetricDimension, PerformanceSnapshotMetrics};

use crate::{StorageError, StorageResult};

use super::{PerformanceMonitoringStore, PerformanceSnapshotInput, SnapshotAggregationWindow, SystemMetricsSnapshot};

pub(super) async fn aggregate_window_with_system(
    store: &PerformanceMonitoringStore,
    window: SnapshotAggregationWindow,
    system: SystemMetricsSnapshot,
) -> StorageResult<()> {
    let summary = request_summary(store, &window).await?;
    let dimensions = request_dimensions(store, &window).await?;
    let input = PerformanceSnapshotInput {
        bucket_granularity: window.granularity,
        bucket_started_at: window.started_at,
        bucket_ended_at: window.ended_at,
        metrics: metrics(summary, dimensions, window_duration_seconds(&window), system),
    };
    store.upsert_snapshot(input).await
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

async fn request_dimensions(store: &PerformanceMonitoringStore, window: &SnapshotAggregationWindow) -> StorageResult<RequestDimensions> {
    let model_sql = dimension_sql("COALESCE(model_name_snapshot, global_model_id, 'unknown')");
    let provider_sql = dimension_sql("COALESCE(provider_name_snapshot, provider_id, 'unknown')");
    let models = dimension_rows(store, model_sql, window).await?;
    let providers = dimension_rows(store, provider_sql, window).await?;
    Ok(RequestDimensions { models, providers })
}

async fn dimension_rows(store: &PerformanceMonitoringStore, sql: String, window: &SnapshotAggregationWindow) -> StorageResult<Vec<MetricDimension>> {
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, sql, window_values(window));
    let rows = DimensionRow::find_by_statement(statement).all(store.connection()).await?;
    Ok(rows
        .into_iter()
        .map(|row| MetricDimension {
            name: row.name,
            count: row.count.unwrap_or_default(),
        })
        .collect())
}

fn metrics(summary: RequestSummaryRow, dimensions: RequestDimensions, seconds: i64, system: SystemMetricsSnapshot) -> PerformanceSnapshotMetrics {
    let request_count = summary.request_count.unwrap_or_default();
    let success_count = summary.success_count.unwrap_or_default();
    let error_count = summary.error_count.unwrap_or_default();
    let total_tokens = summary.total_tokens.unwrap_or_default();
    PerformanceSnapshotMetrics {
        core: core_metrics(&summary, request_count, success_count, error_count, seconds),
        llm: llm_metrics(summary, dimensions, request_count, total_tokens, seconds),
        network: system.network,
        host: system.host,
    }
}

fn core_metrics(summary: &RequestSummaryRow, request_count: i64, success_count: i64, error_count: i64, seconds: i64) -> CoreRequestMetrics {
    CoreRequestMetrics {
        request_count,
        qps: rate(request_count, seconds),
        concurrent_requests: summary.concurrent_requests.unwrap_or_default(),
        success_rate: ratio(success_count, request_count),
        error_rate: ratio(error_count, request_count),
        timeout_rate: ratio(summary.timeout_count.unwrap_or_default(), request_count),
        rate_limited_count: summary.rate_limited_count.unwrap_or_default(),
        server_error_count: summary.server_error_count.unwrap_or_default(),
        p50_latency_ms: summary.p50_latency_ms,
        p95_latency_ms: summary.p95_latency_ms,
        p99_latency_ms: summary.p99_latency_ms,
        p50_ttft_ms: summary.p50_ttft_ms,
        p95_ttft_ms: summary.p95_ttft_ms,
        p99_ttft_ms: summary.p99_ttft_ms,
        retry_count: summary.retry_count.unwrap_or_default(),
        circuit_breaker_count: summary.circuit_breaker_count.unwrap_or_default(),
        stream_request_count: summary.stream_request_count.unwrap_or_default(),
    }
}

fn llm_metrics(summary: RequestSummaryRow, dimensions: RequestDimensions, request_count: i64, total_tokens: i64, seconds: i64) -> LlmBusinessMetrics {
    LlmBusinessMetrics {
        prompt_tokens: summary.prompt_tokens.unwrap_or_default(),
        completion_tokens: summary.completion_tokens.unwrap_or_default(),
        total_tokens,
        tokens_per_request: ratio(total_tokens, request_count),
        tokens_per_second: rate(total_tokens, seconds),
        model_distribution: dimensions.models,
        provider_distribution: dimensions.providers,
        failover_count: summary.failover_count.unwrap_or_default(),
        cache_hit_rate: ratio(summary.cache_hit_count.unwrap_or_default(), request_count),
        cost: summary.cost.unwrap_or(Decimal::ZERO),
        quota_limited_count: summary.quota_limited_count.unwrap_or_default(),
    }
}

fn summary_sql() -> &'static str {
    "SELECT \
        COUNT(*)::bigint AS request_count, \
        COUNT(*) FILTER (WHERE status = 'success')::bigint AS success_count, \
        COUNT(*) FILTER (WHERE status IN ('failed', 'cancelled'))::bigint AS error_count, \
        COUNT(*) FILTER (WHERE status IN ('pending', 'streaming'))::bigint AS concurrent_requests, \
        COUNT(*) FILTER (WHERE termination_reason LIKE '%timeout%' OR client_error_type LIKE '%timeout%')::bigint AS timeout_count, \
        COUNT(*) FILTER (WHERE client_status_code = 429 OR client_error_type = 'rate_limit_error')::bigint AS rate_limited_count, \
        COUNT(*) FILTER (WHERE client_status_code >= 500)::bigint AS server_error_count, \
        percentile_disc(0.50) WITHIN GROUP (ORDER BY total_latency_ms) AS p50_latency_ms, \
        percentile_disc(0.95) WITHIN GROUP (ORDER BY total_latency_ms) AS p95_latency_ms, \
        percentile_disc(0.99) WITHIN GROUP (ORDER BY total_latency_ms) AS p99_latency_ms, \
        percentile_disc(0.50) WITHIN GROUP (ORDER BY first_byte_time_ms) AS p50_ttft_ms, \
        percentile_disc(0.95) WITHIN GROUP (ORDER BY first_byte_time_ms) AS p95_ttft_ms, \
        percentile_disc(0.99) WITHIN GROUP (ORDER BY first_byte_time_ms) AS p99_ttft_ms, \
        COUNT(*) FILTER (WHERE has_retry)::bigint AS retry_count, \
        0::bigint AS circuit_breaker_count, \
        COUNT(*) FILTER (WHERE is_stream)::bigint AS stream_request_count, \
        COALESCE(SUM(prompt_tokens), 0)::bigint AS prompt_tokens, \
        COALESCE(SUM(completion_tokens), 0)::bigint AS completion_tokens, \
        COALESCE(SUM(total_tokens), 0)::bigint AS total_tokens, \
        COUNT(*) FILTER (WHERE has_failover)::bigint AS failover_count, \
        COUNT(*) FILTER (WHERE COALESCE(cache_read_input_tokens, 0) > 0)::bigint AS cache_hit_count, \
        COALESCE(SUM(total_cost), 0) AS cost, \
        COUNT(*) FILTER (WHERE client_error_type = 'new_api_error' OR client_error_message LIKE '%quota%')::bigint AS quota_limited_count \
        FROM request_records \
        WHERE created_at >= $1 AND created_at < $2"
}

fn dimension_sql(expression: &str) -> String {
    format!(
        "SELECT {expression} AS name, COUNT(*)::bigint AS count \
        FROM request_records \
        WHERE created_at >= $1 AND created_at < $2 \
        GROUP BY name \
        ORDER BY count DESC, name ASC \
        LIMIT 12"
    )
}

fn window_values(window: &SnapshotAggregationWindow) -> Vec<Value> {
    vec![Value::from(window.started_at), Value::from(window.ended_at)]
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
    success_count: Option<i64>,
    error_count: Option<i64>,
    concurrent_requests: Option<i64>,
    timeout_count: Option<i64>,
    rate_limited_count: Option<i64>,
    server_error_count: Option<i64>,
    p50_latency_ms: Option<i64>,
    p95_latency_ms: Option<i64>,
    p99_latency_ms: Option<i64>,
    p50_ttft_ms: Option<i64>,
    p95_ttft_ms: Option<i64>,
    p99_ttft_ms: Option<i64>,
    retry_count: Option<i64>,
    circuit_breaker_count: Option<i64>,
    stream_request_count: Option<i64>,
    prompt_tokens: Option<i64>,
    completion_tokens: Option<i64>,
    total_tokens: Option<i64>,
    failover_count: Option<i64>,
    cache_hit_count: Option<i64>,
    cost: Option<Decimal>,
    quota_limited_count: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct DimensionRow {
    name: String,
    count: Option<i64>,
}

struct RequestDimensions {
    models: Vec<MetricDimension>,
    providers: Vec<MetricDimension>,
}

#[cfg(test)]
mod tests {
    use types::performance_monitoring::{HostResourceMetrics, MetricSupportStatus, NetworkConnectionMetrics};

    use super::{RequestDimensions, RequestSummaryRow, SystemMetricsSnapshot, metrics};

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

        let result = metrics(RequestSummaryRow::default(), empty_dimensions(), 60, system.clone());

        assert_eq!(result.network, system.network);
        assert_eq!(result.host, system.host);
    }

    fn empty_dimensions() -> RequestDimensions {
        RequestDimensions {
            models: Vec::new(),
            providers: Vec::new(),
        }
    }
}

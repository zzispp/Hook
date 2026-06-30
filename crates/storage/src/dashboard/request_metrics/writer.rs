use rust_decimal::Decimal;
use sea_orm::{ConnectionTrait, DbBackend, Statement, Value};

use crate::{StorageResult, dashboard::scope::SqlParams};

use super::{
    bucket::{BucketBounds, BucketGranularity},
    types::{HistogramContribution, HistogramSample, MetricContribution},
};

const METRIC_COLUMNS: &[&str] = &[
    "id",
    "source_type",
    "bucket_granularity",
    "bucket_started_at",
    "bucket_ended_at",
    "user_id",
    "username",
    "token_id",
    "token_name",
    "token_prefix",
    "provider_id",
    "provider_name",
    "global_model_id",
    "model_name",
    "client_api_format",
    "provider_api_format",
    "is_stream",
    "needs_conversion",
    "request_count",
    "success_count",
    "failed_count",
    "active_count",
    "prompt_tokens",
    "completion_tokens",
    "cache_creation_input_tokens",
    "cache_read_input_tokens",
    "total_tokens",
    "output_tokens",
    "total_cost",
    "base_cost",
    "upstream_total_cost",
    "cache_read_cost",
    "cache_creation_cost",
    "latency_total_ms",
    "latency_sample_count",
    "first_byte_total_ms",
    "first_byte_sample_count",
    "response_headers_total_ms",
    "response_headers_sample_count",
    "first_sse_event_total_ms",
    "first_sse_event_sample_count",
    "first_token_total_ms",
    "first_token_sample_count",
    "sse_to_output_total_ms",
    "sse_to_output_sample_count",
    "tps_latency_total_ms",
    "tps_output_tokens",
    "tps_sample_count",
    "retry_count",
    "failover_count",
    "timeout_count",
    "rate_limited_count",
    "server_error_count",
    "quota_limited_count",
    "slow_request_count",
    "created_at",
    "updated_at",
];

const HISTOGRAM_COLUMNS: &[&str] = &[
    "id",
    "source_type",
    "metric_kind",
    "bucket_granularity",
    "bucket_started_at",
    "bucket_ended_at",
    "le_ms",
    "provider_id",
    "provider_name",
    "global_model_id",
    "provider_api_format",
    "is_stream",
    "needs_conversion",
    "sample_count",
    "created_at",
    "updated_at",
];

pub(super) async fn upsert_metric_delta<C>(connection: &C, metric: &MetricContribution, granularity: BucketGranularity, multiplier: i64) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let bounds = granularity.bounds(metric.created_at);
    let now = time::OffsetDateTime::now_utc();
    let mut params = SqlParams::new();
    let sql = metric_upsert_sql(&mut params, metric, bounds, multiplier, now);
    connection
        .execute_raw(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .await?;
    Ok(())
}

pub(super) async fn upsert_histogram_delta<C>(
    connection: &C,
    histogram: &HistogramContribution,
    granularity: BucketGranularity,
    multiplier: i64,
) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    for sample in histogram.samples() {
        upsert_histogram_sample(connection, histogram, granularity, sample, multiplier).await?;
    }
    Ok(())
}

async fn upsert_histogram_sample<C>(
    connection: &C,
    histogram: &HistogramContribution,
    granularity: BucketGranularity,
    sample: HistogramSample,
    multiplier: i64,
) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let bounds = granularity.bounds(histogram.created_at);
    let now = time::OffsetDateTime::now_utc();
    let mut params = SqlParams::new();
    let sql = histogram_upsert_sql(&mut params, histogram, bounds, sample, multiplier, now);
    connection
        .execute_raw(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .await?;
    Ok(())
}

fn metric_upsert_sql(params: &mut SqlParams, metric: &MetricContribution, bounds: BucketBounds, multiplier: i64, now: time::OffsetDateTime) -> String {
    let insert_values = metric_values(metric, bounds, multiplier, now)
        .into_iter()
        .map(|value| push_value(params, value))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "INSERT INTO dashboard_request_metric_buckets ({}) VALUES ({insert_values}) \
        ON CONFLICT (source_type, bucket_granularity, bucket_started_at, user_id, token_id, provider_id, global_model_id, client_api_format, provider_api_format, is_stream, needs_conversion) \
        DO UPDATE SET {}",
        metric_columns().join(", "),
        metric_update_sql()
    )
}

fn histogram_upsert_sql(
    params: &mut SqlParams,
    histogram: &HistogramContribution,
    bounds: BucketBounds,
    sample: HistogramSample,
    multiplier: i64,
    now: time::OffsetDateTime,
) -> String {
    let insert_values = histogram_values(histogram, bounds, sample, multiplier, now)
        .into_iter()
        .map(|value| push_value(params, value))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "INSERT INTO dashboard_latency_histogram_buckets ({}) VALUES ({insert_values}) \
        ON CONFLICT (source_type, metric_kind, bucket_granularity, bucket_started_at, le_ms, provider_id, global_model_id, provider_api_format, is_stream, needs_conversion) \
        DO UPDATE SET provider_name = COALESCE(EXCLUDED.provider_name, dashboard_latency_histogram_buckets.provider_name), \
        sample_count = dashboard_latency_histogram_buckets.sample_count + EXCLUDED.sample_count, updated_at = EXCLUDED.updated_at",
        histogram_columns().join(", ")
    )
}

fn push_value(params: &mut SqlParams, value: Value) -> String {
    params.values.push(value);
    format!("${}", params.values.len())
}

fn metric_values(metric: &MetricContribution, bounds: BucketBounds, multiplier: i64, now: time::OffsetDateTime) -> Vec<Value> {
    let mut values = metric_identity_values(metric, bounds);
    values.extend(metric_count_values(metric, multiplier));
    values.extend(metric_cost_values(metric, multiplier));
    values.extend(metric_latency_values(metric, multiplier));
    values.extend(metric_event_values(metric, multiplier));
    values.extend([Value::from(now), Value::from(now)]);
    values
}

fn metric_identity_values(metric: &MetricContribution, bounds: BucketBounds) -> Vec<Value> {
    vec![
        Value::from(uuid::Uuid::now_v7().to_string()),
        Value::from(metric.source_type.clone()),
        Value::from(bounds.granularity.to_owned()),
        Value::from(bounds.started_at),
        Value::from(bounds.ended_at),
        Value::from(metric.user_id.clone()),
        Value::from(metric.username.clone()),
        Value::from(metric.token_id.clone()),
        Value::from(metric.token_name.clone()),
        Value::from(metric.token_prefix.clone()),
        Value::from(metric.provider_id.clone()),
        Value::from(metric.provider_name.clone()),
        Value::from(metric.global_model_id.clone()),
        Value::from(metric.model_name.clone()),
        Value::from(metric.client_api_format.clone()),
        Value::from(metric.provider_api_format.clone()),
        Value::from(metric.is_stream),
        Value::from(metric.needs_conversion),
    ]
}

fn metric_count_values(metric: &MetricContribution, multiplier: i64) -> Vec<Value> {
    vec![
        Value::from(metric.request_count * multiplier),
        Value::from(metric.success_count * multiplier),
        Value::from(metric.failed_count * multiplier),
        Value::from(metric.active_count * multiplier),
        Value::from(metric.prompt_tokens * multiplier),
        Value::from(metric.completion_tokens * multiplier),
        Value::from(metric.cache_creation_input_tokens * multiplier),
        Value::from(metric.cache_read_input_tokens * multiplier),
        Value::from(metric.total_tokens * multiplier),
        Value::from(metric.output_tokens * multiplier),
    ]
}

fn metric_cost_values(metric: &MetricContribution, multiplier: i64) -> Vec<Value> {
    let multiplier = Decimal::from(multiplier);
    vec![
        Value::from(metric.total_cost * multiplier),
        Value::from(metric.base_cost * multiplier),
        Value::from(metric.upstream_total_cost * multiplier),
        Value::from(metric.cache_read_cost * multiplier),
        Value::from(metric.cache_creation_cost * multiplier),
    ]
}

fn metric_latency_values(metric: &MetricContribution, multiplier: i64) -> Vec<Value> {
    vec![
        Value::from(metric.latency_total_ms * multiplier),
        Value::from(metric.latency_sample_count * multiplier),
        Value::from(metric.first_byte_total_ms * multiplier),
        Value::from(metric.first_byte_sample_count * multiplier),
        Value::from(metric.response_headers_total_ms * multiplier),
        Value::from(metric.response_headers_sample_count * multiplier),
        Value::from(metric.first_sse_event_total_ms * multiplier),
        Value::from(metric.first_sse_event_sample_count * multiplier),
        Value::from(metric.first_token_total_ms * multiplier),
        Value::from(metric.first_token_sample_count * multiplier),
        Value::from(metric.sse_to_output_total_ms * multiplier),
        Value::from(metric.sse_to_output_sample_count * multiplier),
        Value::from(metric.tps_latency_total_ms * multiplier),
        Value::from(metric.tps_output_tokens * multiplier),
        Value::from(metric.tps_sample_count * multiplier),
    ]
}

fn metric_event_values(metric: &MetricContribution, multiplier: i64) -> Vec<Value> {
    vec![
        Value::from(metric.retry_count * multiplier),
        Value::from(metric.failover_count * multiplier),
        Value::from(metric.timeout_count * multiplier),
        Value::from(metric.rate_limited_count * multiplier),
        Value::from(metric.server_error_count * multiplier),
        Value::from(metric.quota_limited_count * multiplier),
        Value::from(metric.slow_request_count * multiplier),
    ]
}

fn histogram_values(
    histogram: &HistogramContribution,
    bounds: BucketBounds,
    sample: HistogramSample,
    multiplier: i64,
    now: time::OffsetDateTime,
) -> Vec<Value> {
    vec![
        Value::from(uuid::Uuid::now_v7().to_string()),
        Value::from(histogram.source_type.clone()),
        Value::from(sample.metric_kind.to_owned()),
        Value::from(bounds.granularity.to_owned()),
        Value::from(bounds.started_at),
        Value::from(bounds.ended_at),
        Value::from(sample.le_ms),
        Value::from(histogram.provider_id.clone()),
        Value::from(histogram.provider_name.clone()),
        Value::from(histogram.global_model_id.clone()),
        Value::from(histogram.provider_api_format.clone()),
        Value::from(histogram.is_stream),
        Value::from(histogram.needs_conversion),
        Value::from(multiplier),
        Value::from(now),
        Value::from(now),
    ]
}

fn metric_update_sql() -> &'static str {
    "username = COALESCE(EXCLUDED.username, dashboard_request_metric_buckets.username), \
    token_name = COALESCE(EXCLUDED.token_name, dashboard_request_metric_buckets.token_name), \
    token_prefix = COALESCE(EXCLUDED.token_prefix, dashboard_request_metric_buckets.token_prefix), \
    provider_name = COALESCE(EXCLUDED.provider_name, dashboard_request_metric_buckets.provider_name), \
    model_name = COALESCE(EXCLUDED.model_name, dashboard_request_metric_buckets.model_name), \
    request_count = dashboard_request_metric_buckets.request_count + EXCLUDED.request_count, \
    success_count = dashboard_request_metric_buckets.success_count + EXCLUDED.success_count, \
    failed_count = dashboard_request_metric_buckets.failed_count + EXCLUDED.failed_count, \
    active_count = dashboard_request_metric_buckets.active_count + EXCLUDED.active_count, \
    prompt_tokens = dashboard_request_metric_buckets.prompt_tokens + EXCLUDED.prompt_tokens, \
    completion_tokens = dashboard_request_metric_buckets.completion_tokens + EXCLUDED.completion_tokens, \
    cache_creation_input_tokens = dashboard_request_metric_buckets.cache_creation_input_tokens + EXCLUDED.cache_creation_input_tokens, \
    cache_read_input_tokens = dashboard_request_metric_buckets.cache_read_input_tokens + EXCLUDED.cache_read_input_tokens, \
    total_tokens = dashboard_request_metric_buckets.total_tokens + EXCLUDED.total_tokens, \
    output_tokens = dashboard_request_metric_buckets.output_tokens + EXCLUDED.output_tokens, \
    total_cost = dashboard_request_metric_buckets.total_cost + EXCLUDED.total_cost, \
    base_cost = dashboard_request_metric_buckets.base_cost + EXCLUDED.base_cost, \
    upstream_total_cost = dashboard_request_metric_buckets.upstream_total_cost + EXCLUDED.upstream_total_cost, \
    cache_read_cost = dashboard_request_metric_buckets.cache_read_cost + EXCLUDED.cache_read_cost, \
    cache_creation_cost = dashboard_request_metric_buckets.cache_creation_cost + EXCLUDED.cache_creation_cost, \
    latency_total_ms = dashboard_request_metric_buckets.latency_total_ms + EXCLUDED.latency_total_ms, \
    latency_sample_count = dashboard_request_metric_buckets.latency_sample_count + EXCLUDED.latency_sample_count, \
    first_byte_total_ms = dashboard_request_metric_buckets.first_byte_total_ms + EXCLUDED.first_byte_total_ms, \
    first_byte_sample_count = dashboard_request_metric_buckets.first_byte_sample_count + EXCLUDED.first_byte_sample_count, \
    response_headers_total_ms = dashboard_request_metric_buckets.response_headers_total_ms + EXCLUDED.response_headers_total_ms, \
    response_headers_sample_count = dashboard_request_metric_buckets.response_headers_sample_count + EXCLUDED.response_headers_sample_count, \
    first_sse_event_total_ms = dashboard_request_metric_buckets.first_sse_event_total_ms + EXCLUDED.first_sse_event_total_ms, \
    first_sse_event_sample_count = dashboard_request_metric_buckets.first_sse_event_sample_count + EXCLUDED.first_sse_event_sample_count, \
    first_token_total_ms = dashboard_request_metric_buckets.first_token_total_ms + EXCLUDED.first_token_total_ms, \
    first_token_sample_count = dashboard_request_metric_buckets.first_token_sample_count + EXCLUDED.first_token_sample_count, \
    sse_to_output_total_ms = dashboard_request_metric_buckets.sse_to_output_total_ms + EXCLUDED.sse_to_output_total_ms, \
    sse_to_output_sample_count = dashboard_request_metric_buckets.sse_to_output_sample_count + EXCLUDED.sse_to_output_sample_count, \
    tps_latency_total_ms = dashboard_request_metric_buckets.tps_latency_total_ms + EXCLUDED.tps_latency_total_ms, \
    tps_output_tokens = dashboard_request_metric_buckets.tps_output_tokens + EXCLUDED.tps_output_tokens, \
    tps_sample_count = dashboard_request_metric_buckets.tps_sample_count + EXCLUDED.tps_sample_count, \
    retry_count = dashboard_request_metric_buckets.retry_count + EXCLUDED.retry_count, \
    failover_count = dashboard_request_metric_buckets.failover_count + EXCLUDED.failover_count, \
    timeout_count = dashboard_request_metric_buckets.timeout_count + EXCLUDED.timeout_count, \
    rate_limited_count = dashboard_request_metric_buckets.rate_limited_count + EXCLUDED.rate_limited_count, \
    server_error_count = dashboard_request_metric_buckets.server_error_count + EXCLUDED.server_error_count, \
    quota_limited_count = dashboard_request_metric_buckets.quota_limited_count + EXCLUDED.quota_limited_count, \
    slow_request_count = dashboard_request_metric_buckets.slow_request_count + EXCLUDED.slow_request_count, updated_at = EXCLUDED.updated_at"
}

fn metric_columns() -> &'static [&'static str] {
    METRIC_COLUMNS
}

fn histogram_columns() -> &'static [&'static str] {
    HISTOGRAM_COLUMNS
}

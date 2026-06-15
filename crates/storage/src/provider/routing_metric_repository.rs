use rust_decimal::Decimal;
use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, Value};
use types::provider::{RouteIdentity, RoutingMetricSnapshot, RoutingMetricWindow};

use crate::StorageResult;

use super::routing_repository::{RoutingMetricDelta, RoutingMetricRecord};

const BUCKET_GRANULARITY: &str = "minute";

pub(super) async fn upsert_metric_delta<C>(connection: &C, delta: RoutingMetricDelta) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let bounds = minute_bounds(delta.observed_at);
    let now = time::OffsetDateTime::now_utc();
    let mut params = Vec::new();
    let values = metric_values(&delta, bounds, now)
        .into_iter()
        .map(|value| push(&mut params, value))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "INSERT INTO routing_metric_buckets ({}) VALUES ({values}) {}",
        metric_columns().join(", "),
        metric_upsert_sql()
    );
    connection.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres, sql, params)).await?;
    upsert_route_state(connection, delta).await
}

pub(super) async fn list_metrics<C>(connection: &C, window: RoutingMetricWindow) -> StorageResult<Vec<RoutingMetricRecord>>
where
    C: ConnectionTrait,
{
    let since = time::OffsetDateTime::now_utc() - time::Duration::seconds(window.seconds());
    let mut params = Vec::new();
    let sql = format!(
        "{} WHERE bucket_started_at >= {} {}",
        metric_select_sql(),
        push(&mut params, Value::from(since)),
        metric_group_sql()
    );
    let rows = RoutingMetricRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params))
        .all(connection)
        .await?;
    Ok(rows.into_iter().map(RoutingMetricRecord::from).collect())
}

async fn upsert_route_state<C>(connection: &C, delta: RoutingMetricDelta) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let now = time::OffsetDateTime::now_utc();
    let latency = average_decimal(delta.latency_sum_ms, delta.latency_sample_count);
    let ttfb = average_decimal(delta.ttfb_sum_ms, delta.ttfb_sample_count);
    let output_tps = output_tps(&delta);
    let mut params = Vec::new();
    let values = route_state_values(&delta.route, success_rate(&delta), latency, ttfb, output_tps, delta.request_count.max(0), now)
        .into_iter()
        .map(|value| push(&mut params, value))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "INSERT INTO routing_route_states ({}) VALUES ({values}) {}",
        route_state_columns().join(", "),
        route_state_upsert_sql()
    );
    connection.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres, sql, params)).await?;
    Ok(())
}

fn metric_upsert_sql() -> &'static str {
    "ON CONFLICT (bucket_granularity, bucket_started_at, provider_id, key_id, endpoint_id, global_model_id, client_api_format, provider_api_format, is_stream) \
     DO UPDATE SET provider_name = COALESCE(EXCLUDED.provider_name, routing_metric_buckets.provider_name), \
     key_name = COALESCE(EXCLUDED.key_name, routing_metric_buckets.key_name), endpoint_name = COALESCE(EXCLUDED.endpoint_name, routing_metric_buckets.endpoint_name), \
     request_count = routing_metric_buckets.request_count + EXCLUDED.request_count, success_count = routing_metric_buckets.success_count + EXCLUDED.success_count, \
     failure_count = routing_metric_buckets.failure_count + EXCLUDED.failure_count, timeout_count = routing_metric_buckets.timeout_count + EXCLUDED.timeout_count, \
     rate_limited_count = routing_metric_buckets.rate_limited_count + EXCLUDED.rate_limited_count, server_error_count = routing_metric_buckets.server_error_count + EXCLUDED.server_error_count, \
     latency_sum_ms = routing_metric_buckets.latency_sum_ms + EXCLUDED.latency_sum_ms, latency_sample_count = routing_metric_buckets.latency_sample_count + EXCLUDED.latency_sample_count, \
     ttfb_sum_ms = routing_metric_buckets.ttfb_sum_ms + EXCLUDED.ttfb_sum_ms, ttfb_sample_count = routing_metric_buckets.ttfb_sample_count + EXCLUDED.ttfb_sample_count, \
     output_tokens = routing_metric_buckets.output_tokens + EXCLUDED.output_tokens, tps_latency_sum_ms = routing_metric_buckets.tps_latency_sum_ms + EXCLUDED.tps_latency_sum_ms, \
     tps_sample_count = routing_metric_buckets.tps_sample_count + EXCLUDED.tps_sample_count, upstream_total_cost = routing_metric_buckets.upstream_total_cost + EXCLUDED.upstream_total_cost, \
     total_tokens = routing_metric_buckets.total_tokens + EXCLUDED.total_tokens, last_seen_at = EXCLUDED.last_seen_at, updated_at = EXCLUDED.updated_at"
}

fn route_state_upsert_sql() -> &'static str {
    "ON CONFLICT (provider_id, key_id, endpoint_id, global_model_id, client_api_format, provider_api_format, is_stream) \
     DO UPDATE SET ema_success_rate = (routing_route_states.ema_success_rate * 0.8 + EXCLUDED.ema_success_rate * 0.2), \
     ema_latency_ms = COALESCE((routing_route_states.ema_latency_ms * 0.8 + EXCLUDED.ema_latency_ms * 0.2), routing_route_states.ema_latency_ms, EXCLUDED.ema_latency_ms), \
     ema_ttfb_ms = COALESCE((routing_route_states.ema_ttfb_ms * 0.8 + EXCLUDED.ema_ttfb_ms * 0.2), routing_route_states.ema_ttfb_ms, EXCLUDED.ema_ttfb_ms), \
     ema_output_tps = COALESCE((routing_route_states.ema_output_tps * 0.8 + EXCLUDED.ema_output_tps * 0.2), routing_route_states.ema_output_tps, EXCLUDED.ema_output_tps), \
     sample_count = routing_route_states.sample_count + EXCLUDED.sample_count, state = EXCLUDED.state, last_updated_at = EXCLUDED.last_updated_at"
}

fn metric_select_sql() -> &'static str {
    "SELECT provider_id, provider_name, key_id, key_name, endpoint_id, endpoint_name, global_model_id, client_api_format, provider_api_format, is_stream, \
     SUM(request_count)::BIGINT AS request_count, SUM(success_count)::BIGINT AS success_count, SUM(failure_count)::BIGINT AS failure_count, \
     SUM(timeout_count)::BIGINT AS timeout_count, SUM(rate_limited_count)::BIGINT AS rate_limited_count, SUM(server_error_count)::BIGINT AS server_error_count, \
     SUM(latency_sum_ms)::BIGINT AS latency_sum_ms, SUM(latency_sample_count)::BIGINT AS latency_sample_count, SUM(ttfb_sum_ms)::BIGINT AS ttfb_sum_ms, \
     SUM(ttfb_sample_count)::BIGINT AS ttfb_sample_count, SUM(output_tokens)::BIGINT AS output_tokens, SUM(tps_latency_sum_ms)::BIGINT AS tps_latency_sum_ms, \
     SUM(upstream_total_cost) AS upstream_total_cost, SUM(total_tokens)::BIGINT AS total_tokens, MAX(last_seen_at) AS last_seen_at FROM routing_metric_buckets"
}

fn metric_group_sql() -> &'static str {
    "GROUP BY provider_id, provider_name, key_id, key_name, endpoint_id, endpoint_name, global_model_id, client_api_format, provider_api_format, is_stream"
}

fn metric_values(delta: &RoutingMetricDelta, bounds: BucketBounds, now: time::OffsetDateTime) -> Vec<Value> {
    vec![
        Value::from(uuid::Uuid::now_v7().to_string()),
        Value::from(BUCKET_GRANULARITY.to_owned()),
        Value::from(bounds.started_at),
        Value::from(bounds.ended_at),
        Value::from(delta.route.provider_id.clone()),
        Value::from(delta.provider_name.clone()),
        Value::from(delta.route.key_id.clone()),
        Value::from(delta.key_name.clone()),
        Value::from(delta.route.endpoint_id.clone()),
        Value::from(delta.endpoint_name.clone()),
        Value::from(delta.route.global_model_id.clone()),
        Value::from(delta.route.client_api_format.clone()),
        Value::from(delta.route.provider_api_format.clone()),
        Value::from(delta.route.is_stream),
        Value::from(delta.request_count),
        Value::from(delta.success_count),
        Value::from(delta.failure_count),
        Value::from(delta.timeout_count),
        Value::from(delta.rate_limited_count),
        Value::from(delta.server_error_count),
        Value::from(delta.latency_sum_ms),
        Value::from(delta.latency_sample_count),
        Value::from(delta.ttfb_sum_ms),
        Value::from(delta.ttfb_sample_count),
        Value::from(delta.output_tokens),
        Value::from(delta.tps_latency_sum_ms),
        Value::from(delta.tps_sample_count),
        Value::from(delta.upstream_total_cost),
        Value::from(delta.total_tokens),
        Value::from(delta.observed_at),
        Value::from(now),
        Value::from(now),
    ]
}

fn route_state_values(
    route: &RouteIdentity,
    success_rate: Decimal,
    latency: Option<Decimal>,
    ttfb: Option<Decimal>,
    output_tps: Option<Decimal>,
    sample_count: i64,
    now: time::OffsetDateTime,
) -> Vec<Value> {
    vec![
        Value::from(route.provider_id.clone()),
        Value::from(route.key_id.clone()),
        Value::from(route.endpoint_id.clone()),
        Value::from(route.global_model_id.clone()),
        Value::from(route.client_api_format.clone()),
        Value::from(route.provider_api_format.clone()),
        Value::from(route.is_stream),
        Value::from(success_rate),
        Value::from(ttfb),
        Value::from(latency),
        Value::from(output_tps),
        Value::from(Option::<i32>::None),
        Value::from(sample_count),
        Value::from("eligible".to_owned()),
        Value::from(now),
    ]
}

fn metric_columns() -> [&'static str; 32] {
    [
        "id",
        "bucket_granularity",
        "bucket_started_at",
        "bucket_ended_at",
        "provider_id",
        "provider_name",
        "key_id",
        "key_name",
        "endpoint_id",
        "endpoint_name",
        "global_model_id",
        "client_api_format",
        "provider_api_format",
        "is_stream",
        "request_count",
        "success_count",
        "failure_count",
        "timeout_count",
        "rate_limited_count",
        "server_error_count",
        "latency_sum_ms",
        "latency_sample_count",
        "ttfb_sum_ms",
        "ttfb_sample_count",
        "output_tokens",
        "tps_latency_sum_ms",
        "tps_sample_count",
        "upstream_total_cost",
        "total_tokens",
        "last_seen_at",
        "created_at",
        "updated_at",
    ]
}

fn route_state_columns() -> [&'static str; 15] {
    [
        "provider_id",
        "key_id",
        "endpoint_id",
        "global_model_id",
        "client_api_format",
        "provider_api_format",
        "is_stream",
        "ema_success_rate",
        "ema_ttfb_ms",
        "ema_latency_ms",
        "ema_output_tps",
        "learned_rpm_limit",
        "sample_count",
        "state",
        "last_updated_at",
    ]
}

fn success_rate(delta: &RoutingMetricDelta) -> Decimal {
    if delta.request_count <= 0 {
        return Decimal::ZERO;
    }
    Decimal::from(delta.success_count.max(0)) / Decimal::from(delta.request_count.max(1))
}

fn output_tps(delta: &RoutingMetricDelta) -> Option<Decimal> {
    (delta.tps_latency_sum_ms > 0 && delta.output_tokens > 0)
        .then(|| Decimal::from(delta.output_tokens.max(0)) * Decimal::from(1000) / Decimal::from(delta.tps_latency_sum_ms.max(1)))
}

fn average_decimal(sum: i64, count: i64) -> Option<Decimal> {
    (count > 0).then(|| Decimal::from(sum.max(0)) / Decimal::from(count))
}

fn push(params: &mut Vec<Value>, value: Value) -> String {
    params.push(value);
    format!("${}", params.len())
}

#[derive(Clone, Debug, FromQueryResult)]
struct RoutingMetricRow {
    provider_id: String,
    provider_name: Option<String>,
    key_id: String,
    key_name: Option<String>,
    endpoint_id: String,
    endpoint_name: Option<String>,
    global_model_id: String,
    client_api_format: String,
    provider_api_format: String,
    is_stream: bool,
    request_count: i64,
    success_count: i64,
    failure_count: i64,
    timeout_count: i64,
    rate_limited_count: i64,
    server_error_count: i64,
    latency_sum_ms: i64,
    latency_sample_count: i64,
    ttfb_sum_ms: i64,
    ttfb_sample_count: i64,
    output_tokens: i64,
    tps_latency_sum_ms: i64,
    upstream_total_cost: Decimal,
    total_tokens: i64,
    last_seen_at: time::OffsetDateTime,
}

impl From<RoutingMetricRow> for RoutingMetricRecord {
    fn from(row: RoutingMetricRow) -> Self {
        let request_count = u64_value(row.request_count);
        Self {
            route: row.route(),
            provider_name: row.provider_name.clone(),
            key_name: row.key_name.clone(),
            endpoint_name: row.endpoint_name.clone(),
            snapshot: snapshot(&row, request_count),
            last_seen_at: row.last_seen_at,
        }
    }
}

impl RoutingMetricRow {
    fn route(&self) -> RouteIdentity {
        RouteIdentity {
            provider_id: self.provider_id.clone(),
            key_id: self.key_id.clone(),
            endpoint_id: self.endpoint_id.clone(),
            global_model_id: self.global_model_id.clone(),
            client_api_format: self.client_api_format.clone(),
            provider_api_format: self.provider_api_format.clone(),
            is_stream: self.is_stream,
        }
    }
}

fn snapshot(row: &RoutingMetricRow, request_count: u64) -> RoutingMetricSnapshot {
    RoutingMetricSnapshot {
        request_count,
        success_count: u64_value(row.success_count),
        failure_count: u64_value(row.failure_count),
        timeout_count: u64_value(row.timeout_count),
        rate_limited_count: u64_value(row.rate_limited_count),
        server_error_count: u64_value(row.server_error_count),
        latency_avg_ms: average(row.latency_sum_ms, row.latency_sample_count),
        ttfb_avg_ms: average(row.ttfb_sum_ms, row.ttfb_sample_count),
        output_tps: tps(row.output_tokens, row.tps_latency_sum_ms),
        upstream_total_cost: Some(row.upstream_total_cost),
        total_tokens: u64_value(row.total_tokens),
        sample_count: request_count,
        rpm_used: 0,
        rpm_limit: None,
    }
}

#[derive(Clone, Copy)]
struct BucketBounds {
    started_at: time::OffsetDateTime,
    ended_at: time::OffsetDateTime,
}

fn minute_bounds(value: time::OffsetDateTime) -> BucketBounds {
    let timestamp = value.unix_timestamp().div_euclid(60) * 60;
    let started_at = time::OffsetDateTime::from_unix_timestamp(timestamp).expect("minute bucket timestamp must be valid");
    BucketBounds {
        started_at,
        ended_at: started_at + time::Duration::minutes(1),
    }
}

fn average(sum: i64, count: i64) -> Option<f64> {
    (count > 0).then(|| sum.max(0) as f64 / count as f64)
}

fn tps(output_tokens: i64, latency_sum_ms: i64) -> Option<f64> {
    (latency_sum_ms > 0 && output_tokens > 0).then(|| output_tokens as f64 * 1000.0 / latency_sum_ms as f64)
}

fn u64_value(value: i64) -> u64 {
    value.max(0) as u64
}

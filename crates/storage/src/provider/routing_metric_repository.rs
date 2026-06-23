use rust_decimal::Decimal;
use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, Value};
use types::provider::{RouteIdentity, RoutingMetricSnapshot, RoutingMetricWindow};

use crate::StorageResult;

use super::routing_repository::{RoutingMetricDelta, RoutingMetricRecord, normalized_ema_weights};
use sql::{
    average_decimal, metric_columns, metric_group_sql, metric_select_sql, metric_upsert_sql, metric_values, minute_bounds, output_tps, push,
    route_state_columns, route_state_upsert_sql, route_state_values, success_rate,
};

mod sql;

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
    let values = route_state_values(&delta, success_rate(&delta), latency, ttfb, output_tps, delta.request_count.max(0), now)
        .into_iter()
        .map(|value| push(&mut params, value))
        .collect::<Vec<_>>()
        .join(", ");
    let (current_weight, incoming_weight) = normalized_ema_weights(delta.ema_alpha);
    let current_weight_ref = push(&mut params, Value::from(current_weight));
    let incoming_weight_ref = push(&mut params, Value::from(incoming_weight));
    let sql = format!(
        "INSERT INTO routing_route_states ({}) VALUES ({values}) {}",
        route_state_columns().join(", "),
        route_state_upsert_sql(&current_weight_ref, &incoming_weight_ref)
    );
    connection.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres, sql, params)).await?;
    Ok(())
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
    route_config_fingerprint: Option<String>,
    price_config_fingerprint: Option<String>,
    request_count: i64,
    success_count: i64,
    failure_count: i64,
    first_output_success_count: i64,
    first_output_failure_count: i64,
    timeout_count: i64,
    rate_limited_count: i64,
    server_error_count: i64,
    format_conversion_failure_count: i64,
    usage_missing_count: i64,
    stream_abnormal_end_count: i64,
    schema_tool_call_failure_count: i64,
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
            route_config_fingerprint: row.route_config_fingerprint,
            price_config_fingerprint: row.price_config_fingerprint,
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
        first_output_success_count: u64_value(row.first_output_success_count),
        first_output_failure_count: u64_value(row.first_output_failure_count),
        timeout_count: u64_value(row.timeout_count),
        rate_limited_count: u64_value(row.rate_limited_count),
        server_error_count: u64_value(row.server_error_count),
        format_conversion_failure_count: u64_value(row.format_conversion_failure_count),
        usage_missing_count: u64_value(row.usage_missing_count),
        stream_abnormal_end_count: u64_value(row.stream_abnormal_end_count),
        schema_tool_call_failure_count: u64_value(row.schema_tool_call_failure_count),
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

fn average(sum: i64, count: i64) -> Option<f64> {
    (count > 0).then(|| sum.max(0) as f64 / count as f64)
}

fn tps(output_tokens: i64, latency_sum_ms: i64) -> Option<f64> {
    (latency_sum_ms > 0 && output_tokens > 0).then(|| output_tokens as f64 * 1000.0 / latency_sum_ms as f64)
}

fn u64_value(value: i64) -> u64 {
    value.max(0) as u64
}

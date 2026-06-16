use rust_decimal::{Decimal, prelude::ToPrimitive};
use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, Value};
use types::provider::RouteIdentity;

use crate::StorageResult;

use super::routing_repository::{RoutingContextRouteStateDelta, RoutingContextRouteStateRecord};

pub(super) async fn upsert_context_route_state<C>(connection: &C, delta: RoutingContextRouteStateDelta) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let mut params = Vec::new();
    let values = context_state_values(&delta)
        .into_iter()
        .map(|value| push(&mut params, value))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "INSERT INTO routing_context_route_states ({}) VALUES ({values}) {}",
        context_state_columns().join(", "),
        context_state_upsert_sql()
    );
    connection.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres, sql, params)).await?;
    Ok(())
}

pub(super) async fn list_context_route_states<C>(connection: &C) -> StorageResult<Vec<RoutingContextRouteStateRecord>>
where
    C: ConnectionTrait,
{
    let rows = RoutingContextRouteStateRow::find_by_statement(Statement::from_string(DbBackend::Postgres, select_sql().to_owned()))
        .all(connection)
        .await?;
    Ok(rows.into_iter().map(RoutingContextRouteStateRecord::from).collect())
}

fn context_state_upsert_sql() -> &'static str {
    "ON CONFLICT (context_key, provider_id, key_id, endpoint_id, global_model_id, client_api_format, provider_api_format, is_stream, \
     route_config_fingerprint, price_config_fingerprint) \
     DO UPDATE SET sample_count = routing_context_route_states.sample_count + EXCLUDED.sample_count, \
     success_count = routing_context_route_states.success_count + EXCLUDED.success_count, \
     failure_count = routing_context_route_states.failure_count + EXCLUDED.failure_count, \
     ema_success_rate = (routing_context_route_states.ema_success_rate * 0.8 + EXCLUDED.ema_success_rate * 0.2), \
     ema_latency_ms = COALESCE((routing_context_route_states.ema_latency_ms * 0.8 + EXCLUDED.ema_latency_ms * 0.2), \
     routing_context_route_states.ema_latency_ms, EXCLUDED.ema_latency_ms), \
     ema_ttfb_ms = COALESCE((routing_context_route_states.ema_ttfb_ms * 0.8 + EXCLUDED.ema_ttfb_ms * 0.2), \
     routing_context_route_states.ema_ttfb_ms, EXCLUDED.ema_ttfb_ms), \
     ema_output_tps = COALESCE((routing_context_route_states.ema_output_tps * 0.8 + EXCLUDED.ema_output_tps * 0.2), \
     routing_context_route_states.ema_output_tps, EXCLUDED.ema_output_tps), last_updated_at = EXCLUDED.last_updated_at"
}

fn select_sql() -> &'static str {
    "SELECT context_key, provider_id, key_id, endpoint_id, global_model_id, client_api_format, provider_api_format, is_stream, \
     route_config_fingerprint, price_config_fingerprint, sample_count, success_count, failure_count, ema_success_rate, ema_ttfb_ms, ema_latency_ms, \
     ema_output_tps, last_updated_at \
     FROM routing_context_route_states"
}

fn context_state_values(delta: &RoutingContextRouteStateDelta) -> Vec<Value> {
    vec![
        Value::from(delta.context_key.clone()),
        Value::from(delta.route.provider_id.clone()),
        Value::from(delta.route.key_id.clone()),
        Value::from(delta.route.endpoint_id.clone()),
        Value::from(delta.route.global_model_id.clone()),
        Value::from(delta.route.client_api_format.clone()),
        Value::from(delta.route.provider_api_format.clone()),
        Value::from(delta.route.is_stream),
        Value::from(delta.route_config_fingerprint.clone()),
        Value::from(delta.price_config_fingerprint.clone()),
        Value::from(delta.sample_count),
        Value::from(delta.success_count),
        Value::from(delta.failure_count),
        Value::from(success_rate(delta)),
        Value::from(delta.ttfb_ms.map(Decimal::from)),
        Value::from(delta.latency_ms.map(Decimal::from)),
        Value::from(output_tps(delta)),
        Value::from(delta.observed_at),
    ]
}

fn context_state_columns() -> [&'static str; 18] {
    [
        "context_key",
        "provider_id",
        "key_id",
        "endpoint_id",
        "global_model_id",
        "client_api_format",
        "provider_api_format",
        "is_stream",
        "route_config_fingerprint",
        "price_config_fingerprint",
        "sample_count",
        "success_count",
        "failure_count",
        "ema_success_rate",
        "ema_ttfb_ms",
        "ema_latency_ms",
        "ema_output_tps",
        "last_updated_at",
    ]
}

fn success_rate(delta: &RoutingContextRouteStateDelta) -> Decimal {
    if delta.sample_count <= 0 {
        return Decimal::ZERO;
    }
    Decimal::from(delta.success_count.max(0)) / Decimal::from(delta.sample_count.max(1))
}

fn output_tps(delta: &RoutingContextRouteStateDelta) -> Option<Decimal> {
    (delta.tps_latency_ms > 0 && delta.output_tokens > 0)
        .then(|| Decimal::from(delta.output_tokens.max(0)) * Decimal::from(1000) / Decimal::from(delta.tps_latency_ms.max(1)))
}

fn push(params: &mut Vec<Value>, value: Value) -> String {
    params.push(value);
    format!("${}", params.len())
}

#[derive(Clone, Debug, FromQueryResult)]
struct RoutingContextRouteStateRow {
    context_key: String,
    provider_id: String,
    key_id: String,
    endpoint_id: String,
    global_model_id: String,
    client_api_format: String,
    provider_api_format: String,
    is_stream: bool,
    route_config_fingerprint: Option<String>,
    price_config_fingerprint: Option<String>,
    sample_count: i64,
    success_count: i64,
    failure_count: i64,
    ema_success_rate: Decimal,
    ema_ttfb_ms: Option<Decimal>,
    ema_latency_ms: Option<Decimal>,
    ema_output_tps: Option<Decimal>,
    last_updated_at: time::OffsetDateTime,
}

impl From<RoutingContextRouteStateRow> for RoutingContextRouteStateRecord {
    fn from(row: RoutingContextRouteStateRow) -> Self {
        let route = row.route();
        Self {
            context_key: row.context_key,
            route,
            sample_count: row.sample_count.max(0) as u64,
            success_count: row.success_count.max(0) as u64,
            failure_count: row.failure_count.max(0) as u64,
            ema_success_rate: decimal(row.ema_success_rate).unwrap_or_default(),
            ema_ttfb_ms: row.ema_ttfb_ms.and_then(decimal),
            ema_latency_ms: row.ema_latency_ms.and_then(decimal),
            ema_output_tps: row.ema_output_tps.and_then(decimal),
            route_config_fingerprint: row.route_config_fingerprint,
            price_config_fingerprint: row.price_config_fingerprint,
            last_updated_at: row.last_updated_at,
        }
    }
}

impl RoutingContextRouteStateRow {
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

fn decimal(value: Decimal) -> Option<f64> {
    value.to_f64()
}

#[cfg(test)]
mod tests {
    use super::{context_state_columns, context_state_upsert_sql};

    #[test]
    fn context_state_upsert_accumulates_counts_and_updates_ema() {
        let sql = context_state_upsert_sql();

        assert!(sql.contains("sample_count = routing_context_route_states.sample_count + EXCLUDED.sample_count"));
        assert!(sql.contains("success_count = routing_context_route_states.success_count + EXCLUDED.success_count"));
        assert!(sql.contains("ema_success_rate = (routing_context_route_states.ema_success_rate * 0.8"));
        assert!(sql.contains("route_config_fingerprint, price_config_fingerprint)"));
    }

    #[test]
    fn context_state_insert_includes_fingerprints() {
        let columns = context_state_columns();

        assert!(columns.contains(&"route_config_fingerprint"));
        assert!(columns.contains(&"price_config_fingerprint"));
        assert_eq!(columns[0], "context_key");
    }
}

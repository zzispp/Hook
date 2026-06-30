use rust_decimal::{Decimal, prelude::ToPrimitive};
use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, Value};
use types::provider::{ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1, RouteIdentity};

use crate::StorageResult;

use super::routing_repository::RoutingRouteStateRecord;

pub(super) async fn list_route_states_for_routes<C>(connection: &C, routes: &[RouteIdentity]) -> StorageResult<Vec<RoutingRouteStateRecord>>
where
    C: ConnectionTrait,
{
    let Some(statement) = select_statement(routes) else {
        return Ok(Vec::new());
    };
    let rows = RoutingRouteStateRow::find_by_statement(statement).all(connection).await?;
    Ok(rows.into_iter().map(RoutingRouteStateRecord::from).collect())
}

pub(super) async fn list_route_states<C>(connection: &C) -> StorageResult<Vec<RoutingRouteStateRecord>>
where
    C: ConnectionTrait,
{
    let mut params = Vec::new();
    let sql = format!(
        "{} WHERE timing_metric_semantics_version = {}",
        select_sql(),
        push(&mut params, Value::from(ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1))
    );
    let rows = RoutingRouteStateRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params))
        .all(connection)
        .await?;
    Ok(rows.into_iter().map(RoutingRouteStateRecord::from).collect())
}

fn select_statement(routes: &[RouteIdentity]) -> Option<Statement> {
    let mut params = Vec::new();
    let filter = route_filter_sql(routes, &mut params)?;
    let timing = push(&mut params, Value::from(ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1));
    let sql = format!("{} WHERE timing_metric_semantics_version = {timing} AND ({filter})", select_sql());
    Some(Statement::from_sql_and_values(DbBackend::Postgres, sql, params))
}

fn select_sql() -> &'static str {
    "SELECT profile_id, provider_id, key_id, endpoint_id, global_model_id, client_api_format, provider_api_format, is_stream, route_config_fingerprint, price_config_fingerprint, timing_metric_semantics_version, \
     ema_success_rate, ema_first_token_ms, ema_latency_ms, ema_output_tps, sample_count, last_updated_at \
     FROM routing_route_states"
}

fn route_filter_sql(routes: &[RouteIdentity], params: &mut Vec<Value>) -> Option<String> {
    if routes.is_empty() {
        return None;
    }
    Some(routes.iter().map(|route| route_clause(route, params)).collect::<Vec<_>>().join(" OR "))
}

fn route_clause(route: &RouteIdentity, params: &mut Vec<Value>) -> String {
    let provider = push(params, Value::from(route.provider_id.clone()));
    let key = push(params, Value::from(route.key_id.clone()));
    let endpoint = push(params, Value::from(route.endpoint_id.clone()));
    let model = push(params, Value::from(route.global_model_id.clone()));
    let client_format = push(params, Value::from(route.client_api_format.clone()));
    let provider_format = push(params, Value::from(route.provider_api_format.clone()));
    let is_stream = push(params, Value::from(route.is_stream));
    format!(
        "(provider_id = {provider} AND key_id = {key} AND endpoint_id = {endpoint} AND global_model_id = {model} \
         AND client_api_format = {client_format} AND provider_api_format = {provider_format} AND is_stream = {is_stream})"
    )
}

fn push(params: &mut Vec<Value>, value: Value) -> String {
    params.push(value);
    format!("${}", params.len())
}

#[derive(Clone, Debug, FromQueryResult)]
struct RoutingRouteStateRow {
    profile_id: String,
    provider_id: String,
    key_id: String,
    endpoint_id: String,
    global_model_id: String,
    client_api_format: String,
    provider_api_format: String,
    is_stream: bool,
    route_config_fingerprint: Option<String>,
    price_config_fingerprint: Option<String>,
    timing_metric_semantics_version: String,
    ema_success_rate: Decimal,
    ema_first_token_ms: Option<Decimal>,
    ema_latency_ms: Option<Decimal>,
    ema_output_tps: Option<Decimal>,
    sample_count: i64,
    last_updated_at: time::OffsetDateTime,
}

impl From<RoutingRouteStateRow> for RoutingRouteStateRecord {
    fn from(row: RoutingRouteStateRow) -> Self {
        let route = row.route();
        Self {
            profile_id: row.profile_id,
            route,
            timing_metric_semantics_version: row.timing_metric_semantics_version,
            ema_success_rate: decimal(row.ema_success_rate).unwrap_or_default(),
            ema_first_token_ms: row.ema_first_token_ms.and_then(decimal),
            ema_latency_ms: row.ema_latency_ms.and_then(decimal),
            ema_output_tps: row.ema_output_tps.and_then(decimal),
            sample_count: row.sample_count.max(0) as u64,
            route_config_fingerprint: row.route_config_fingerprint,
            price_config_fingerprint: row.price_config_fingerprint,
            last_updated_at: row.last_updated_at,
        }
    }
}

impl RoutingRouteStateRow {
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
    use sea_orm::Value;
    use types::provider::RouteIdentity;

    use super::route_filter_sql;

    #[test]
    fn empty_route_filter_is_none() {
        let mut params = Vec::new();

        let filter = route_filter_sql(&[], &mut params);

        assert_eq!(filter, None);
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn route_filter_matches_every_identity_dimension() {
        let routes = vec![route("provider-a", "key-a", false), route("provider-b", "key-b", true)];
        let mut params = Vec::<Value>::new();

        let filter = route_filter_sql(&routes, &mut params).expect("filter");

        assert!(filter.contains("provider_id = $1"));
        assert!(filter.contains("key_id = $2"));
        assert!(filter.contains("endpoint_id = $3"));
        assert!(filter.contains("global_model_id = $4"));
        assert!(filter.contains("client_api_format = $5"));
        assert!(filter.contains("provider_api_format = $6"));
        assert!(filter.contains("is_stream = $7"));
        assert!(filter.contains("provider_id = $8"));
        assert!(filter.contains("is_stream = $14"));
        assert_eq!(params.len(), 14);
    }

    fn route(provider_id: &str, key_id: &str, is_stream: bool) -> RouteIdentity {
        RouteIdentity {
            provider_id: provider_id.into(),
            key_id: key_id.into(),
            endpoint_id: "endpoint-a".into(),
            global_model_id: "model-a".into(),
            client_api_format: "openai:chat".into(),
            provider_api_format: "openai:chat".into(),
            is_stream,
        }
    }
}

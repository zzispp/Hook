use rust_decimal::Decimal;
use sea_orm::Value;

use crate::provider::routing_repository::RoutingMetricDelta;

const BUCKET_GRANULARITY: &str = "minute";

pub(super) fn metric_upsert_sql() -> &'static str {
    "ON CONFLICT (bucket_granularity, bucket_started_at, provider_id, key_id, endpoint_id, global_model_id, client_api_format, \
     provider_api_format, is_stream, route_config_fingerprint, price_config_fingerprint) \
     DO UPDATE SET provider_name = COALESCE(EXCLUDED.provider_name, routing_metric_buckets.provider_name), \
     key_name = COALESCE(EXCLUDED.key_name, routing_metric_buckets.key_name), endpoint_name = COALESCE(EXCLUDED.endpoint_name, routing_metric_buckets.endpoint_name), \
     request_count = routing_metric_buckets.request_count + EXCLUDED.request_count, success_count = routing_metric_buckets.success_count + EXCLUDED.success_count, \
     failure_count = routing_metric_buckets.failure_count + EXCLUDED.failure_count, timeout_count = routing_metric_buckets.timeout_count + EXCLUDED.timeout_count, \
     rate_limited_count = routing_metric_buckets.rate_limited_count + EXCLUDED.rate_limited_count, server_error_count = routing_metric_buckets.server_error_count + EXCLUDED.server_error_count, \
     format_conversion_failure_count = routing_metric_buckets.format_conversion_failure_count + EXCLUDED.format_conversion_failure_count, \
     usage_missing_count = routing_metric_buckets.usage_missing_count + EXCLUDED.usage_missing_count, \
     stream_abnormal_end_count = routing_metric_buckets.stream_abnormal_end_count + EXCLUDED.stream_abnormal_end_count, \
     schema_tool_call_failure_count = routing_metric_buckets.schema_tool_call_failure_count + EXCLUDED.schema_tool_call_failure_count, \
     latency_sum_ms = routing_metric_buckets.latency_sum_ms + EXCLUDED.latency_sum_ms, latency_sample_count = routing_metric_buckets.latency_sample_count + EXCLUDED.latency_sample_count, \
     ttfb_sum_ms = routing_metric_buckets.ttfb_sum_ms + EXCLUDED.ttfb_sum_ms, ttfb_sample_count = routing_metric_buckets.ttfb_sample_count + EXCLUDED.ttfb_sample_count, \
     output_tokens = routing_metric_buckets.output_tokens + EXCLUDED.output_tokens, tps_latency_sum_ms = routing_metric_buckets.tps_latency_sum_ms + EXCLUDED.tps_latency_sum_ms, \
     tps_sample_count = routing_metric_buckets.tps_sample_count + EXCLUDED.tps_sample_count, upstream_total_cost = routing_metric_buckets.upstream_total_cost + EXCLUDED.upstream_total_cost, \
     total_tokens = routing_metric_buckets.total_tokens + EXCLUDED.total_tokens, last_seen_at = EXCLUDED.last_seen_at, updated_at = EXCLUDED.updated_at"
}

pub(super) fn route_state_upsert_sql(current_weight_ref: &str, incoming_weight_ref: &str) -> String {
    format!(
        "ON CONFLICT (profile_id, provider_id, key_id, endpoint_id, global_model_id, client_api_format, provider_api_format, is_stream, \
         route_config_fingerprint, price_config_fingerprint) \
         DO UPDATE SET ema_success_rate = (routing_route_states.ema_success_rate * {current_weight_ref} + EXCLUDED.ema_success_rate * {incoming_weight_ref}), \
         ema_latency_ms = COALESCE((routing_route_states.ema_latency_ms * {current_weight_ref} + EXCLUDED.ema_latency_ms * {incoming_weight_ref}), routing_route_states.ema_latency_ms, EXCLUDED.ema_latency_ms), \
         ema_ttfb_ms = COALESCE((routing_route_states.ema_ttfb_ms * {current_weight_ref} + EXCLUDED.ema_ttfb_ms * {incoming_weight_ref}), routing_route_states.ema_ttfb_ms, EXCLUDED.ema_ttfb_ms), \
         ema_output_tps = COALESCE((routing_route_states.ema_output_tps * {current_weight_ref} + EXCLUDED.ema_output_tps * {incoming_weight_ref}), routing_route_states.ema_output_tps, EXCLUDED.ema_output_tps), \
         sample_count = routing_route_states.sample_count + EXCLUDED.sample_count, state = EXCLUDED.state, last_updated_at = EXCLUDED.last_updated_at"
    )
}

pub(super) fn metric_select_sql() -> &'static str {
    "SELECT provider_id, provider_name, key_id, key_name, endpoint_id, endpoint_name, global_model_id, client_api_format, provider_api_format, is_stream, \
     route_config_fingerprint, price_config_fingerprint, \
     SUM(request_count)::BIGINT AS request_count, SUM(success_count)::BIGINT AS success_count, SUM(failure_count)::BIGINT AS failure_count, \
     SUM(timeout_count)::BIGINT AS timeout_count, SUM(rate_limited_count)::BIGINT AS rate_limited_count, SUM(server_error_count)::BIGINT AS server_error_count, \
     SUM(format_conversion_failure_count)::BIGINT AS format_conversion_failure_count, SUM(usage_missing_count)::BIGINT AS usage_missing_count, \
     SUM(stream_abnormal_end_count)::BIGINT AS stream_abnormal_end_count, SUM(schema_tool_call_failure_count)::BIGINT AS schema_tool_call_failure_count, \
     SUM(latency_sum_ms)::BIGINT AS latency_sum_ms, SUM(latency_sample_count)::BIGINT AS latency_sample_count, SUM(ttfb_sum_ms)::BIGINT AS ttfb_sum_ms, \
     SUM(ttfb_sample_count)::BIGINT AS ttfb_sample_count, SUM(output_tokens)::BIGINT AS output_tokens, SUM(tps_latency_sum_ms)::BIGINT AS tps_latency_sum_ms, \
     SUM(upstream_total_cost) AS upstream_total_cost, SUM(total_tokens)::BIGINT AS total_tokens, MAX(last_seen_at) AS last_seen_at FROM routing_metric_buckets"
}

pub(super) fn metric_group_sql() -> &'static str {
    "GROUP BY provider_id, provider_name, key_id, key_name, endpoint_id, endpoint_name, global_model_id, client_api_format, provider_api_format, is_stream, \
     route_config_fingerprint, price_config_fingerprint"
}

pub(super) fn metric_values(delta: &RoutingMetricDelta, bounds: BucketBounds, now: time::OffsetDateTime) -> Vec<Value> {
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
        Value::from(delta.format_conversion_failure_count),
        Value::from(delta.usage_missing_count),
        Value::from(delta.stream_abnormal_end_count),
        Value::from(delta.schema_tool_call_failure_count),
        Value::from(delta.latency_sum_ms),
        Value::from(delta.latency_sample_count),
        Value::from(delta.ttfb_sum_ms),
        Value::from(delta.ttfb_sample_count),
        Value::from(delta.output_tokens),
        Value::from(delta.tps_latency_sum_ms),
        Value::from(delta.tps_sample_count),
        Value::from(delta.upstream_total_cost),
        Value::from(delta.total_tokens),
        Value::from(delta.route_config_fingerprint.clone()),
        Value::from(delta.price_config_fingerprint.clone()),
        Value::from(delta.observed_at),
        Value::from(now),
        Value::from(now),
    ]
}

pub(super) fn route_state_values(
    delta: &RoutingMetricDelta,
    success_rate: Decimal,
    latency: Option<Decimal>,
    ttfb: Option<Decimal>,
    output_tps: Option<Decimal>,
    sample_count: i64,
    now: time::OffsetDateTime,
) -> Vec<Value> {
    vec![
        Value::from(delta.profile_id.clone()),
        Value::from(delta.route.provider_id.clone()),
        Value::from(delta.route.key_id.clone()),
        Value::from(delta.route.endpoint_id.clone()),
        Value::from(delta.route.global_model_id.clone()),
        Value::from(delta.route.client_api_format.clone()),
        Value::from(delta.route.provider_api_format.clone()),
        Value::from(delta.route.is_stream),
        Value::from(delta.route_config_fingerprint.clone()),
        Value::from(delta.price_config_fingerprint.clone()),
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

pub(super) fn metric_columns() -> [&'static str; 38] {
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
        "format_conversion_failure_count",
        "usage_missing_count",
        "stream_abnormal_end_count",
        "schema_tool_call_failure_count",
        "latency_sum_ms",
        "latency_sample_count",
        "ttfb_sum_ms",
        "ttfb_sample_count",
        "output_tokens",
        "tps_latency_sum_ms",
        "tps_sample_count",
        "upstream_total_cost",
        "total_tokens",
        "route_config_fingerprint",
        "price_config_fingerprint",
        "last_seen_at",
        "created_at",
        "updated_at",
    ]
}

pub(super) fn route_state_columns() -> [&'static str; 18] {
    [
        "profile_id",
        "provider_id",
        "key_id",
        "endpoint_id",
        "global_model_id",
        "client_api_format",
        "provider_api_format",
        "is_stream",
        "route_config_fingerprint",
        "price_config_fingerprint",
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

pub(super) fn success_rate(delta: &RoutingMetricDelta) -> Decimal {
    if delta.request_count <= 0 {
        return Decimal::ZERO;
    }
    Decimal::from(delta.success_count.max(0)) / Decimal::from(delta.request_count.max(1))
}

pub(super) fn output_tps(delta: &RoutingMetricDelta) -> Option<Decimal> {
    (delta.tps_latency_sum_ms > 0 && delta.output_tokens > 0)
        .then(|| Decimal::from(delta.output_tokens.max(0)) * Decimal::from(1000) / Decimal::from(delta.tps_latency_sum_ms.max(1)))
}

pub(super) fn average_decimal(sum: i64, count: i64) -> Option<Decimal> {
    (count > 0).then(|| Decimal::from(sum.max(0)) / Decimal::from(count))
}

pub(super) fn push(params: &mut Vec<Value>, value: Value) -> String {
    params.push(value);
    format!("${}", params.len())
}

#[derive(Clone, Copy)]
pub(super) struct BucketBounds {
    pub(super) started_at: time::OffsetDateTime,
    pub(super) ended_at: time::OffsetDateTime,
}

pub(super) fn minute_bounds(value: time::OffsetDateTime) -> BucketBounds {
    let timestamp = value.unix_timestamp().div_euclid(60) * 60;
    let started_at = time::OffsetDateTime::from_unix_timestamp(timestamp).expect("minute bucket timestamp must be valid");
    BucketBounds {
        started_at,
        ended_at: started_at + time::Duration::minutes(1),
    }
}

#[cfg(test)]
mod tests {
    use super::{metric_upsert_sql, route_state_upsert_sql};

    #[test]
    fn metric_upsert_conflict_includes_fingerprints() {
        let sql = metric_upsert_sql();

        assert!(sql.contains("route_config_fingerprint, price_config_fingerprint)"));
    }

    #[test]
    fn route_state_upsert_conflict_includes_fingerprints() {
        let sql = route_state_upsert_sql("$19", "$20");

        assert!(sql.contains("profile_id, provider_id"));
        assert!(sql.contains("is_stream, route_config_fingerprint, price_config_fingerprint)"));
        assert!(sql.contains("ema_success_rate * $19 + EXCLUDED.ema_success_rate * $20"));
    }
}

use rust_decimal::Decimal;
use storage::provider::{ProviderStore, RoutingMetricDelta};
use types::provider::{RequestUpstreamCost, RouteIdentity};

use super::{AttemptAuditInput, TokenUsage, billing_runtime::total_tokens};
use crate::llm_proxy::{LlmProxyError, candidate::CandidateSelection, routing::circuit};

pub(super) async fn record_decision_sample(store: &ProviderStore, selection: &CandidateSelection) -> Result<(), LlmProxyError> {
    let (Some(profile_id), Some(profile_version)) = (selection.routing_profile_id, selection.routing_profile_version.as_deref()) else {
        return Ok(());
    };
    let selected = selection.candidates.first().map(|candidate| candidate.trace.route_identity());
    store
        .upsert_routing_decision_sample(
            &selection.request_id,
            profile_id.as_str(),
            profile_version,
            selected.as_ref(),
            &selection.routing_explanations,
        )
        .await?;
    Ok(())
}

pub(super) async fn record_finished_attempt(
    state: &crate::llm_proxy::LlmProxyState,
    store: &ProviderStore,
    input: &AttemptAuditInput,
    upstream_cost: &RequestUpstreamCost,
) -> Result<(), LlmProxyError> {
    if !input.finished {
        return Ok(());
    }
    let route = input.candidate.trace.route_identity();
    store.upsert_routing_metric_delta(metric_delta(input, upstream_cost, route.clone())).await?;
    circuit::record_attempt(state, circuit_event(input, &route)).await
}

fn metric_delta(input: &AttemptAuditInput, upstream_cost: &RequestUpstreamCost, route: RouteIdentity) -> RoutingMetricDelta {
    let usage = input.usage;
    let output_tokens = output_tokens(usage);
    RoutingMetricDelta {
        route,
        provider_name: Some(input.candidate.trace.provider_name_snapshot.clone()),
        key_name: Some(input.candidate.trace.key_name_snapshot.clone()),
        endpoint_name: Some(input.candidate.trace.endpoint_name_snapshot.clone()),
        request_count: 1,
        success_count: success_count(input),
        failure_count: failure_count(input),
        timeout_count: timeout_count(input),
        rate_limited_count: rate_limited_count(input),
        server_error_count: server_error_count(input),
        latency_sum_ms: input.latency_ms.unwrap_or_default().max(0),
        latency_sample_count: sample_count(input.latency_ms),
        ttfb_sum_ms: input.first_byte_time_ms.unwrap_or_default().max(0),
        ttfb_sample_count: sample_count(input.first_byte_time_ms),
        output_tokens,
        tps_latency_sum_ms: tps_latency(input, output_tokens),
        tps_sample_count: sample_count(input.latency_ms).min(output_tokens.signum()),
        upstream_total_cost: upstream_cost.upstream_total_cost.unwrap_or(Decimal::ZERO),
        total_tokens: total_tokens(usage).unwrap_or_default().max(0),
        observed_at: time::OffsetDateTime::now_utc(),
    }
}

fn circuit_event<'a>(input: &'a AttemptAuditInput, route: &'a RouteIdentity) -> circuit::CircuitAttemptEvent<'a> {
    circuit::CircuitAttemptEvent {
        route,
        success: input.status == "success",
        status_code: input.status_code,
        error_type: input.error_type.as_deref(),
        error_code: input.error_code.as_deref(),
        error_message: input.error_message.as_deref(),
        now_seconds: time::OffsetDateTime::now_utc().unix_timestamp(),
    }
}

fn success_count(input: &AttemptAuditInput) -> i64 {
    i64::from(input.status == "success")
}

fn failure_count(input: &AttemptAuditInput) -> i64 {
    i64::from(input.status != "success")
}

fn timeout_count(input: &AttemptAuditInput) -> i64 {
    let error_timeout = input.error_type.as_deref().is_some_and(|value| value.contains("timeout"));
    i64::from(error_timeout || input.status_code == Some(504))
}

fn rate_limited_count(input: &AttemptAuditInput) -> i64 {
    i64::from(input.status_code == Some(429) || input.error_type.as_deref() == Some("provider_key_rate_limit"))
}

fn server_error_count(input: &AttemptAuditInput) -> i64 {
    i64::from(input.status_code.is_some_and(|code| (500..=599).contains(&code)))
}

fn sample_count(value: Option<i64>) -> i64 {
    i64::from(value.is_some_and(|value| value >= 0))
}

fn output_tokens(usage: Option<TokenUsage>) -> i64 {
    let Some(usage) = usage else {
        return 0;
    };
    usage.completion_tokens.or_else(|| output_dimensions(usage)).unwrap_or_default().max(0)
}

fn output_dimensions(usage: TokenUsage) -> Option<i64> {
    Some(usage.output_text_tokens.unwrap_or_default() + usage.output_audio_tokens.unwrap_or_default() + usage.output_image_tokens.unwrap_or_default())
        .filter(|value| *value > 0)
}

fn tps_latency(input: &AttemptAuditInput, output_tokens: i64) -> i64 {
    if output_tokens <= 0 {
        return 0;
    }
    input.latency_ms.unwrap_or_default().max(0)
}

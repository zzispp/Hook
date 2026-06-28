use rust_decimal::Decimal;
use storage::provider::{ProviderStore, RoutingContextRouteStateDelta, RoutingMetricDelta};
use types::model::PatchField;
use types::provider::{RequestUpstreamCost, RouteIdentity};

use super::{AttemptAuditInput, TokenUsage, billing_runtime::total_tokens};
use crate::llm_proxy::{LlmProxyError, candidate::CandidateSelection};

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
    store: &ProviderStore,
    input: &AttemptAuditInput,
    upstream_cost: &RequestUpstreamCost,
) -> Result<(), LlmProxyError> {
    if !input.finished {
        return Ok(());
    }
    let route = input.candidate.trace.route_identity();
    let observed_at = time::OffsetDateTime::now_utc();
    store
        .upsert_routing_metric_delta(metric_delta(input, upstream_cost, route.clone(), observed_at))
        .await?;
    store
        .upsert_routing_context_route_state(context_delta(input, route.clone(), observed_at))
        .await?;
    Ok(())
}

fn metric_delta(input: &AttemptAuditInput, upstream_cost: &RequestUpstreamCost, route: RouteIdentity, observed_at: time::OffsetDateTime) -> RoutingMetricDelta {
    let usage = input.usage;
    let output_tokens = output_tokens(usage);
    RoutingMetricDelta {
        profile_id: input.candidate.trace.routing_profile_id.as_str().to_owned(),
        ema_alpha: input.candidate.trace.routing_profile_ema_alpha,
        route,
        provider_name: Some(input.candidate.trace.provider_name_snapshot.clone()),
        key_name: Some(input.candidate.trace.key_name_snapshot.clone()),
        endpoint_name: Some(input.candidate.trace.endpoint_name_snapshot.clone()),
        route_config_fingerprint: Some(input.candidate.trace.route_config_fingerprint.clone()),
        price_config_fingerprint: Some(input.candidate.trace.price_config_fingerprint.clone()),
        request_count: 1,
        success_count: success_count(input),
        failure_count: failure_count(input),
        first_output_success_count: first_output_success_count(input),
        first_output_failure_count: first_output_failure_count(input),
        timeout_count: timeout_count(input),
        rate_limited_count: rate_limited_count(input),
        server_error_count: server_error_count(input),
        format_conversion_failure_count: format_conversion_failure_count(input),
        usage_missing_count: usage_missing_count(input),
        stream_abnormal_end_count: stream_abnormal_end_count(input),
        schema_tool_call_failure_count: schema_tool_call_failure_count(input),
        latency_sum_ms: input.latency_ms.unwrap_or_default().max(0),
        latency_sample_count: sample_count(input.latency_ms),
        ttfb_sum_ms: effective_first_byte_time_ms(input).unwrap_or_default().max(0),
        ttfb_sample_count: sample_count(effective_first_byte_time_ms(input)),
        output_tokens,
        tps_latency_sum_ms: tps_latency(input, output_tokens),
        tps_sample_count: sample_count(input.latency_ms).min(output_tokens.signum()),
        upstream_total_cost: upstream_cost.upstream_total_cost.unwrap_or(Decimal::ZERO),
        total_tokens: total_tokens(usage).unwrap_or_default().max(0),
        observed_at,
    }
}

fn context_delta(input: &AttemptAuditInput, route: RouteIdentity, observed_at: time::OffsetDateTime) -> RoutingContextRouteStateDelta {
    let output_tokens = output_tokens(input.usage);
    RoutingContextRouteStateDelta {
        profile_id: input.candidate.trace.routing_profile_id.as_str().to_owned(),
        ema_alpha: input.candidate.trace.routing_profile_ema_alpha,
        context_key: input.candidate.trace.routing_context_key.clone(),
        route,
        route_config_fingerprint: Some(input.candidate.trace.route_config_fingerprint.clone()),
        price_config_fingerprint: Some(input.candidate.trace.price_config_fingerprint.clone()),
        sample_count: 1,
        success_count: success_count(input),
        failure_count: failure_count(input),
        first_output_success_count: first_output_success_count(input),
        first_output_failure_count: first_output_failure_count(input),
        latency_ms: input.latency_ms.map(|value| value.max(0)),
        ttfb_ms: effective_first_byte_time_ms(input).map(|value| value.max(0)),
        output_tokens,
        tps_latency_ms: tps_latency(input, output_tokens),
        observed_at,
    }
}

fn success_count(input: &AttemptAuditInput) -> i64 {
    i64::from(input.status == "success")
}

fn failure_count(input: &AttemptAuditInput) -> i64 {
    i64::from(input.status != "success")
}

fn first_output_success_count(input: &AttemptAuditInput) -> i64 {
    if !input.candidate.trace.is_stream {
        return 0;
    }
    i64::from(input.first_output_time_ms.is_some())
}

fn first_output_failure_count(input: &AttemptAuditInput) -> i64 {
    if !input.candidate.trace.is_stream {
        return 0;
    }
    i64::from(input.first_output_time_ms.is_none())
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

fn format_conversion_failure_count(input: &AttemptAuditInput) -> i64 {
    i64::from(matches!(
        input.error_type.as_deref(),
        Some("request_conversion_error" | "response_conversion_error" | "format_conversion_error")
    ))
}

fn usage_missing_count(input: &AttemptAuditInput) -> i64 {
    i64::from(input.status == "success" && total_tokens(input.usage).is_none())
}

fn stream_abnormal_end_count(input: &AttemptAuditInput) -> i64 {
    let abnormal_reason = matches!(&input.stream_end_reason, PatchField::Value(reason) if !normal_stream_end_reason(reason));
    let abnormal_error = matches!(
        input.error_type.as_deref(),
        Some("upstream_incomplete_stream" | "upstream_eof_without_completion" | "stream_idle_timeout" | "first_output_timeout")
    );
    i64::from(input.candidate.trace.is_stream && (abnormal_reason || abnormal_error))
}

fn schema_tool_call_failure_count(input: &AttemptAuditInput) -> i64 {
    i64::from(
        text_has_schema_tool_failure(input.error_type.as_deref())
            || text_has_schema_tool_failure(input.error_code.as_deref())
            || text_has_schema_tool_failure(input.error_message.as_deref()),
    )
}

fn normal_stream_end_reason(value: &str) -> bool {
    matches!(value, "done" | "eof" | "handler_stop")
}

fn text_has_schema_tool_failure(value: Option<&str>) -> bool {
    let Some(value) = value else {
        return false;
    };
    let value = value.to_ascii_lowercase();
    value.contains("schema") || value.contains("tool_call") || value.contains("tool call") || value.contains("custom_tool")
}

fn sample_count(value: Option<i64>) -> i64 {
    i64::from(value.is_some_and(|value| value >= 0))
}

fn effective_first_byte_time_ms(input: &AttemptAuditInput) -> Option<i64> {
    input.first_output_time_ms.or(input.first_byte_time_ms)
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

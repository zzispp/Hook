use rust_decimal::Decimal;

use crate::{dashboard::latency_stage::StageLatencyContribution, provider::record::request_candidates};

use super::{
    common::{clean_optional, positive},
    constants::{SOURCE_CANDIDATE, STATUS_PENDING, STATUS_SCHEDULED, STATUS_STREAMING, STATUS_SUCCESS},
    types::{HistogramContribution, MetricContribution},
};

pub(super) fn is_started(record: &request_candidates::Model) -> bool {
    record.started_at.is_some()
}

pub(super) fn is_success(record: &request_candidates::Model) -> bool {
    record.status == STATUS_SUCCESS && record.status_code.is_none_or(|status| status < 400)
}

pub(super) fn metric(record: &request_candidates::Model, success: bool) -> MetricContribution {
    let output_tokens = output_tokens(record);
    let latency = record.latency_ms.unwrap_or_default().max(0);
    let tps_sample = i64::from(success && record.latency_ms.is_some() && output_tokens > 0);
    let stage = stage_latency(record, success);
    MetricContribution {
        source_type: SOURCE_CANDIDATE.into(),
        created_at: record.created_at,
        user_id: None,
        username: None,
        token_id: clean_optional(record.token_id.clone()),
        token_name: None,
        token_prefix: None,
        provider_id: clean_optional(record.provider_id.clone()),
        provider_name: clean_optional(record.provider_name_snapshot.clone()),
        global_model_id: clean_optional(record.global_model_id.clone()),
        model_name: clean_optional(record.global_model_id.clone()),
        client_api_format: Some(record.client_api_format.clone()),
        provider_api_format: clean_optional(record.provider_api_format.clone()),
        is_stream: Some(record.is_stream),
        needs_conversion: Some(record.needs_conversion),
        request_count: 1,
        success_count: i64::from(success),
        failed_count: i64::from(!success && !is_active(record)),
        active_count: i64::from(is_active(record)),
        prompt_tokens: positive(record.prompt_tokens),
        completion_tokens: positive(record.completion_tokens),
        cache_creation_input_tokens: cache_creation_tokens(record),
        cache_read_input_tokens: positive(record.cache_read_input_tokens),
        total_tokens: total_tokens(record),
        output_tokens,
        total_cost: record.total_cost.unwrap_or(Decimal::ZERO),
        base_cost: record.base_cost.unwrap_or(Decimal::ZERO),
        upstream_total_cost: record.upstream_total_cost.unwrap_or(Decimal::ZERO),
        cache_read_cost: record.cache_read_cost.unwrap_or(Decimal::ZERO),
        cache_creation_cost: record.cache_creation_cost.unwrap_or(Decimal::ZERO),
        latency_total_ms: success.then_some(record.latency_ms).flatten().unwrap_or_default(),
        latency_sample_count: i64::from(success && record.latency_ms.is_some()),
        ttfb_total_ms: success.then_some(record.first_byte_time_ms).flatten().unwrap_or_default(),
        ttfb_sample_count: i64::from(success && record.first_byte_time_ms.is_some()),
        response_headers_total_ms: StageLatencyContribution::total(stage.response_headers_ms),
        response_headers_sample_count: StageLatencyContribution::sample_count(stage.response_headers_ms),
        first_sse_event_total_ms: StageLatencyContribution::total(stage.first_sse_event_ms),
        first_sse_event_sample_count: StageLatencyContribution::sample_count(stage.first_sse_event_ms),
        first_output_total_ms: StageLatencyContribution::total(stage.first_output_ms),
        first_output_sample_count: StageLatencyContribution::sample_count(stage.first_output_ms),
        sse_to_output_total_ms: StageLatencyContribution::total(stage.sse_to_output_ms),
        sse_to_output_sample_count: StageLatencyContribution::sample_count(stage.sse_to_output_ms),
        tps_latency_total_ms: latency * tps_sample,
        tps_output_tokens: output_tokens * tps_sample,
        tps_sample_count: tps_sample,
        retry_count: i64::from(record.retry_index > 0),
        failover_count: 0,
        timeout_count: 0,
        rate_limited_count: 0,
        server_error_count: 0,
        quota_limited_count: 0,
        slow_request_count: 0,
    }
}

fn is_active(record: &request_candidates::Model) -> bool {
    record.status == STATUS_PENDING || record.status == STATUS_SCHEDULED || record.status == STATUS_STREAMING
}

pub(super) fn histogram(record: &request_candidates::Model, success: bool) -> HistogramContribution {
    let stage = stage_latency(record, success);
    HistogramContribution {
        source_type: SOURCE_CANDIDATE.into(),
        created_at: record.created_at,
        provider_id: clean_optional(record.provider_id.clone()),
        provider_name: clean_optional(record.provider_name_snapshot.clone()),
        global_model_id: clean_optional(record.global_model_id.clone()),
        provider_api_format: clean_optional(record.provider_api_format.clone()),
        is_stream: Some(record.is_stream),
        needs_conversion: Some(record.needs_conversion),
        latency_ms: success.then_some(record.latency_ms).flatten(),
        ttfb_ms: success.then_some(record.first_byte_time_ms).flatten(),
        response_headers_ms: stage.response_headers_ms,
        first_sse_event_ms: stage.first_sse_event_ms,
        first_output_ms: stage.first_output_ms,
        sse_to_output_ms: stage.sse_to_output_ms,
    }
}

fn stage_latency(record: &request_candidates::Model, include: bool) -> StageLatencyContribution {
    if include {
        return StageLatencyContribution::new(record.response_headers_time_ms, record.first_sse_event_time_ms, record.first_output_time_ms);
    }
    StageLatencyContribution::default()
}

fn cache_creation_tokens(record: &request_candidates::Model) -> i64 {
    let split_total = positive(record.cache_creation_5m_input_tokens) + positive(record.cache_creation_1h_input_tokens);
    let total = positive(record.cache_creation_input_tokens);
    if total == 0 && split_total > 0 {
        return split_total;
    }
    total
}

fn total_tokens(record: &request_candidates::Model) -> i64 {
    positive(
        record
            .total_tokens
            .or_else(|| Some(positive(record.prompt_tokens) + positive(record.completion_tokens))),
    ) + cache_creation_tokens(record)
        + positive(record.cache_read_input_tokens)
}

fn output_tokens(record: &request_candidates::Model) -> i64 {
    positive(record.completion_tokens)
        .max(positive(record.output_text_tokens))
        .max(positive(record.total_tokens))
}

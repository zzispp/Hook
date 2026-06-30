use rust_decimal::Decimal;

use crate::{
    dashboard::{latency_stage::StageLatencyContribution, token_context},
    provider::record::request_records,
};

use super::{
    common::{clean_optional, is_active_status, is_failed_status, is_terminal_status, positive, request_is_quota_limited, request_is_timeout},
    constants::{SOURCE_REQUEST, STATUS_SUCCESS},
    types::{HistogramContribution, MetricContribution},
};

pub(super) fn metric(record: &request_records::Model) -> MetricContribution {
    let terminal = is_terminal_status(&record.status);
    let latency = terminal.then_some(record.total_latency_ms).flatten();
    let first_byte = terminal.then_some(record.first_byte_time_ms).flatten();
    let stage = stage_latency(record, terminal);
    MetricContribution {
        source_type: SOURCE_REQUEST.into(),
        created_at: record.created_at,
        user_id: clean_optional(record.user_id_snapshot.clone()),
        username: clean_optional(record.username_snapshot.clone()),
        token_id: clean_optional(record.token_id.clone()),
        token_name: clean_optional(record.token_name_snapshot.clone()),
        token_prefix: clean_optional(record.token_prefix_snapshot.clone()),
        provider_id: clean_optional(record.provider_id.clone()),
        provider_name: clean_optional(record.provider_name_snapshot.clone()),
        global_model_id: clean_optional(record.global_model_id.clone()),
        model_name: clean_optional(record.model_name_snapshot.clone()),
        client_api_format: Some(record.client_api_format.clone()),
        provider_api_format: clean_optional(record.provider_api_format.clone()),
        is_stream: Some(record.is_stream),
        needs_conversion: None,
        request_count: 1,
        success_count: i64::from(record.status == STATUS_SUCCESS),
        failed_count: i64::from(is_failed_status(&record.status)),
        active_count: i64::from(is_active_status(&record.status)),
        prompt_tokens: positive(record.prompt_tokens),
        completion_tokens: positive(record.completion_tokens),
        cache_creation_input_tokens: token_context::cache_creation_tokens(record),
        cache_read_input_tokens: token_context::cache_read_tokens(record),
        total_tokens: token_context::total_tokens(record),
        output_tokens: positive(record.completion_tokens),
        total_cost: record.total_cost.unwrap_or(Decimal::ZERO),
        base_cost: record.base_cost.unwrap_or(Decimal::ZERO),
        upstream_total_cost: record.upstream_total_cost.unwrap_or(Decimal::ZERO),
        cache_read_cost: record.cache_read_cost.unwrap_or(Decimal::ZERO),
        cache_creation_cost: record.cache_creation_cost.unwrap_or(Decimal::ZERO),
        latency_total_ms: latency.unwrap_or_default(),
        latency_sample_count: i64::from(latency.is_some()),
        first_byte_total_ms: first_byte.unwrap_or_default(),
        first_byte_sample_count: i64::from(first_byte.is_some()),
        response_headers_total_ms: StageLatencyContribution::total(stage.response_headers_ms),
        response_headers_sample_count: StageLatencyContribution::sample_count(stage.response_headers_ms),
        first_sse_event_total_ms: StageLatencyContribution::total(stage.first_sse_event_ms),
        first_sse_event_sample_count: StageLatencyContribution::sample_count(stage.first_sse_event_ms),
        first_token_total_ms: StageLatencyContribution::total(stage.first_token_ms),
        first_token_sample_count: StageLatencyContribution::sample_count(stage.first_token_ms),
        sse_to_output_total_ms: StageLatencyContribution::total(stage.sse_to_output_ms),
        sse_to_output_sample_count: StageLatencyContribution::sample_count(stage.sse_to_output_ms),
        tps_latency_total_ms: 0,
        tps_output_tokens: 0,
        tps_sample_count: 0,
        retry_count: i64::from(record.has_retry),
        failover_count: i64::from(record.has_failover),
        timeout_count: i64::from(request_is_timeout(record)),
        rate_limited_count: i64::from(record.client_status_code == Some(429) || record.client_error_type.as_deref() == Some("rate_limit_error")),
        server_error_count: i64::from(record.client_status_code.is_some_and(|status| status >= 500)),
        quota_limited_count: i64::from(request_is_quota_limited(record)),
        slow_request_count: 0,
    }
}

pub(super) fn histogram(record: &request_records::Model, success: bool) -> HistogramContribution {
    let stage = stage_latency(record, success);
    HistogramContribution {
        source_type: SOURCE_REQUEST.into(),
        created_at: record.created_at,
        provider_id: clean_optional(record.provider_id.clone()),
        provider_name: clean_optional(record.provider_name_snapshot.clone()),
        global_model_id: clean_optional(record.global_model_id.clone()),
        provider_api_format: clean_optional(record.provider_api_format.clone()),
        is_stream: Some(record.is_stream),
        needs_conversion: None,
        latency_ms: success.then_some(record.total_latency_ms).flatten(),
        first_byte_ms: success.then_some(record.first_byte_time_ms).flatten(),
        response_headers_ms: stage.response_headers_ms,
        first_sse_event_ms: stage.first_sse_event_ms,
        first_token_ms: stage.first_token_ms,
        sse_to_output_ms: stage.sse_to_output_ms,
    }
}

fn stage_latency(record: &request_records::Model, include: bool) -> StageLatencyContribution {
    if include {
        return StageLatencyContribution::new(record.response_headers_time_ms, record.first_sse_event_time_ms, record.first_token_time_ms);
    }
    StageLatencyContribution::default()
}

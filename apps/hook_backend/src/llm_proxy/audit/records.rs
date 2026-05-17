use axum::http::HeaderMap;
use serde_json::Value;
use storage::provider::{
    RequestBillingRecordValues, RequestCandidateRecordInput, RequestCandidateRecordPatch, RequestRecordRecordInput, RequestRecordRecordPatch,
};
use types::model::PatchField;

use super::{AttemptRecordInput, attempt_billing, record_billing, request_billing_status, total_tokens};
use crate::llm_proxy::{
    LlmProxyError,
    candidate::{CandidateSelection, CandidateTrace},
    proxy::capture::{RequestCapture, recorded_headers, recorded_request_body},
    request_record_policy::RequestRecordPolicy,
};

pub(super) fn attempt_patch(
    request_id: &str,
    input: &AttemptRecordInput<'_>,
    policy: &RequestRecordPolicy,
) -> Result<RequestCandidateRecordPatch, LlmProxyError> {
    Ok(RequestCandidateRecordPatch {
        request_id: request_id.to_owned(),
        candidate_index: input.candidate.trace.candidate_index,
        retry_index: input.retry_index,
        status: input.status.to_owned(),
        skip_reason: input.skip_reason.map(str::to_owned),
        status_code: input.status_code,
        prompt_tokens: input.usage.and_then(|usage| usage.prompt_tokens),
        completion_tokens: input.usage.and_then(|usage| usage.completion_tokens),
        total_tokens: input.usage.and_then(|usage| usage.total_tokens),
        cache_creation_input_tokens: input.usage.and_then(|usage| usage.cache_creation_input_tokens),
        cache_read_input_tokens: input.usage.and_then(|usage| usage.cache_read_input_tokens),
        input_text_tokens: input.usage.and_then(|usage| usage.input_text_tokens),
        input_audio_tokens: input.usage.and_then(|usage| usage.input_audio_tokens),
        input_image_tokens: input.usage.and_then(|usage| usage.input_image_tokens),
        output_text_tokens: input.usage.and_then(|usage| usage.output_text_tokens),
        output_audio_tokens: input.usage.and_then(|usage| usage.output_audio_tokens),
        output_image_tokens: input.usage.and_then(|usage| usage.output_image_tokens),
        reasoning_tokens: input.usage.and_then(|usage| usage.reasoning_tokens),
        cache_creation_5m_input_tokens: input.usage.and_then(|usage| usage.cache_creation_5m_input_tokens),
        cache_creation_1h_input_tokens: input.usage.and_then(|usage| usage.cache_creation_1h_input_tokens),
        usage_source: input.usage.and_then(|usage| usage.usage_source.map(str::to_owned)),
        usage_semantic: input.usage.and_then(|usage| usage.usage_semantic.map(str::to_owned)),
        billing: record_billing::billing_values(input.service_tier.clone(), attempt_billing(input).as_ref()),
        latency_ms: input.latency_ms,
        first_byte_time_ms: input.first_byte_time_ms,
        error_type: input.error_type.map(str::to_owned),
        error_message: input.error_message.map(str::to_owned),
        error_code: input.error_code.map(str::to_owned),
        error_param: input.error_param.map(str::to_owned),
        provider_request_headers: header_patch(input.provider_request_headers.clone(), policy)?,
        provider_request_body: request_body_patch(input.provider_request_body.clone(), policy)?,
        provider_response_headers: header_patch(input.provider_response_headers.clone(), policy)?,
        provider_response_body: response_body_patch(input.provider_response_body.clone(), policy)?,
        finished: input.finished,
    })
}

pub(super) fn attempt_input(
    request_id: &str,
    input: &AttemptRecordInput<'_>,
    policy: &RequestRecordPolicy,
) -> Result<RequestCandidateRecordInput, LlmProxyError> {
    let mut record = base_input(request_id, &input.candidate.trace, input.retry_index, input.status, true, input.finished);
    record.status_code = input.status_code;
    record.skip_reason = input.skip_reason.map(str::to_owned);
    record.prompt_tokens = input.usage.and_then(|usage| usage.prompt_tokens);
    record.completion_tokens = input.usage.and_then(|usage| usage.completion_tokens);
    record.total_tokens = input.usage.and_then(|usage| usage.total_tokens);
    record.cache_creation_input_tokens = input.usage.and_then(|usage| usage.cache_creation_input_tokens);
    record.cache_read_input_tokens = input.usage.and_then(|usage| usage.cache_read_input_tokens);
    record.input_text_tokens = input.usage.and_then(|usage| usage.input_text_tokens);
    record.input_audio_tokens = input.usage.and_then(|usage| usage.input_audio_tokens);
    record.input_image_tokens = input.usage.and_then(|usage| usage.input_image_tokens);
    record.output_text_tokens = input.usage.and_then(|usage| usage.output_text_tokens);
    record.output_audio_tokens = input.usage.and_then(|usage| usage.output_audio_tokens);
    record.output_image_tokens = input.usage.and_then(|usage| usage.output_image_tokens);
    record.reasoning_tokens = input.usage.and_then(|usage| usage.reasoning_tokens);
    record.cache_creation_5m_input_tokens = input.usage.and_then(|usage| usage.cache_creation_5m_input_tokens);
    record.cache_creation_1h_input_tokens = input.usage.and_then(|usage| usage.cache_creation_1h_input_tokens);
    record.usage_source = input.usage.and_then(|usage| usage.usage_source.map(str::to_owned));
    record.usage_semantic = input.usage.and_then(|usage| usage.usage_semantic.map(str::to_owned));
    record.billing = record_billing::billing_values(input.service_tier.clone(), attempt_billing(input).as_ref());
    record.latency_ms = input.latency_ms;
    record.first_byte_time_ms = input.first_byte_time_ms;
    record.error_type = input.error_type.map(str::to_owned);
    record.error_message = input.error_message.map(str::to_owned);
    record.error_code = input.error_code.map(str::to_owned);
    record.error_param = input.error_param.map(str::to_owned);
    record.provider_request_headers = header_input(input.provider_request_headers.clone(), policy)?;
    record.provider_request_body = request_body_input(input.provider_request_body.clone(), policy)?;
    record.provider_response_headers = header_input(input.provider_response_headers.clone(), policy)?;
    record.provider_response_body = response_body_input(input.provider_response_body.clone(), policy)?;
    Ok(record)
}

pub(super) fn scheduled_input(request_id: &str, trace: &CandidateTrace, retry_index: i32) -> RequestCandidateRecordInput {
    base_input(request_id, trace, retry_index, "scheduled", false, false)
}

pub(super) fn request_record_input(
    selection: &CandidateSelection,
    capture: &RequestCapture,
    policy: &RequestRecordPolicy,
) -> Result<RequestRecordRecordInput, LlmProxyError> {
    let primary = selection
        .candidates
        .first()
        .ok_or_else(|| LlmProxyError::Infrastructure("candidate selection must include at least one candidate".into()))?;
    Ok(RequestRecordRecordInput {
        request_id: selection.request_id.clone(),
        token_id: primary.trace.token_id.clone(),
        user_id_snapshot: primary.trace.user_id_snapshot.clone(),
        username_snapshot: primary.trace.username_snapshot.clone(),
        token_name_snapshot: primary.trace.token_name_snapshot.clone(),
        token_prefix_snapshot: primary.trace.token_prefix_snapshot.clone(),
        group_code: primary.trace.group_code.clone(),
        global_model_id: Some(primary.trace.global_model_id.clone()),
        model_name_snapshot: Some(primary.trace.model_name_snapshot.clone()),
        provider_id: Some(primary.trace.provider_id.clone()),
        provider_name_snapshot: Some(primary.trace.provider_name_snapshot.clone()),
        endpoint_id: Some(primary.trace.endpoint_id.clone()),
        key_id: Some(primary.trace.key_id.clone()),
        provider_key_name_snapshot: Some(primary.trace.key_name_snapshot.clone()),
        provider_key_preview_snapshot: Some(primary.trace.key_preview_snapshot.clone()),
        client_api_format: primary.trace.client_api_format.clone(),
        provider_api_format: Some(primary.trace.provider_api_format.clone()),
        request_type: "chat".into(),
        is_stream: primary.trace.is_stream,
        has_failover: false,
        has_retry: false,
        status: "pending".into(),
        billing_status: "pending".into(),
        billing: RequestBillingRecordValues {
            service_tier: capture.service_tier(),
            ..RequestBillingRecordValues::default()
        },
        candidate_count: selection.candidates.len().try_into().unwrap_or(i64::MAX),
        request_headers: capture.request_headers(policy),
        request_body: capture.request_body(policy).map_err(|error| LlmProxyError::Infrastructure(error.to_string()))?,
    })
}

pub(super) fn request_record_patch(
    request_id: &str,
    input: &AttemptRecordInput<'_>,
    policy: &RequestRecordPolicy,
) -> Result<RequestRecordRecordPatch, LlmProxyError> {
    Ok(RequestRecordRecordPatch {
        request_id: request_id.to_owned(),
        provider_id: Some(input.candidate.trace.provider_id.clone()),
        provider_name_snapshot: Some(input.candidate.trace.provider_name_snapshot.clone()),
        endpoint_id: Some(input.candidate.trace.endpoint_id.clone()),
        key_id: Some(input.candidate.trace.key_id.clone()),
        provider_key_name_snapshot: Some(input.candidate.trace.key_name_snapshot.clone()),
        provider_key_preview_snapshot: Some(input.candidate.trace.key_preview_snapshot.clone()),
        provider_api_format: Some(input.candidate.trace.provider_api_format.clone()),
        is_stream: Some(input.candidate.trace.is_stream),
        has_failover: Some(input.candidate.trace.candidate_index > 0),
        has_retry: Some(input.retry_index > 0),
        status: input.status.to_owned(),
        billing_status: request_billing_status(input).into(),
        client_status_code: option_patch(input.status_code),
        client_error_type: option_str_patch(input.error_type),
        client_error_message: option_str_patch(input.error_message),
        termination_origin: input.termination_origin.clone(),
        termination_reason: input.termination_reason.clone(),
        stream_end_reason: input.stream_end_reason.clone(),
        prompt_tokens: option_patch(input.usage.and_then(|usage| usage.prompt_tokens)),
        completion_tokens: option_patch(input.usage.and_then(|usage| usage.completion_tokens)),
        total_tokens: option_patch(total_tokens(input.usage)),
        cache_creation_input_tokens: option_patch(input.usage.and_then(|usage| usage.cache_creation_input_tokens)),
        cache_read_input_tokens: option_patch(input.usage.and_then(|usage| usage.cache_read_input_tokens)),
        input_text_tokens: option_patch(input.usage.and_then(|usage| usage.input_text_tokens)),
        input_audio_tokens: option_patch(input.usage.and_then(|usage| usage.input_audio_tokens)),
        input_image_tokens: option_patch(input.usage.and_then(|usage| usage.input_image_tokens)),
        output_text_tokens: option_patch(input.usage.and_then(|usage| usage.output_text_tokens)),
        output_audio_tokens: option_patch(input.usage.and_then(|usage| usage.output_audio_tokens)),
        output_image_tokens: option_patch(input.usage.and_then(|usage| usage.output_image_tokens)),
        reasoning_tokens: option_patch(input.usage.and_then(|usage| usage.reasoning_tokens)),
        cache_creation_5m_input_tokens: option_patch(input.usage.and_then(|usage| usage.cache_creation_5m_input_tokens)),
        cache_creation_1h_input_tokens: option_patch(input.usage.and_then(|usage| usage.cache_creation_1h_input_tokens)),
        usage_source: option_patch(input.usage.and_then(|usage| usage.usage_source.map(str::to_owned))),
        usage_semantic: option_patch(input.usage.and_then(|usage| usage.usage_semantic.map(str::to_owned))),
        billing: record_billing::billing_patch(attempt_billing(input).as_ref()),
        first_byte_time_ms: option_patch(input.first_byte_time_ms),
        total_latency_ms: option_patch(input.latency_ms),
        client_response_headers: header_patch(input.client_response_headers.clone(), policy)?,
        client_response_body: response_body_patch(input.client_response_body.clone(), policy)?,
        started: true,
        finished: input.finished,
    })
}

fn base_input(request_id: &str, trace: &CandidateTrace, retry_index: i32, status: &str, started: bool, finished: bool) -> RequestCandidateRecordInput {
    RequestCandidateRecordInput {
        request_id: request_id.to_owned(),
        token_id: trace.token_id.clone(),
        group_code: trace.group_code.clone(),
        global_model_id: Some(trace.global_model_id.clone()),
        provider_id: Some(trace.provider_id.clone()),
        provider_name_snapshot: Some(trace.provider_name_snapshot.clone()),
        endpoint_id: Some(trace.endpoint_id.clone()),
        endpoint_name_snapshot: Some(trace.endpoint_name_snapshot.clone()),
        key_id: Some(trace.key_id.clone()),
        key_name_snapshot: Some(trace.key_name_snapshot.clone()),
        key_preview_snapshot: Some(trace.key_preview_snapshot.clone()),
        client_api_format: trace.client_api_format.clone(),
        provider_api_format: Some(trace.provider_api_format.clone()),
        needs_conversion: trace.needs_conversion,
        is_stream: trace.is_stream,
        provider_request_headers: None,
        provider_request_body: None,
        provider_response_headers: None,
        provider_response_body: None,
        candidate_index: trace.candidate_index,
        retry_index,
        status: status.to_owned(),
        skip_reason: None,
        status_code: None,
        prompt_tokens: None,
        completion_tokens: None,
        total_tokens: None,
        cache_creation_input_tokens: None,
        cache_read_input_tokens: None,
        input_text_tokens: None,
        input_audio_tokens: None,
        input_image_tokens: None,
        output_text_tokens: None,
        output_audio_tokens: None,
        output_image_tokens: None,
        reasoning_tokens: None,
        cache_creation_5m_input_tokens: None,
        cache_creation_1h_input_tokens: None,
        usage_source: None,
        usage_semantic: None,
        billing: RequestBillingRecordValues::default(),
        latency_ms: None,
        first_byte_time_ms: None,
        error_type: None,
        error_message: None,
        error_code: None,
        error_param: None,
        started,
        finished,
    }
}

fn header_patch(headers: PatchField<HeaderMap>, policy: &RequestRecordPolicy) -> Result<PatchField<Value>, LlmProxyError> {
    match headers {
        PatchField::Value(headers) => Ok(option_patch(recorded_headers(&headers, policy))),
        PatchField::Null => Ok(PatchField::Null),
        PatchField::Missing => Ok(PatchField::Missing),
    }
}

fn header_input(headers: PatchField<HeaderMap>, policy: &RequestRecordPolicy) -> Result<Option<Value>, LlmProxyError> {
    match headers {
        PatchField::Value(headers) => Ok(recorded_headers(&headers, policy)),
        PatchField::Null | PatchField::Missing => Ok(None),
    }
}

fn request_body_patch(body: PatchField<Value>, policy: &RequestRecordPolicy) -> Result<PatchField<Value>, LlmProxyError> {
    match body {
        PatchField::Value(body) => Ok(option_patch(recorded_request_body(&body, policy).map_err(infra_error)?)),
        PatchField::Null => Ok(PatchField::Null),
        PatchField::Missing => Ok(PatchField::Missing),
    }
}

fn request_body_input(body: PatchField<Value>, policy: &RequestRecordPolicy) -> Result<Option<Value>, LlmProxyError> {
    match body {
        PatchField::Value(body) => recorded_request_body(&body, policy).map_err(infra_error),
        PatchField::Null | PatchField::Missing => Ok(None),
    }
}

fn response_body_patch(body: PatchField<Value>, policy: &RequestRecordPolicy) -> Result<PatchField<Value>, LlmProxyError> {
    match body {
        PatchField::Value(body) => Ok(option_patch(policy.response_body(Some(body)).map_err(infra_error)?)),
        PatchField::Null => Ok(PatchField::Null),
        PatchField::Missing => Ok(PatchField::Missing),
    }
}

fn response_body_input(body: PatchField<Value>, policy: &RequestRecordPolicy) -> Result<Option<Value>, LlmProxyError> {
    match body {
        PatchField::Value(body) => policy.response_body(Some(body)).map_err(infra_error),
        PatchField::Null | PatchField::Missing => Ok(None),
    }
}

fn option_str_patch(value: Option<&str>) -> PatchField<String> {
    option_patch(value.map(str::to_owned))
}

fn option_patch<T>(value: Option<T>) -> PatchField<T> {
    match value {
        Some(value) => PatchField::Value(value),
        None => PatchField::Null,
    }
}

fn infra_error(error: serde_json::Error) -> LlmProxyError {
    LlmProxyError::Infrastructure(error.to_string())
}

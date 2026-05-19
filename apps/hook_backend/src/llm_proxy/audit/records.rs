mod payload;
mod usage_fields;

use serde_json::Value;
use storage::provider::{
    RequestBillingRecordValues, RequestCandidateRecordInput, RequestCandidateRecordPatch, RequestRecordRecordInput, RequestRecordRecordPatch,
};
use types::model::PatchField;

use super::{AttemptRecordInput, BillingAttempt, record_billing, request_billing_status};
use crate::llm_proxy::{
    LlmProxyError,
    candidate::{CandidateSelection, CandidateTrace},
    proxy::capture::RequestCapture,
    request_record_policy::RequestRecordPolicies,
};

pub(super) fn attempt_patch(
    request_id: &str,
    input: &AttemptRecordInput<'_>,
    billing: Option<&BillingAttempt>,
    policies: &RequestRecordPolicies,
) -> Result<RequestCandidateRecordPatch, LlmProxyError> {
    let policy = policies.provider();
    let mut patch = RequestCandidateRecordPatch {
        request_id: request_id.to_owned(),
        candidate_index: input.candidate.trace.candidate_index,
        retry_index: input.retry_index,
        status: input.status.to_owned(),
        skip_reason: input.skip_reason.map(str::to_owned),
        status_code: input.status_code,
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
        billing: record_billing::billing_values(input.service_tier.clone(), billing.map(|item| &item.amount)),
        billing_snapshot: billing_snapshot_patch(billing),
        latency_ms: input.latency_ms,
        first_byte_time_ms: input.first_byte_time_ms,
        error_type: input.error_type.map(str::to_owned),
        error_message: input.error_message.map(str::to_owned),
        error_code: input.error_code.map(str::to_owned),
        error_param: input.error_param.map(str::to_owned),
        provider_request_headers: payload::header_patch(input.provider_request_headers.clone(), policy)?,
        provider_request_body: payload::request_body_patch(input.provider_request_body.clone(), policy)?,
        provider_response_headers: payload::header_patch(input.provider_response_headers.clone(), policy)?,
        provider_response_body: payload::response_body_patch(input.provider_response_body.clone(), policy)?,
        finished: input.finished,
    };
    usage_fields::candidate_patch(input, &mut patch);
    Ok(patch)
}

pub(super) fn attempt_input(
    request_id: &str,
    input: &AttemptRecordInput<'_>,
    billing: Option<&BillingAttempt>,
    policies: &RequestRecordPolicies,
) -> Result<RequestCandidateRecordInput, LlmProxyError> {
    let policy = policies.provider();
    let mut record = base_input(request_id, &input.candidate.trace, input.retry_index, input.status, true, input.finished);
    record.status_code = input.status_code;
    record.skip_reason = input.skip_reason.map(str::to_owned);
    usage_fields::candidate_input(input, &mut record);
    record.billing = record_billing::billing_values(input.service_tier.clone(), billing.map(|item| &item.amount));
    record.billing_snapshot = billing.map(|item| item.snapshot.clone());
    record.latency_ms = input.latency_ms;
    record.first_byte_time_ms = input.first_byte_time_ms;
    record.error_type = input.error_type.map(str::to_owned);
    record.error_message = input.error_message.map(str::to_owned);
    record.error_code = input.error_code.map(str::to_owned);
    record.error_param = input.error_param.map(str::to_owned);
    record.provider_request_headers = payload::header_input(input.provider_request_headers.clone(), policy);
    record.provider_request_body = payload::request_body_input(input.provider_request_body.clone(), policy)?;
    record.provider_response_headers = payload::header_input(input.provider_response_headers.clone(), policy);
    record.provider_response_body = payload::response_body_input(input.provider_response_body.clone(), policy)?;
    Ok(record)
}

pub(super) fn scheduled_input(request_id: &str, trace: &CandidateTrace, retry_index: i32) -> RequestCandidateRecordInput {
    base_input(request_id, trace, retry_index, "scheduled", false, false)
}

pub(super) fn request_record_input(
    selection: &CandidateSelection,
    capture: &RequestCapture,
    policies: &RequestRecordPolicies,
) -> Result<RequestRecordRecordInput, LlmProxyError> {
    let policy = policies.client();
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
        billing_snapshot: None,
        candidate_count: selection.candidates.len().try_into().unwrap_or(i64::MAX),
        request_headers: capture.request_headers(policy),
        request_body: capture.request_body(policy).map_err(|error| LlmProxyError::Infrastructure(error.to_string()))?,
    })
}

pub(super) fn request_record_patch(
    request_id: &str,
    input: &AttemptRecordInput<'_>,
    billing: Option<&BillingAttempt>,
    policies: &RequestRecordPolicies,
) -> Result<RequestRecordRecordPatch, LlmProxyError> {
    let policy = policies.client();
    let mut patch = RequestRecordRecordPatch {
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
        billing_status: request_billing_status(input, billing).into(),
        client_status_code: option_patch(input.status_code),
        client_error_type: option_str_patch(input.error_type),
        client_error_message: option_str_patch(input.error_message),
        termination_origin: input.termination_origin.clone(),
        termination_reason: input.termination_reason.clone(),
        stream_end_reason: input.stream_end_reason.clone(),
        prompt_tokens: PatchField::Missing,
        completion_tokens: PatchField::Missing,
        total_tokens: PatchField::Missing,
        cache_creation_input_tokens: PatchField::Missing,
        cache_read_input_tokens: PatchField::Missing,
        input_text_tokens: PatchField::Missing,
        input_audio_tokens: PatchField::Missing,
        input_image_tokens: PatchField::Missing,
        output_text_tokens: PatchField::Missing,
        output_audio_tokens: PatchField::Missing,
        output_image_tokens: PatchField::Missing,
        reasoning_tokens: PatchField::Missing,
        cache_creation_5m_input_tokens: PatchField::Missing,
        cache_creation_1h_input_tokens: PatchField::Missing,
        usage_source: PatchField::Missing,
        usage_semantic: PatchField::Missing,
        billing: record_billing::billing_patch(billing.map(|item| &item.amount)),
        billing_snapshot: billing_snapshot_patch(billing),
        first_byte_time_ms: option_patch(input.first_byte_time_ms),
        total_latency_ms: option_patch(input.latency_ms),
        client_response_headers: payload::header_patch(input.client_response_headers.clone(), policy)?,
        client_response_body: payload::response_body_patch(input.client_response_body.clone(), policy)?,
        started: true,
        finished: input.finished,
    };
    usage_fields::request_patch(input, &mut patch);
    Ok(patch)
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
        is_cached: trace.is_cached,
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
        billing_snapshot: None,
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

fn option_str_patch(value: Option<&str>) -> PatchField<String> {
    option_patch(value.map(str::to_owned))
}

fn option_patch<T>(value: Option<T>) -> PatchField<T> {
    match value {
        Some(value) => PatchField::Value(value),
        None => PatchField::Null,
    }
}

fn billing_snapshot_patch(billing: Option<&BillingAttempt>) -> PatchField<Value> {
    match billing {
        Some(billing) => PatchField::Value(billing.snapshot.clone()),
        None => PatchField::Missing,
    }
}

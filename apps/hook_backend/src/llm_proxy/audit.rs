use axum::http::HeaderMap;
use provider::application::billing::{RequestBillingInput, calculate_request_billing};
use serde_json::Value;
use storage::{
    StorageError,
    api_token::ApiTokenUsageRecord,
    provider::{ProviderStore, RequestCandidateRecordInput, RequestCandidateRecordPatch, RequestRecordRecordInput, RequestRecordRecordPatch},
    setting::SettingStore,
};
use time::OffsetDateTime;
use types::model::PatchField;

use super::{
    LlmProxyError, LlmProxyState,
    candidate::{CandidateSelection, CandidateTrace, ProxyCandidate},
    proxy::capture::{RequestCapture, recorded_headers, recorded_request_body},
    request_record_policy::RequestRecordPolicy,
};

pub const SKIP_REASON_REQUEST_TERMINATED: &str = "request_terminated_before_attempt";

#[derive(Clone, Copy, Debug, Default)]
pub struct TokenUsage {
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
}

pub async fn record_scheduled_candidates(state: &LlmProxyState, selection: &CandidateSelection, capture: &RequestCapture) -> Result<(), LlmProxyError> {
    let policy = request_record_policy(state).await?;
    create_request_record(state, request_record_input(selection, capture, &policy)?).await?;
    for candidate in &selection.candidates {
        create_record(state, scheduled_input(&selection.request_id, &candidate.trace, 0)).await?;
    }
    Ok(())
}

pub async fn record_attempt(state: &LlmProxyState, request_id: &str, input: AttemptRecordInput<'_>) -> Result<(), LlmProxyError> {
    let usage_record = token_usage_record(request_id, &input, OffsetDateTime::now_utc());
    let policy = request_record_policy(state).await?;
    let store = ProviderStore::new(state.database.clone());
    match store.update_request_candidate(attempt_patch(request_id, &input, &policy)?).await {
        Ok(_) => {}
        Err(StorageError::NotFound) => create_missing_attempt(&store, request_id, &input, &policy).await?,
        Err(error) => return Err(error.into()),
    }
    store.update_request_record(request_record_patch(request_id, &input, &policy)?).await?;
    if let Some(record) = usage_record? {
        state.tokens.record_usage(record).await?;
    }
    Ok(())
}

pub async fn record_skipped_candidates(state: &LlmProxyState, request_id: &str, skip_reason: &str) -> Result<(), LlmProxyError> {
    ProviderStore::new(state.database.clone())
        .mark_scheduled_request_candidates_skipped(request_id, skip_reason)
        .await?;
    Ok(())
}

async fn create_record(state: &LlmProxyState, input: RequestCandidateRecordInput) -> Result<(), LlmProxyError> {
    ProviderStore::new(state.database.clone()).create_request_candidate(input).await?;
    Ok(())
}

async fn create_request_record(state: &LlmProxyState, input: RequestRecordRecordInput) -> Result<(), LlmProxyError> {
    ProviderStore::new(state.database.clone()).create_request_record(input).await?;
    Ok(())
}

async fn request_record_policy(state: &LlmProxyState) -> Result<RequestRecordPolicy, LlmProxyError> {
    let settings = SettingStore::new(state.database.clone()).get_system_settings().await?;
    RequestRecordPolicy::from_settings(&settings).map_err(LlmProxyError::Infrastructure)
}

async fn create_missing_attempt(
    store: &ProviderStore,
    request_id: &str,
    input: &AttemptRecordInput<'_>,
    policy: &RequestRecordPolicy,
) -> Result<(), LlmProxyError> {
    store.create_request_candidate(attempt_input(request_id, input, policy)?).await?;
    Ok(())
}

fn attempt_patch(request_id: &str, input: &AttemptRecordInput<'_>, policy: &RequestRecordPolicy) -> Result<RequestCandidateRecordPatch, LlmProxyError> {
    let billing = attempt_billing(input);
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
        cost_currency: billing.as_ref().map(|amount| amount.currency.clone()),
        token_cost: billing.as_ref().map(|amount| amount.token_cost),
        base_cost: billing.as_ref().map(|amount| amount.base_cost),
        total_cost: billing.as_ref().map(|amount| amount.total_cost),
        billing_multiplier: billing.map(|amount| amount.billing_multiplier),
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

fn attempt_input(request_id: &str, input: &AttemptRecordInput<'_>, policy: &RequestRecordPolicy) -> Result<RequestCandidateRecordInput, LlmProxyError> {
    let billing = attempt_billing(input);
    let mut record = base_input(request_id, &input.candidate.trace, input.retry_index, input.status, true, input.finished);
    record.status_code = input.status_code;
    record.skip_reason = input.skip_reason.map(str::to_owned);
    record.prompt_tokens = input.usage.and_then(|usage| usage.prompt_tokens);
    record.completion_tokens = input.usage.and_then(|usage| usage.completion_tokens);
    record.total_tokens = input.usage.and_then(|usage| usage.total_tokens);
    record.cache_creation_input_tokens = input.usage.and_then(|usage| usage.cache_creation_input_tokens);
    record.cache_read_input_tokens = input.usage.and_then(|usage| usage.cache_read_input_tokens);
    record.cost_currency = billing.as_ref().map(|amount| amount.currency.clone());
    record.token_cost = billing.as_ref().map(|amount| amount.token_cost);
    record.base_cost = billing.as_ref().map(|amount| amount.base_cost);
    record.total_cost = billing.as_ref().map(|amount| amount.total_cost);
    record.billing_multiplier = billing.map(|amount| amount.billing_multiplier);
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
        cost_currency: None,
        token_cost: None,
        base_cost: None,
        total_cost: None,
        billing_multiplier: None,
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

fn scheduled_input(request_id: &str, trace: &CandidateTrace, retry_index: i32) -> RequestCandidateRecordInput {
    base_input(request_id, trace, retry_index, "scheduled", false, false)
}

fn request_record_input(
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
        candidate_count: selection.candidates.len().try_into().unwrap_or(i64::MAX),
        request_headers: capture.request_headers(policy),
        request_body: capture.request_body(policy).map_err(|error| LlmProxyError::Infrastructure(error.to_string()))?,
    })
}

fn request_record_patch(request_id: &str, input: &AttemptRecordInput<'_>, policy: &RequestRecordPolicy) -> Result<RequestRecordRecordPatch, LlmProxyError> {
    let billing = attempt_billing(input);
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
        billing_status: billing_status(input.status).into(),
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
        cost_currency: option_patch(billing.as_ref().map(|amount| amount.currency.clone())),
        token_cost: option_patch(billing.as_ref().map(|amount| amount.token_cost)),
        base_cost: option_patch(billing.as_ref().map(|amount| amount.base_cost)),
        total_cost: option_patch(billing.as_ref().map(|amount| amount.total_cost)),
        billing_multiplier: option_patch(billing.map(|amount| amount.billing_multiplier)),
        first_byte_time_ms: option_patch(input.first_byte_time_ms),
        total_latency_ms: option_patch(input.latency_ms),
        client_response_headers: header_patch(input.client_response_headers.clone(), policy)?,
        client_response_body: response_body_patch(input.client_response_body.clone(), policy)?,
        started: true,
        finished: input.finished,
    })
}

fn attempt_billing(input: &AttemptRecordInput<'_>) -> Option<provider::application::billing::RequestBillingAmount> {
    if input.status != "success" {
        return None;
    }
    let usage = input.usage?;
    Some(calculate_request_billing(RequestBillingInput {
        prompt_tokens: usage.prompt_tokens.unwrap_or(0),
        completion_tokens: usage.completion_tokens.unwrap_or(0),
        cache_creation_input_tokens: usage.cache_creation_input_tokens.unwrap_or(0),
        cache_read_input_tokens: usage.cache_read_input_tokens.unwrap_or(0),
        price_per_request: input.candidate.price_per_request,
        tiered_pricing: input.candidate.tiered_pricing.clone(),
        billing_multiplier: input.candidate.billing_multiplier,
    }))
}

fn token_usage_record(request_id: &str, input: &AttemptRecordInput<'_>, used_at: OffsetDateTime) -> Result<Option<ApiTokenUsageRecord>, LlmProxyError> {
    if !should_record_token_usage(input) {
        return Ok(None);
    }
    let Some(token_id) = input.candidate.trace.token_id.clone() else {
        return Ok(None);
    };
    let cost = attempt_billing(input)
        .map(|amount| amount.total_cost)
        .ok_or_else(|| LlmProxyError::Infrastructure(format!("successful token request missing billing usage: {request_id}/{token_id}")))?;
    Ok(Some(ApiTokenUsageRecord { cost, token_id, used_at }))
}

fn should_record_token_usage(input: &AttemptRecordInput<'_>) -> bool {
    input.status == "success" && input.finished
}

fn billing_status(status: &str) -> &'static str {
    match status {
        "success" => "settled",
        "failed" | "cancelled" => "void",
        _ => "pending",
    }
}

fn total_tokens(usage: Option<TokenUsage>) -> Option<i64> {
    usage.and_then(|item| item.total_tokens.or_else(|| Some(item.prompt_tokens? + item.completion_tokens?)))
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

pub struct AttemptRecordInput<'a> {
    pub candidate: &'a ProxyCandidate,
    pub retry_index: i32,
    pub status: &'a str,
    pub skip_reason: Option<&'a str>,
    pub status_code: Option<i32>,
    pub usage: Option<TokenUsage>,
    pub latency_ms: Option<i64>,
    pub first_byte_time_ms: Option<i64>,
    pub error_type: Option<&'a str>,
    pub error_message: Option<&'a str>,
    pub error_code: Option<&'a str>,
    pub error_param: Option<&'a str>,
    pub provider_request_headers: PatchField<HeaderMap>,
    pub provider_request_body: PatchField<Value>,
    pub provider_response_headers: PatchField<HeaderMap>,
    pub provider_response_body: PatchField<Value>,
    pub client_response_headers: PatchField<HeaderMap>,
    pub client_response_body: PatchField<Value>,
    pub termination_origin: PatchField<String>,
    pub termination_reason: PatchField<String>,
    pub stream_end_reason: PatchField<String>,
    pub finished: bool,
}

impl<'a> AttemptRecordInput<'a> {
    pub fn new(candidate: &'a ProxyCandidate, retry_index: i32, status: &'a str, finished: bool) -> Self {
        Self {
            candidate,
            retry_index,
            status,
            skip_reason: None,
            status_code: None,
            usage: None,
            latency_ms: None,
            first_byte_time_ms: None,
            error_type: None,
            error_message: None,
            error_code: None,
            error_param: None,
            provider_request_headers: PatchField::Missing,
            provider_request_body: PatchField::Missing,
            provider_response_headers: PatchField::Missing,
            provider_response_body: PatchField::Missing,
            client_response_headers: PatchField::Missing,
            client_response_body: PatchField::Missing,
            termination_origin: PatchField::Missing,
            termination_reason: PatchField::Missing,
            stream_end_reason: PatchField::Missing,
            finished,
        }
    }
}

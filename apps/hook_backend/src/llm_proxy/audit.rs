use provider::application::billing::{RequestBillingInput, calculate_request_billing};
use storage::{
    api_token::ApiTokenUsageRecord,
    provider::{ProviderStore, RequestCandidateRecordInput, RequestCandidateRecordPatch},
    setting::SettingStore,
};
use time::OffsetDateTime;

use super::{
    LlmProxyError, LlmProxyState,
    candidate::{CandidateSelection, CandidateTrace, ProxyCandidate},
    proxy::capture::RequestCapture,
    request_record_policy::RequestRecordPolicy,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct TokenUsage {
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
}

pub async fn record_available_candidates(state: &LlmProxyState, selection: &CandidateSelection, capture: &RequestCapture) -> Result<(), LlmProxyError> {
    let policy = request_record_policy(state).await?;
    for candidate in &selection.candidates {
        for retry_index in 0..=candidate.max_retries {
            create_record(state, available_input(&selection.request_id, &candidate.trace, retry_index, capture, &policy)?).await?;
        }
    }
    Ok(())
}

pub async fn record_attempt(state: &LlmProxyState, request_id: &str, input: AttemptRecordInput<'_>) -> Result<(), LlmProxyError> {
    update_attempt(state, request_id, input).await
}

pub async fn update_attempt(state: &LlmProxyState, request_id: &str, input: AttemptRecordInput<'_>) -> Result<(), LlmProxyError> {
    let usage_record = token_usage_record(request_id, &input, OffsetDateTime::now_utc());
    let policy = request_record_policy(state).await?;
    ProviderStore::new(state.database.clone())
        .update_request_candidate(attempt_patch(request_id, input, &policy)?)
        .await?;
    if let Some(record) = usage_record? {
        state.tokens.record_usage(record).await?;
    }
    Ok(())
}

async fn create_record(state: &LlmProxyState, input: RequestCandidateRecordInput) -> Result<(), LlmProxyError> {
    ProviderStore::new(state.database.clone()).create_request_candidate(input).await?;
    Ok(())
}

async fn request_record_policy(state: &LlmProxyState) -> Result<RequestRecordPolicy, LlmProxyError> {
    let settings = SettingStore::new(state.database.clone()).get_system_settings().await?;
    RequestRecordPolicy::from_settings(&settings).map_err(LlmProxyError::Infrastructure)
}

fn attempt_patch(request_id: &str, input: AttemptRecordInput<'_>, policy: &RequestRecordPolicy) -> Result<RequestCandidateRecordPatch, LlmProxyError> {
    let billing = attempt_billing(&input);
    Ok(RequestCandidateRecordPatch {
        request_id: request_id.to_owned(),
        candidate_index: input.candidate.trace.candidate_index,
        retry_index: input.retry_index,
        status: input.status.to_owned(),
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
        response_body: policy
            .response_body(input.response_body)
            .map_err(|error| LlmProxyError::Infrastructure(error.to_string()))?,
        finished: input.finished,
    })
}

fn base_input(
    request_id: &str,
    trace: &CandidateTrace,
    retry_index: i32,
    status: &str,
    started: bool,
    finished: bool,
    capture: &RequestCapture,
    policy: &RequestRecordPolicy,
) -> Result<RequestCandidateRecordInput, LlmProxyError> {
    Ok(RequestCandidateRecordInput {
        request_id: request_id.to_owned(),
        token_id: trace.token_id.clone(),
        group_code: trace.group_code.clone(),
        global_model_id: Some(trace.global_model_id.clone()),
        provider_id: Some(trace.provider_id.clone()),
        endpoint_id: Some(trace.endpoint_id.clone()),
        key_id: Some(trace.key_id.clone()),
        client_api_format: trace.client_api_format.clone(),
        provider_api_format: Some(trace.provider_api_format.clone()),
        needs_conversion: trace.needs_conversion,
        is_stream: trace.is_stream,
        request_headers: capture.request_headers(policy),
        request_body: capture.request_body(policy).map_err(|error| LlmProxyError::Infrastructure(error.to_string()))?,
        response_body: None,
        candidate_index: trace.candidate_index,
        retry_index,
        status: status.to_owned(),
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
        started,
        finished,
    })
}

fn available_input(
    request_id: &str,
    trace: &CandidateTrace,
    retry_index: i32,
    capture: &RequestCapture,
    policy: &RequestRecordPolicy,
) -> Result<RequestCandidateRecordInput, LlmProxyError> {
    base_input(request_id, trace, retry_index, "available", false, false, capture, policy)
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

pub struct AttemptRecordInput<'a> {
    pub candidate: &'a ProxyCandidate,
    pub retry_index: i32,
    pub status: &'a str,
    pub status_code: Option<i32>,
    pub usage: Option<TokenUsage>,
    pub latency_ms: Option<i64>,
    pub first_byte_time_ms: Option<i64>,
    pub error_type: Option<&'a str>,
    pub error_message: Option<&'a str>,
    pub response_body: Option<serde_json::Value>,
    pub finished: bool,
}

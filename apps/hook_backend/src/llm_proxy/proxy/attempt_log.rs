use std::time::Instant;

use axum::response::Response;
use types::model::PatchField;

use super::{LlmProxyError, LlmProxyState};
use crate::llm_proxy::{
    audit::{AttemptRecordInput, record_attempt},
    candidate::ProxyCandidate,
};

pub(super) async fn record_attempt_error(
    state: &LlmProxyState,
    request_id: &str,
    candidate: &ProxyCandidate,
    retry_index: i32,
    error: LlmProxyError,
    last_error: &mut Option<LlmProxyError>,
) -> Result<Option<Response>, LlmProxyError> {
    record_failed_attempt(state, request_id, candidate, retry_index, "request_conversion_error", &error).await?;
    *last_error = Some(error);
    Ok(None)
}

pub(super) async fn record_rate_limit_rejection(
    state: &LlmProxyState,
    request_id: &str,
    candidate: &ProxyCandidate,
    retry_index: i32,
    error: LlmProxyError,
    last_error: &mut Option<LlmProxyError>,
) -> Result<Option<Response>, LlmProxyError> {
    record_failed_attempt(state, request_id, candidate, retry_index, "provider_key_rate_limit", &error).await?;
    *last_error = Some(error);
    Ok(None)
}

async fn record_failed_attempt(
    state: &LlmProxyState,
    request_id: &str,
    candidate: &ProxyCandidate,
    retry_index: i32,
    error_type: &'static str,
    error: &LlmProxyError,
) -> Result<(), LlmProxyError> {
    let error_message = error.to_string();
    record_attempt(
        state,
        request_id,
        AttemptRecordInput {
            error_type: Some(error_type),
            error_message: Some(error_message.as_str()),
            ..AttemptRecordInput::new(candidate, retry_index, "failed", true)
        },
    )
    .await
}

pub(super) async fn record_started_attempt(
    state: &LlmProxyState,
    request_id: &str,
    candidate: &ProxyCandidate,
    is_stream: bool,
    retry_index: i32,
    request: &reqwest::Request,
    provider_body: &serde_json::Value,
) -> Result<(), LlmProxyError> {
    record_attempt(
        state,
        request_id,
        AttemptRecordInput {
            status: if is_stream { "streaming" } else { "pending" },
            provider_request_headers: PatchField::Value(request.headers().clone()),
            provider_request_body: PatchField::Value(provider_body.clone()),
            client_response_headers: PatchField::Null,
            client_response_body: PatchField::Null,
            ..AttemptRecordInput::new(candidate, retry_index, "pending", false)
        },
    )
    .await
}

pub(super) async fn record_send_error(
    state: &LlmProxyState,
    request_id: &str,
    candidate: &ProxyCandidate,
    retry_index: i32,
    started: Instant,
    error: &reqwest::Error,
    last_error: &mut Option<LlmProxyError>,
) -> Result<Option<Response>, LlmProxyError> {
    let error_message = error.to_string();
    record_attempt(
        state,
        request_id,
        AttemptRecordInput {
            latency_ms: Some(elapsed_ms(started)),
            error_type: Some(send_error_type(error)),
            error_message: Some(error_message.as_str()),
            ..AttemptRecordInput::new(candidate, retry_index, "failed", true)
        },
    )
    .await?;
    *last_error = Some(LlmProxyError::Upstream(error_message));
    Ok(None)
}

fn send_error_type(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        return "upstream_timeout";
    }
    "upstream_send_error"
}

fn elapsed_ms(started: Instant) -> i64 {
    started.elapsed().as_millis().try_into().unwrap_or(i64::MAX)
}

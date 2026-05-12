use std::time::Instant;

use axum::response::Response;

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
    record_conversion_error(state, request_id, candidate, retry_index, &error).await?;
    *last_error = Some(error);
    Ok(None)
}

async fn record_conversion_error(
    state: &LlmProxyState,
    request_id: &str,
    candidate: &ProxyCandidate,
    retry_index: i32,
    error: &LlmProxyError,
) -> Result<(), LlmProxyError> {
    let error_message = error.to_string();
    record_attempt(
        state,
        request_id,
        AttemptRecordInput {
            candidate,
            retry_index,
            status: "failed",
            status_code: None,
            usage: None,
            latency_ms: None,
            first_byte_time_ms: None,
            error_type: Some("request_conversion_error"),
            error_message: Some(error_message.as_str()),
            response_body: None,
            finished: true,
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
) -> Result<(), LlmProxyError> {
    record_attempt(
        state,
        request_id,
        AttemptRecordInput {
            candidate,
            retry_index,
            status: if is_stream { "streaming" } else { "pending" },
            status_code: None,
            usage: None,
            latency_ms: None,
            first_byte_time_ms: None,
            error_type: None,
            error_message: None,
            response_body: None,
            finished: false,
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
            candidate,
            retry_index,
            status: "failed",
            status_code: None,
            usage: None,
            latency_ms: Some(elapsed_ms(started)),
            first_byte_time_ms: None,
            error_type: Some(send_error_type(error)),
            error_message: Some(error_message.as_str()),
            response_body: None,
            finished: true,
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

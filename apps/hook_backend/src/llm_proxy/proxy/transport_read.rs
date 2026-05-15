use std::time::Instant;

use super::{LlmProxyError, LlmProxyState};
use crate::llm_proxy::{
    audit::{AttemptRecordInput, record_attempt},
    candidate::ProxyCandidate,
};

pub(super) async fn response_bytes(
    state: &LlmProxyState,
    request_id: &str,
    candidate: &ProxyCandidate,
    retry_index: i32,
    started: Instant,
    first_byte_time_ms: Option<i64>,
    response: req::Response,
) -> Result<Vec<u8>, LlmProxyError> {
    match req::response_bytes(response).await {
        Ok(bytes) => Ok(bytes),
        Err(error) => {
            record_response_read_error(state, request_id, candidate, retry_index, started, first_byte_time_ms, &error).await?;
            Err(error.into())
        }
    }
}

async fn record_response_read_error(
    state: &LlmProxyState,
    request_id: &str,
    candidate: &ProxyCandidate,
    retry_index: i32,
    started: Instant,
    first_byte_time_ms: Option<i64>,
    error: &req::ClientError,
) -> Result<(), LlmProxyError> {
    let error_message = error.to_string();
    record_attempt(
        state,
        request_id,
        AttemptRecordInput {
            latency_ms: Some(elapsed_ms(started)),
            first_byte_time_ms,
            error_type: Some(response_read_error_type(error)),
            error_message: Some(error_message.as_str()),
            ..AttemptRecordInput::new(candidate, retry_index, "failed", true)
        },
    )
    .await
}

fn response_read_error_type(error: &req::ClientError) -> &'static str {
    if matches!(error, req::ClientError::Timeout) {
        return "upstream_timeout";
    }
    "upstream_response_read_error"
}

fn elapsed_ms(started: Instant) -> i64 {
    started.elapsed().as_millis().try_into().unwrap_or(i64::MAX)
}

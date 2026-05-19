use std::{future::Future, time::Instant};

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
    read_timeout: Option<std::time::Duration>,
    response: req::Response,
) -> Result<Vec<u8>, LlmProxyError> {
    match read_response_bytes(req::response_bytes(response), read_timeout).await {
        Ok(bytes) => Ok(bytes),
        Err(error) => {
            record_response_read_error(state, request_id, candidate, retry_index, started, first_byte_time_ms, &error).await?;
            Err(error.into())
        }
    }
}

async fn read_response_bytes<F>(read: F, read_timeout: Option<std::time::Duration>) -> Result<Vec<u8>, req::ClientError>
where
    F: Future<Output = Result<Vec<u8>, req::ClientError>>,
{
    match read_timeout {
        Some(timeout) => tokio::time::timeout(timeout, read).await.unwrap_or(Err(req::ClientError::Timeout)),
        None => read.await,
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::read_response_bytes;

    #[tokio::test]
    async fn read_response_bytes_returns_timeout_when_read_exceeds_budget() {
        let result = read_response_bytes(
            async {
                tokio::time::sleep(Duration::from_secs(1)).await;
                Ok(Vec::new())
            },
            Some(Duration::from_millis(1)),
        )
        .await;

        assert!(matches!(result, Err(req::ClientError::Timeout)));
    }

    #[tokio::test]
    async fn read_response_bytes_returns_body_without_timeout() {
        let result = read_response_bytes(async { Ok(vec![1, 2, 3]) }, None).await;

        assert_eq!(result.unwrap(), vec![1, 2, 3]);
    }
}

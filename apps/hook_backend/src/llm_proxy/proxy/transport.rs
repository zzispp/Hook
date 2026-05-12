use std::time::Instant;

use axum::{
    body::Body,
    http::{HeaderValue, StatusCode, header},
    response::Response,
};
use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::Value;

use super::{LlmProxyError, LlmProxyState, response_payload::body_value, transport_read::response_bytes, usage};
use crate::llm_proxy::{
    audit::{AttemptRecordInput, record_attempt},
    candidate::ProxyCandidate,
};

pub struct UpstreamFailure {
    status: StatusCode,
    content_type: Option<HeaderValue>,
    body: Vec<u8>,
}

pub async fn full_response(
    state: LlmProxyState,
    request_id: String,
    response: reqwest::Response,
    candidate: ProxyCandidate,
    source_format: ApiFormat,
    target_format: ApiFormat,
    started: Instant,
    retry_index: i32,
) -> Result<Response, LlmProxyError> {
    let status = status_code(response.status())?;
    let content_type = response_content_type(&response);
    let bytes = response_bytes(&state, &request_id, &candidate, retry_index, started, None, response).await?;
    let elapsed = elapsed_ms(started);
    if status.is_success() {
        return full_success_response(FullResponseInput {
            state,
            request_id,
            candidate,
            source_format,
            target_format,
            retry_index,
            status,
            content_type,
            bytes,
            elapsed,
        })
        .await;
    }
    record_attempt(
        &state,
        &request_id,
        AttemptRecordInput {
            candidate: &candidate,
            retry_index,
            status: "failed",
            status_code: Some(status.as_u16() as i32),
            usage: None,
            latency_ms: Some(elapsed),
            first_byte_time_ms: Some(elapsed),
            error_type: Some("upstream_status"),
            error_message: Some("upstream returned non-success status"),
            response_body: Some(body_value(&bytes)),
            finished: true,
        },
    )
    .await?;
    response_builder(status, content_type).body(Body::from(bytes)).map_err(response_error)
}

struct FullResponseInput {
    state: LlmProxyState,
    request_id: String,
    candidate: ProxyCandidate,
    source_format: ApiFormat,
    target_format: ApiFormat,
    retry_index: i32,
    status: StatusCode,
    content_type: Option<HeaderValue>,
    bytes: Vec<u8>,
    elapsed: i64,
}

async fn full_success_response(input: FullResponseInput) -> Result<Response, LlmProxyError> {
    let body = match response_body(&input.bytes, input.candidate.trace.needs_conversion, input.source_format, input.target_format) {
        Ok(value) => value,
        Err(error) => {
            record_response_conversion_failure(&input, &error).await?;
            return Err(error);
        }
    };
    record_attempt(
        &input.state,
        &input.request_id,
        AttemptRecordInput {
            candidate: &input.candidate,
            retry_index: input.retry_index,
            status: "success",
            status_code: Some(input.status.as_u16() as i32),
            usage: usage::from_response_bytes(&input.bytes, input.target_format),
            latency_ms: Some(input.elapsed),
            first_byte_time_ms: Some(input.elapsed),
            error_type: None,
            error_message: None,
            response_body: Some(body_value(&body)),
            finished: true,
        },
    )
    .await?;
    response_builder(input.status, input.content_type)
        .body(Body::from(body))
        .map_err(response_error)
}

async fn record_response_conversion_failure(input: &FullResponseInput, error: &LlmProxyError) -> Result<(), LlmProxyError> {
    let error_message = error.to_string();
    record_attempt(
        &input.state,
        &input.request_id,
        AttemptRecordInput {
            candidate: &input.candidate,
            retry_index: input.retry_index,
            status: "failed",
            status_code: Some(input.status.as_u16() as i32),
            usage: None,
            latency_ms: Some(input.elapsed),
            first_byte_time_ms: Some(input.elapsed),
            error_type: Some("response_conversion_error"),
            error_message: Some(error_message.as_str()),
            response_body: Some(body_value(&input.bytes)),
            finished: true,
        },
    )
    .await
}

pub async fn record_upstream_failure(
    state: &LlmProxyState,
    request_id: &str,
    response: reqwest::Response,
    candidate: &ProxyCandidate,
    started: Instant,
    retry_index: i32,
) -> Result<UpstreamFailure, LlmProxyError> {
    let status = status_code(response.status())?;
    let content_type = response_content_type(&response);
    let body = response.bytes().await?.to_vec();
    record_attempt(
        state,
        request_id,
        AttemptRecordInput {
            candidate,
            retry_index,
            status: "failed",
            status_code: Some(status.as_u16() as i32),
            usage: None,
            latency_ms: Some(elapsed_ms(started)),
            first_byte_time_ms: None,
            error_type: Some("upstream_status"),
            error_message: Some("upstream returned non-success status"),
            response_body: Some(body_value(&body)),
            finished: true,
        },
    )
    .await?;
    Ok(UpstreamFailure { status, content_type, body })
}

pub fn failure_response(failure: UpstreamFailure) -> Result<Response, LlmProxyError> {
    response_builder(failure.status, failure.content_type)
        .body(Body::from(failure.body))
        .map_err(response_error)
}

fn response_body(bytes: &[u8], needs_conversion: bool, source_format: ApiFormat, target_format: ApiFormat) -> Result<Vec<u8>, LlmProxyError> {
    if !needs_conversion {
        return Ok(bytes.to_vec());
    }
    let value: Value = serde_json::from_slice(bytes).map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
    let converted = FormatConversionRegistry::default()
        .convert_response(&value, target_format, source_format)
        .map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
    serde_json::to_vec(&converted).map_err(|error| LlmProxyError::Infrastructure(error.to_string()))
}

pub(super) fn elapsed_ms(started: Instant) -> i64 {
    started.elapsed().as_millis().try_into().unwrap_or(i64::MAX)
}

pub(super) fn status_code(status: reqwest::StatusCode) -> Result<StatusCode, LlmProxyError> {
    StatusCode::from_u16(status.as_u16()).map_err(|error| LlmProxyError::Infrastructure(error.to_string()))
}

pub(super) fn response_content_type(response: &reqwest::Response) -> Option<HeaderValue> {
    response.headers().get(header::CONTENT_TYPE).cloned()
}

pub(super) fn response_builder(status: StatusCode, content_type: Option<HeaderValue>) -> axum::http::response::Builder {
    let mut builder = Response::builder().status(status);
    if let Some(value) = content_type {
        builder = builder.header(header::CONTENT_TYPE, value);
    }
    builder
}

pub(super) fn response_error(error: axum::http::Error) -> LlmProxyError {
    LlmProxyError::Infrastructure(error.to_string())
}

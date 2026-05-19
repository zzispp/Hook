use std::time::Instant;

use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::Response,
};
use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::Value;
use types::model::PatchField;

use super::{
    LlmProxyError, LlmProxyState,
    response_model::rewrite_response_model_bytes,
    response_payload::{body_value, upstream_status_error_details},
    timeout::{non_stream_total_timeout, remaining_timeout},
    transport_read::{ResponseBytesInput, response_bytes},
    usage,
};
use crate::llm_proxy::{
    audit::{AttemptRecordInput, record_attempt},
    cache::ProviderCooldownFailureInput,
    candidate::ProxyCandidate,
    client_error,
};

pub struct UpstreamFailure {
    status: StatusCode,
    cooldown_triggered: bool,
}

impl UpstreamFailure {
    pub fn cooldown_triggered(&self) -> bool {
        self.cooldown_triggered
    }
}

pub struct FullResponseArgs {
    pub state: LlmProxyState,
    pub request_id: String,
    pub response: req::Response,
    pub candidate: ProxyCandidate,
    pub service_tier: Option<String>,
    pub source_format: ApiFormat,
    pub target_format: ApiFormat,
    pub started: Instant,
    pub retry_index: i32,
    pub request_timeout: Option<std::time::Duration>,
}

pub async fn full_response(args: FullResponseArgs) -> Result<Response, LlmProxyError> {
    let FullResponseArgs {
        state,
        request_id,
        response,
        candidate,
        service_tier,
        source_format,
        target_format,
        started,
        retry_index,
        request_timeout,
    } = args;
    let status = status_code(response.status())?;
    let content_type = response_content_type(&response);
    let upstream_headers = response.headers().clone();
    let read_timeout = request_timeout.map(|timeout| remaining_timeout(started, timeout));
    let bytes = response_bytes(ResponseBytesInput {
        state: &state,
        request_id: &request_id,
        candidate: &candidate,
        retry_index,
        started,
        first_byte_time_ms: None,
        read_timeout,
        response,
    })
    .await?;
    let elapsed = elapsed_ms(started);
    if status.is_success() {
        return full_success_response(FullResponseInput {
            state,
            request_id,
            candidate,
            service_tier,
            source_format,
            target_format,
            retry_index,
            status,
            content_type,
            upstream_headers,
            bytes,
            elapsed,
        })
        .await;
    }
    let error = upstream_status_error_details(status.as_u16(), &bytes);
    let client_error = client_error::upstream_failure(status);
    record_attempt(
        &state,
        &request_id,
        AttemptRecordInput {
            status_code: Some(status.as_u16() as i32),
            latency_ms: Some(elapsed),
            first_byte_time_ms: Some(elapsed),
            error_type: Some("upstream_status"),
            error_message: Some(error.message.as_str()),
            error_code: error.code.as_deref(),
            error_param: error.param.as_deref(),
            provider_response_headers: PatchField::Value(upstream_headers),
            provider_response_body: PatchField::Value(body_value(&bytes)),
            client_response_headers: PatchField::Value(client_error::json_headers()),
            client_response_body: PatchField::Value(client_error.value.clone()),
            ..AttemptRecordInput::new(&candidate, retry_index, "failed", true)
        },
    )
    .await?;
    response_builder(client_error.status, Some(client_error::json_content_type()))
        .body(Body::from(client_error.bytes().map_err(json_error)?))
        .map_err(response_error)
}

struct FullResponseInput {
    state: LlmProxyState,
    request_id: String,
    candidate: ProxyCandidate,
    service_tier: Option<String>,
    source_format: ApiFormat,
    target_format: ApiFormat,
    retry_index: i32,
    status: StatusCode,
    content_type: Option<HeaderValue>,
    upstream_headers: HeaderMap,
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
    let body = if input.candidate.provider_model_name != input.candidate.requested_model_name {
        rewrite_response_model_bytes(&body, &input.candidate.requested_model_name)?
    } else {
        body
    };
    record_attempt(
        &input.state,
        &input.request_id,
        AttemptRecordInput {
            status_code: Some(input.status.as_u16() as i32),
            usage: usage::from_response_bytes(&input.bytes, input.target_format),
            service_tier: input.service_tier.clone(),
            latency_ms: Some(input.elapsed),
            first_byte_time_ms: Some(input.elapsed),
            provider_response_headers: PatchField::Value(input.upstream_headers.clone()),
            provider_response_body: PatchField::Value(body_value(&input.bytes)),
            client_response_headers: content_type_headers(input.content_type.as_ref()),
            client_response_body: PatchField::Value(body_value(&body)),
            ..AttemptRecordInput::new(&input.candidate, input.retry_index, "success", true)
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
            status_code: Some(input.status.as_u16() as i32),
            latency_ms: Some(input.elapsed),
            first_byte_time_ms: Some(input.elapsed),
            error_type: Some("response_conversion_error"),
            error_message: Some(error_message.as_str()),
            provider_response_headers: PatchField::Value(input.upstream_headers.clone()),
            provider_response_body: PatchField::Value(body_value(&input.bytes)),
            client_response_headers: PatchField::Null,
            client_response_body: PatchField::Null,
            ..AttemptRecordInput::new(&input.candidate, input.retry_index, "failed", true)
        },
    )
    .await
}

pub async fn record_upstream_failure(
    state: &LlmProxyState,
    request_id: &str,
    response: req::Response,
    candidate: &ProxyCandidate,
    started: Instant,
    retry_index: i32,
    record_cooldown: bool,
) -> Result<UpstreamFailure, LlmProxyError> {
    let status = status_code(response.status())?;
    let upstream_headers = response.headers().clone();
    let request_timeout = non_stream_total_timeout(candidate, candidate.trace.is_stream);
    let read_timeout = request_timeout.map(|timeout| remaining_timeout(started, timeout));
    let body = response_bytes(ResponseBytesInput {
        state,
        request_id,
        candidate,
        retry_index,
        started,
        first_byte_time_ms: None,
        read_timeout,
        response,
    })
    .await?;
    let error = upstream_status_error_details(status.as_u16(), &body);
    let error_type = "upstream_status";
    let client_error = client_error::upstream_failure(status);
    record_attempt(
        state,
        request_id,
        AttemptRecordInput {
            status_code: Some(status.as_u16() as i32),
            latency_ms: Some(elapsed_ms(started)),
            error_type: Some(error_type),
            error_message: Some(error.message.as_str()),
            error_code: error.code.as_deref(),
            error_param: error.param.as_deref(),
            provider_response_headers: PatchField::Value(upstream_headers),
            provider_response_body: PatchField::Value(body_value(&body)),
            client_response_headers: PatchField::Value(client_error::json_headers()),
            client_response_body: PatchField::Value(client_error.value.clone()),
            ..AttemptRecordInput::new(candidate, retry_index, "failed", true)
        },
    )
    .await?;
    let cooldown_triggered = if record_cooldown {
        record_provider_cooldown(state, request_id, candidate, retry_index, status, error_type, &error).await?
    } else {
        false
    };
    Ok(UpstreamFailure { status, cooldown_triggered })
}

async fn record_provider_cooldown(
    state: &LlmProxyState,
    request_id: &str,
    candidate: &ProxyCandidate,
    retry_index: i32,
    status: StatusCode,
    error_type: &str,
    error: &super::response_payload::UpstreamStatusErrorDetails,
) -> Result<bool, LlmProxyError> {
    state
        .record_provider_status_failure(ProviderCooldownFailureInput {
            request_id,
            candidate,
            retry_index,
            status_code: status.as_u16() as i32,
            error_type,
            error_message: &error.message,
            error_code: error.code.as_deref(),
            error_param: error.param.as_deref(),
        })
        .await
}

pub fn failure_response(failure: UpstreamFailure) -> Result<Response, LlmProxyError> {
    let client_error = client_error::upstream_failure(failure.status);
    response_builder(client_error.status, Some(client_error::json_content_type()))
        .body(Body::from(client_error.bytes().map_err(json_error)?))
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

pub(super) fn content_type_headers(content_type: Option<&HeaderValue>) -> PatchField<HeaderMap> {
    let Some(content_type) = content_type.cloned() else {
        return PatchField::Null;
    };
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, content_type);
    PatchField::Value(headers)
}

pub(super) fn elapsed_ms(started: Instant) -> i64 {
    started.elapsed().as_millis().try_into().unwrap_or(i64::MAX)
}

pub(super) fn status_code(status: req::StatusCode) -> Result<StatusCode, LlmProxyError> {
    Ok(req::response_status_code(status))
}

pub(super) fn response_content_type(response: &req::Response) -> Option<HeaderValue> {
    req::response_content_type(response)
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

fn json_error(error: serde_json::Error) -> LlmProxyError {
    LlmProxyError::Infrastructure(error.to_string())
}

#[cfg(test)]
mod tests;

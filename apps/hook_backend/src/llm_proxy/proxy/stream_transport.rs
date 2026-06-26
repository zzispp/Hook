mod body_capture;
mod estimated_usage;
mod event;
mod output_start;
mod preflight;
mod record;
mod relay;
mod sse_event;
mod status;
mod terminal;
pub(super) mod token_estimator;
mod usage_parser;

use std::{pin::Pin, time::Duration, time::Instant};

use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::Response,
};
use futures_util::{Stream, stream};
use proxy::format_conversion::ApiFormat;
use types::model::PatchField;

use super::{
    LlmProxyError, LlmProxyState,
    attempt_log::AttemptCancelGuard,
    response_payload::{body_value, upstream_status_error_details},
    timeout, transport,
};
use crate::llm_proxy::{
    audit::{AttemptRecordInput, record_attempt},
    candidate::ProxyCandidate,
    client_error,
};

type UpstreamStream = Pin<Box<dyn Stream<Item = Result<req::Bytes, req::ClientError>> + Send>>;

pub(super) struct StreamAttemptContext {
    state: LlmProxyState,
    request_id: String,
    candidate: ProxyCandidate,
    provider_request_body: serde_json::Value,
    retry_index: i32,
    started: Instant,
    response_headers_time_ms: i64,
    status: StatusCode,
}

pub struct StreamResponseArgs {
    pub state: LlmProxyState,
    pub request_id: String,
    pub response: req::Response,
    pub candidate: ProxyCandidate,
    pub source_format: ApiFormat,
    pub target_format: ApiFormat,
    pub provider_request_body: serde_json::Value,
    pub started: Instant,
    pub retry_index: i32,
}

pub(super) enum StreamResponseOutcome {
    Response(Response),
    PreOutputFailure(StreamPreOutputFailure),
}

pub(super) struct StreamPreOutputFailure {
    pub(super) status: StatusCode,
    pub(super) error_type: &'static str,
    pub(super) message: String,
    pub(super) advance_candidate: bool,
}

pub async fn stream_response(args: StreamResponseArgs, attempt_cancel: &AttemptCancelGuard) -> Result<StreamResponseOutcome, LlmProxyError> {
    let StreamResponseArgs {
        state,
        request_id,
        response,
        candidate,
        source_format,
        target_format,
        provider_request_body,
        started,
        retry_index,
    } = args;
    let status = transport::status_code(response.status())?;
    let content_type = transport::response_content_type(&response);
    let upstream_headers = response.headers().clone();
    let context = StreamAttemptContext {
        state,
        request_id,
        candidate,
        provider_request_body,
        retry_index,
        started,
        response_headers_time_ms: transport::elapsed_ms(started),
        status,
    };
    if !status.is_success() {
        return stream_status_failure(context, response, upstream_headers, content_type)
            .await
            .map(StreamResponseOutcome::Response);
    }

    let upstream = req::response_bytes_stream(response);
    let first_byte_timeout = timeout::remaining_stream_first_byte_timeout(started, &context.candidate);
    attempt_cancel.disarm();
    record_stream_headers(
        &context,
        "pending",
        upstream_headers.clone(),
        content_type.as_ref(),
        context.response_headers_time_ms,
        None,
        None,
        None,
    )
        .await?;
    let mut relay = relay::StreamRelay::new(context, upstream, source_format, target_format);
    match prefetch_with_timeout(&mut relay, first_byte_timeout).await? {
        PrefetchOutcome::Ready => {}
        PrefetchOutcome::FailureResponse(response) => return Ok(StreamResponseOutcome::Response(response)),
        PrefetchOutcome::PreOutputFailure(failure) => return Ok(StreamResponseOutcome::PreOutputFailure(failure)),
    }
    relay.record_streaming_started(upstream_headers, content_type.as_ref()).await?;
    let body = Body::from_stream(stream::unfold(relay, relay::next_body_item));
    transport::response_builder(status, content_type)
        .body(body)
        .map(StreamResponseOutcome::Response)
        .map_err(transport::response_error)
}

async fn prefetch_with_timeout(relay: &mut relay::StreamRelay, timeout: Option<Duration>) -> Result<PrefetchOutcome, LlmProxyError> {
    match timeout {
        Some(timeout) => match tokio::time::timeout(timeout, relay.prefetch()).await {
            Ok(Ok(())) => prefetch_outcome(relay),
            Ok(Err(error)) => prefetch_error_outcome(relay, error),
            Err(_) => {
                relay.record_first_byte_timeout().await?;
                prefetch_outcome(relay)
            }
        },
        None => match relay.prefetch().await {
            Ok(()) => prefetch_outcome(relay),
            Err(error) => prefetch_error_outcome(relay, error),
        },
    }
}

enum PrefetchOutcome {
    Ready,
    FailureResponse(Response),
    PreOutputFailure(StreamPreOutputFailure),
}

fn prefetch_outcome(relay: &relay::StreamRelay) -> Result<PrefetchOutcome, LlmProxyError> {
    if let Some(failure) = relay.pre_output_failure()? {
        return Ok(PrefetchOutcome::PreOutputFailure(failure));
    }
    if let Some(response) = relay.prefetch_failure_response()? {
        return Ok(PrefetchOutcome::FailureResponse(response));
    }
    Ok(PrefetchOutcome::Ready)
}

fn prefetch_error_outcome(relay: &relay::StreamRelay, error: LlmProxyError) -> Result<PrefetchOutcome, LlmProxyError> {
    if let Some(failure) = relay.pre_output_failure()? {
        return Ok(PrefetchOutcome::PreOutputFailure(failure));
    }
    relay.failure_response().map(PrefetchOutcome::FailureResponse).or(Err(error))
}

fn should_record_streaming_started_after_prefetch(finished: bool, recorded_terminal: bool) -> bool {
    !finished && !recorded_terminal
}

async fn stream_status_failure(
    context: StreamAttemptContext,
    response: req::Response,
    upstream_headers: HeaderMap,
    _content_type: Option<HeaderValue>,
) -> Result<Response, LlmProxyError> {
    let bytes = req::response_bytes(response).await?;
    let error = upstream_status_error_details(context.status.as_u16(), &bytes);
    let client_error = client_error::upstream_failure(context.status);
    record_attempt(
        &context.state,
        &context.request_id,
        AttemptRecordInput {
            status_code: Some(context.status.as_u16() as i32),
            latency_ms: Some(transport::elapsed_ms(context.started)),
            error_type: Some("upstream_status"),
            error_message: Some(error.message.as_str()),
            error_code: error.code.as_deref(),
            error_param: error.param.as_deref(),
            provider_response_headers: PatchField::Value(upstream_headers),
            provider_response_body: PatchField::Value(body_value(&bytes)),
            client_response_headers: PatchField::Value(client_error::json_headers()),
            client_response_body: PatchField::Value(client_error.value.clone()),
            ..AttemptRecordInput::new(&context.candidate, context.retry_index, "failed", true)
        },
    )
    .await?;
    transport::response_builder(client_error.status, Some(client_error::json_content_type()))
        .body(Body::from(client_error.bytes().map_err(json_error)?))
        .map_err(transport::response_error)
}

async fn record_stream_headers(
    context: &StreamAttemptContext,
    status: &'static str,
    upstream_headers: HeaderMap,
    content_type: Option<&HeaderValue>,
    response_headers_time_ms: i64,
    first_sse_event_time_ms: Option<i64>,
    first_output_time_ms: Option<i64>,
    first_byte_time_ms: Option<i64>,
) -> Result<(), LlmProxyError> {
    record_attempt(
        &context.state,
        &context.request_id,
        AttemptRecordInput {
            status_code: Some(context.status.as_u16() as i32),
            response_headers_time_ms: Some(response_headers_time_ms),
            first_sse_event_time_ms,
            first_output_time_ms,
            first_byte_time_ms,
            provider_response_headers: PatchField::Value(upstream_headers),
            client_response_headers: transport::content_type_headers(content_type),
            client_response_body: PatchField::Null,
            ..AttemptRecordInput::new(&context.candidate, context.retry_index, status, false)
        },
    )
    .await
}

fn json_error(error: serde_json::Error) -> LlmProxyError {
    LlmProxyError::Infrastructure(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::should_record_streaming_started_after_prefetch;

    #[test]
    fn image_stream_prefetch_terminal_skips_streaming_started_patch() {
        assert!(!should_record_streaming_started_after_prefetch(true, true));
    }

    #[test]
    fn active_stream_prefetch_records_streaming_started_patch() {
        assert!(should_record_streaming_started_after_prefetch(false, false));
    }
}

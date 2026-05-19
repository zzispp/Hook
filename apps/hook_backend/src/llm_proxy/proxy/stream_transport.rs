mod estimated_usage;
mod event;
mod record;
mod relay;
mod token_estimator;
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
    response_payload::{body_value, upstream_status_error_details},
    timeout::proxy_timeouts,
    transport,
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

pub async fn stream_response(args: StreamResponseArgs) -> Result<Response, LlmProxyError> {
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
        status,
    };
    if !status.is_success() {
        return stream_status_failure(context, response, upstream_headers, content_type).await;
    }

    record_stream_headers(&context, upstream_headers, content_type.as_ref()).await?;
    let upstream = req::response_bytes_stream(response);
    let first_byte_timeout = proxy_timeouts(&context.candidate).stream_first_byte;
    let mut relay = relay::StreamRelay::new(context, upstream, source_format, target_format);
    prefetch_with_timeout(&mut relay, first_byte_timeout).await?;
    let body = Body::from_stream(stream::unfold(relay, relay::next_body_item));
    transport::response_builder(status, content_type).body(body).map_err(transport::response_error)
}

async fn prefetch_with_timeout(relay: &mut relay::StreamRelay, timeout: Option<Duration>) -> Result<(), LlmProxyError> {
    match timeout {
        Some(timeout) => match tokio::time::timeout(timeout, relay.prefetch()).await {
            Ok(result) => result,
            Err(_) => {
                relay.record_first_byte_timeout().await?;
                Err(LlmProxyError::Upstream("stream first byte timeout".into()))
            }
        },
        None => relay.prefetch().await,
    }
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

async fn record_stream_headers(context: &StreamAttemptContext, upstream_headers: HeaderMap, content_type: Option<&HeaderValue>) -> Result<(), LlmProxyError> {
    record_attempt(
        &context.state,
        &context.request_id,
        AttemptRecordInput {
            status_code: Some(context.status.as_u16() as i32),
            provider_response_headers: PatchField::Value(upstream_headers),
            client_response_headers: transport::content_type_headers(content_type),
            client_response_body: PatchField::Null,
            ..AttemptRecordInput::new(&context.candidate, context.retry_index, "streaming", false)
        },
    )
    .await
}

fn json_error(error: serde_json::Error) -> LlmProxyError {
    LlmProxyError::Infrastructure(error.to_string())
}

mod event;
mod record;
mod relay;

use std::{pin::Pin, time::Instant};

use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::Response,
};
use futures_util::{Stream, StreamExt, stream};
use proxy::format_conversion::ApiFormat;
use types::model::PatchField;

use super::{
    LlmProxyError, LlmProxyState,
    response_payload::{body_value, upstream_status_error_details},
    transport,
};
use crate::llm_proxy::{
    audit::{AttemptRecordInput, record_attempt},
    candidate::ProxyCandidate,
};

type UpstreamStream = Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>;

pub(super) struct StreamAttemptContext {
    state: LlmProxyState,
    request_id: String,
    candidate: ProxyCandidate,
    retry_index: i32,
    started: Instant,
    status: StatusCode,
}

pub async fn stream_response(
    state: LlmProxyState,
    request_id: String,
    response: reqwest::Response,
    candidate: ProxyCandidate,
    source_format: ApiFormat,
    target_format: ApiFormat,
    started: Instant,
    retry_index: i32,
) -> Result<Response, LlmProxyError> {
    let status = transport::status_code(response.status())?;
    let content_type = transport::response_content_type(&response);
    let upstream_headers = response.headers().clone();
    let context = StreamAttemptContext {
        state,
        request_id,
        candidate,
        retry_index,
        started,
        status,
    };
    if !status.is_success() {
        return stream_status_failure(context, response, upstream_headers, content_type).await;
    }

    record_stream_headers(&context, upstream_headers, content_type.as_ref()).await?;
    let upstream = response.bytes_stream().boxed();
    let mut relay = relay::StreamRelay::new(context, upstream, source_format, target_format);
    relay.prefetch().await?;
    let body = Body::from_stream(stream::unfold(relay, relay::next_body_item));
    transport::response_builder(status, content_type).body(body).map_err(transport::response_error)
}

async fn stream_status_failure(
    context: StreamAttemptContext,
    response: reqwest::Response,
    upstream_headers: HeaderMap,
    content_type: Option<HeaderValue>,
) -> Result<Response, LlmProxyError> {
    let bytes = response.bytes().await?;
    let error = upstream_status_error_details(context.status.as_u16(), &bytes);
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
            client_response_headers: transport::content_type_headers(content_type.as_ref()),
            client_response_body: PatchField::Value(body_value(&bytes)),
            ..AttemptRecordInput::new(&context.candidate, context.retry_index, "failed", true)
        },
    )
    .await?;
    transport::response_builder(context.status, content_type)
        .body(Body::from(bytes))
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

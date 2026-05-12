mod event;
mod record;
mod relay;

use std::{pin::Pin, time::Instant};

use axum::{
    body::{Body, Bytes},
    http::{HeaderValue, StatusCode},
    response::Response,
};
use futures_util::{Stream, StreamExt, stream};
use proxy::format_conversion::ApiFormat;

use super::{LlmProxyError, LlmProxyState, response_payload::body_value, transport};
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
    let context = StreamAttemptContext {
        state,
        request_id,
        candidate,
        retry_index,
        started,
        status,
    };
    if !status.is_success() {
        return stream_status_failure(context, response, content_type).await;
    }

    let upstream = response.bytes_stream().boxed();
    let mut relay = relay::StreamRelay::new(context, upstream, source_format, target_format);
    relay.prefetch().await?;
    let body = Body::from_stream(stream::unfold(relay, relay::next_body_item));
    transport::response_builder(status, content_type).body(body).map_err(transport::response_error)
}

async fn stream_status_failure(
    context: StreamAttemptContext,
    response: reqwest::Response,
    content_type: Option<HeaderValue>,
) -> Result<Response, LlmProxyError> {
    let bytes = response.bytes().await?;
    record_attempt(
        &context.state,
        &context.request_id,
        AttemptRecordInput {
            candidate: &context.candidate,
            retry_index: context.retry_index,
            status: "failed",
            status_code: Some(context.status.as_u16() as i32),
            usage: None,
            latency_ms: Some(transport::elapsed_ms(context.started)),
            first_byte_time_ms: None,
            error_type: Some("upstream_status"),
            error_message: Some("upstream returned non-success status"),
            response_body: Some(body_value(&bytes)),
            finished: true,
        },
    )
    .await?;
    transport::response_builder(context.status, content_type)
        .body(Body::from(bytes))
        .map_err(transport::response_error)
}

use std::{pin::Pin, time::Duration};

use axum::{body::Body, http::header, response::Response};
use futures_util::{Stream, stream};

use super::{LlmProxyError, LlmProxyState, image_record::sse_content_type, transport, transport::UpstreamFailure};
use crate::llm_proxy::candidate::ProxyCandidate;

const IMAGE_SYNC_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(10);

pub(super) type ResponseBytesResult = Result<Vec<u8>, LlmProxyError>;
pub(super) type ResponseStream = Pin<Box<dyn Stream<Item = Result<req::Bytes, std::io::Error>> + Send>>;
pub(super) type StreamImageExecutor = fn(LlmProxyState, StreamImageRequest) -> tokio::task::JoinHandle<ResponseBytesResult>;

#[derive(Clone)]
pub(super) struct StreamImageRequest {
    pub(super) request_id: String,
    pub(super) cache_affinity_ttl_minutes: i64,
    pub(super) candidates: Vec<ProxyCandidate>,
    pub(super) body: super::image_prepared::PreparedImageRequestBody,
}

pub(super) async fn stream_client_response(
    state: LlmProxyState,
    request: StreamImageRequest,
    executor: StreamImageExecutor,
) -> Result<Response, LlmProxyError> {
    let body = Body::from_stream(stream_image_attempts(state, request, executor));
    transport::response_builder(axum::http::StatusCode::OK, Some(sse_content_type()))
        .header(header::CACHE_CONTROL, "no-cache")
        .body(body)
        .map_err(transport::response_error)
}

pub(super) fn stream_image_attempts(state: LlmProxyState, request: StreamImageRequest, executor: StreamImageExecutor) -> ResponseStream {
    Box::pin(stream::unfold(
        StreamImageState::Start {
            state: Box::new(state),
            request: Box::new(request),
            executor,
        },
        next_stream_image_item,
    ))
}

enum StreamImageState {
    Start {
        state: Box<LlmProxyState>,
        request: Box<StreamImageRequest>,
        executor: StreamImageExecutor,
    },
    Waiting {
        task: tokio::task::JoinHandle<ResponseBytesResult>,
        interval: tokio::time::Interval,
    },
    Done,
}

async fn next_stream_image_item(state: StreamImageState) -> Option<(Result<req::Bytes, std::io::Error>, StreamImageState)> {
    match state {
        StreamImageState::Start { state, request, executor } => {
            let task = executor(*state, *request);
            let mut interval = tokio::time::interval(IMAGE_SYNC_KEEPALIVE_INTERVAL);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            Some((Ok(keepalive_bytes()), StreamImageState::Waiting { task, interval }))
        }
        StreamImageState::Waiting { mut task, mut interval } => tokio::select! {
            result = &mut task => Some((stream_task_result(result), StreamImageState::Done)),
            _ = interval.tick() => Some((Ok(keepalive_bytes()), StreamImageState::Waiting { task, interval })),
        },
        StreamImageState::Done => None,
    }
}

fn keepalive_bytes() -> req::Bytes {
    req::Bytes::from_static(b": PING\n\n")
}

fn stream_task_result(result: Result<ResponseBytesResult, tokio::task::JoinError>) -> Result<req::Bytes, std::io::Error> {
    match result {
        Ok(Ok(bytes)) => Ok(req::Bytes::from(bytes)),
        Ok(Err(error)) => Err(std::io::Error::other(error.to_string())),
        Err(error) => Err(std::io::Error::other(format!("image stream task failed: {error}"))),
    }
}

pub(super) async fn response_to_bytes(response: Response) -> Result<Vec<u8>, LlmProxyError> {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .map_err(|error| LlmProxyError::Infrastructure(error.to_string()))?;
    Ok(bytes.to_vec())
}

pub(super) fn failure_sse_body(failure: UpstreamFailure) -> Result<Vec<u8>, LlmProxyError> {
    response_to_error_sse(transport::failure_response(failure)?)
}

fn response_to_error_sse(response: Response) -> Result<Vec<u8>, LlmProxyError> {
    let status = response.status().as_u16();
    let body = serde_json::json!({
        "error": {
            "message": format!("upstream image request failed with status {status}"),
            "type": "upstream_status",
            "code": status,
        }
    });
    Ok(format!("event: error\ndata: {body}\n\n").into_bytes())
}

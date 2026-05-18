mod attempt_log;
mod body_rules;
pub(in crate::llm_proxy) mod capture;
mod executor;
mod failure_classification;
mod header_condition;
mod header_rules;
mod outbound_request;
mod request;
mod response_model;
mod response_payload;
mod stream_transport;
mod timeout;
mod transport;
mod transport_read;
pub(super) mod usage;

use axum::http::HeaderMap;
use axum::response::Response;
use serde_json::Value;

use super::{CurrentApiToken, LlmProxyError, LlmProxyState, candidate::CandidateSelection};

pub struct ProxyJsonRequest {
    state: LlmProxyState,
    token: CurrentApiToken,
    headers: HeaderMap,
    body: Value,
    api_format: &'static str,
    force_non_stream: bool,
}

pub(in crate::llm_proxy) struct ProxyFixedJsonRequest {
    state: LlmProxyState,
    headers: HeaderMap,
    body: Value,
    api_format: String,
    force_non_stream: bool,
    selection: CandidateSelection,
}

impl ProxyFixedJsonRequest {
    pub(in crate::llm_proxy) fn new(
        state: LlmProxyState,
        headers: HeaderMap,
        body: Value,
        api_format: impl Into<String>,
        force_non_stream: bool,
        selection: CandidateSelection,
    ) -> Self {
        Self {
            state,
            headers,
            body,
            api_format: api_format.into(),
            force_non_stream,
            selection,
        }
    }
}

impl ProxyJsonRequest {
    pub fn new(state: LlmProxyState, token: CurrentApiToken, headers: HeaderMap, body: Value, api_format: &'static str, force_non_stream: bool) -> Self {
        Self {
            state,
            token,
            headers,
            body,
            api_format,
            force_non_stream,
        }
    }
}

pub async fn proxy_json(request: ProxyJsonRequest) -> Result<Response, LlmProxyError> {
    let capture = capture::RequestCapture::new(&request.headers, &request.body);
    let prepared = request::prepare_proxy_request(
        &request.state,
        &request.token.0,
        request.body,
        request.api_format,
        request.force_non_stream,
        capture,
    )
    .await?;
    executor::execute_proxy_request(request.state, prepared).await
}

pub(in crate::llm_proxy) async fn proxy_fixed_json(request: ProxyFixedJsonRequest) -> Result<Response, LlmProxyError> {
    let capture = capture::RequestCapture::new(&request.headers, &request.body);
    let prepared = request::prepare_proxy_request_with_candidates(
        &request.state,
        request.body,
        &request.api_format,
        request.force_non_stream,
        request.headers,
        request.selection,
        capture,
    )
    .await?;
    executor::execute_proxy_request(request.state, prepared).await
}

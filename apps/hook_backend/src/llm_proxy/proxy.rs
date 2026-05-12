mod attempt_log;
pub(in crate::llm_proxy) mod capture;
mod executor;
mod request;
mod response_payload;
mod stream_transport;
mod transport;
mod transport_read;
mod usage;

use axum::http::HeaderMap;
use axum::response::Response;
use serde_json::Value;

use super::{CurrentApiToken, LlmProxyError, LlmProxyState};

pub struct ProxyJsonRequest {
    state: LlmProxyState,
    token: CurrentApiToken,
    headers: HeaderMap,
    body: Value,
    api_format: &'static str,
    force_non_stream: bool,
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

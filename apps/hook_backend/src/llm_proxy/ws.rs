mod connect;
mod relay;

use std::collections::HashMap;
use std::time::Instant;

use axum::{
    extract::{Extension, Query, State, ws::WebSocketUpgrade},
    http::HeaderMap,
    response::Response,
};

use self::{connect::connect_first_upstream, relay::relay};
use super::{
    CurrentApiToken, LlmProxyError, LlmProxyState, OPENAI_CHAT_FORMAT,
    audit::{AttemptRecordInput, record_attempt, record_available_candidates},
    candidate::{CandidateRequest, ProxyCandidate, select_candidates},
    proxy::capture::RequestCapture,
};

pub async fn realtime(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    websocket: WebSocketUpgrade,
) -> Result<Response, LlmProxyError> {
    let model_name = required_query_model(&query)?;
    let selection = select_candidates(
        &state,
        &token.0,
        CandidateRequest {
            api_format: OPENAI_CHAT_FORMAT,
            model_name,
            is_stream: true,
        },
    )
    .await?;
    let request_body = serde_json::to_value(&query).map_err(|error| LlmProxyError::Infrastructure(error.to_string()))?;
    let capture = RequestCapture::new(&headers, &request_body);
    record_available_candidates(&state, &selection, &capture).await?;
    let connected = connect_first_upstream(&state, &selection, &query).await?;
    let started = Instant::now();
    record_streaming_attempt(&state, &selection.request_id, &connected.candidate, connected.retry_index).await?;
    remember_affinity(&state, &connected.candidate).await?;
    Ok(websocket.on_upgrade(move |client| relay(state, selection.request_id, connected, started, client)))
}

fn required_query_model(query: &HashMap<String, String>) -> Result<&str, LlmProxyError> {
    query
        .get("model")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| LlmProxyError::InvalidRequest("websocket request must include model query parameter".into()))
}

async fn record_streaming_attempt(state: &LlmProxyState, request_id: &str, candidate: &ProxyCandidate, retry_index: i32) -> Result<(), LlmProxyError> {
    record_attempt(
        state,
        request_id,
        AttemptRecordInput {
            candidate,
            retry_index,
            status: "streaming",
            status_code: Some(200),
            usage: None,
            latency_ms: None,
            first_byte_time_ms: None,
            error_type: None,
            error_message: None,
            response_body: None,
            finished: false,
        },
    )
    .await
}

async fn remember_affinity(state: &LlmProxyState, candidate: &ProxyCandidate) -> Result<(), LlmProxyError> {
    let Some(token_id) = candidate.trace.token_id.as_deref() else {
        return Ok(());
    };
    state
        .remember_affinity_key(
            token_id,
            &candidate.trace.global_model_id,
            &candidate.trace.client_api_format,
            &candidate.trace.key_id,
            candidate.cache_ttl_minutes,
        )
        .await
}

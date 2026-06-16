mod connect;
mod relay;

use std::collections::HashMap;
use std::time::Instant;

use axum::{
    extract::{Extension, Query, State, ws::WebSocketUpgrade},
    http::HeaderMap,
    response::Response,
};
use types::{model::PatchField, provider::RoutingRequestFeatures};

use self::{
    connect::{ConnectedUpstream, connect_first_upstream},
    relay::relay,
};
use super::{
    CurrentApiToken, LlmProxyError, LlmProxyState, OPENAI_CHAT_FORMAT, SetAffinityInput,
    audit::{AttemptRecordInput, SKIP_REASON_REQUEST_TERMINATED, record_attempt, record_scheduled_candidates, record_skipped_candidates},
    billing::enforce_preflight_access,
    candidate::{CandidateRequest, ProxyCandidate, select_candidates},
    proxy::capture::RequestCapture,
    rate_limit,
};

pub async fn realtime(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    websocket: WebSocketUpgrade,
) -> Result<Response, LlmProxyError> {
    let model_name = required_query_model(&query)?;
    enforce_preflight_access(&state, &token.0).await?;
    rate_limit::enforce_request_limits(&state, &token.0).await?;
    let selection = select_candidates(
        &state,
        &token.0,
        CandidateRequest {
            api_format: OPENAI_CHAT_FORMAT,
            routing_api_format: OPENAI_CHAT_FORMAT,
            model_name,
            is_stream: true,
            has_openai_responses_custom_tool_items: false,
            required_capability: None,
            features: RoutingRequestFeatures::unknown(OPENAI_CHAT_FORMAT, true, None),
        },
    )
    .await?;
    let request_body = serde_json::to_value(&query).map_err(|error| LlmProxyError::Infrastructure(error.to_string()))?;
    let capture = RequestCapture::new(&headers, &request_body);
    record_scheduled_candidates(&state, &selection, &capture).await?;
    let connected = connect_first_upstream(&state, &selection, &query).await?;
    let started = Instant::now();
    record_streaming_attempt(&state, &selection.request_id, &connected).await?;
    remember_affinity(&state, &connected.candidate, selection.cache_affinity_ttl_minutes).await?;
    record_skipped_candidates(&state, &selection.request_id, SKIP_REASON_REQUEST_TERMINATED).await?;
    Ok(websocket.on_upgrade(move |client| relay(state, selection.request_id, connected, started, client)))
}

fn required_query_model(query: &HashMap<String, String>) -> Result<&str, LlmProxyError> {
    query
        .get("model")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| LlmProxyError::InvalidRequest("websocket request must include model query parameter".into()))
}

async fn record_streaming_attempt(state: &LlmProxyState, request_id: &str, connected: &ConnectedUpstream) -> Result<(), LlmProxyError> {
    record_attempt(
        state,
        request_id,
        AttemptRecordInput {
            status: "streaming",
            status_code: Some(101),
            provider_request_headers: PatchField::Value(connected.request_headers.clone()),
            provider_response_headers: PatchField::Value(connected.response_headers.clone()),
            client_response_headers: PatchField::Null,
            client_response_body: PatchField::Null,
            ..AttemptRecordInput::new(&connected.candidate, connected.retry_index, "pending", false)
        },
    )
    .await
}

async fn remember_affinity(state: &LlmProxyState, candidate: &ProxyCandidate, ttl_minutes: i64) -> Result<(), LlmProxyError> {
    let Some(input) = set_affinity_input(candidate, ttl_minutes) else {
        return Ok(());
    };
    state.remember_affinity(input).await
}

fn set_affinity_input(candidate: &ProxyCandidate, ttl_minutes: i64) -> Option<SetAffinityInput<'_>> {
    Some(SetAffinityInput {
        token_id: candidate.trace.token_id.as_deref()?,
        model_id: &candidate.trace.global_model_id,
        api_format: &candidate.trace.client_api_format,
        provider_id: &candidate.trace.provider_id,
        endpoint_id: &candidate.trace.endpoint_id,
        key_id: &candidate.trace.key_id,
        ttl_minutes,
    })
}

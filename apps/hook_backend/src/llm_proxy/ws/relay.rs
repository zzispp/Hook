use std::time::Instant;

use axum::extract::ws::{Message as ClientMessage, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message as UpstreamMessage;

use super::connect::ConnectedUpstream;
use crate::llm_proxy::{
    LlmProxyError, LlmProxyState,
    audit::{AttemptRecordInput, record_attempt},
    candidate::ProxyCandidate,
};

pub(super) async fn relay(state: LlmProxyState, request_id: String, connected: ConnectedUpstream, started: Instant, client: WebSocket) {
    let candidate = connected.candidate;
    let retry_index = connected.retry_index;
    let upstream = connected.stream;
    let (mut client_sender, mut client_receiver) = client.split();
    let (mut upstream_sender, mut upstream_receiver) = upstream.split();
    let client_to_upstream = async move {
        while let Some(result) = client_receiver.next().await {
            let Ok(message) = result else { break };
            if upstream_sender.send(client_message(message)).await.is_err() {
                break;
            }
        }
    };
    let upstream_to_client = async move {
        let mut outcome = RelayOutcome::Success;
        while let Some(result) = upstream_receiver.next().await {
            let Ok(message) = result else {
                outcome = RelayOutcome::Failed("upstream websocket read failed".into());
                break;
            };
            if let Some(error_message) = upstream_error_message(&message) {
                outcome = RelayOutcome::Failed(error_message);
            }
            if client_sender.send(upstream_message(message)).await.is_err() {
                break;
            }
            if matches!(outcome, RelayOutcome::Failed(_)) {
                break;
            }
        }
        outcome
    };
    let outcome = tokio::select! {
        _ = client_to_upstream => RelayOutcome::Success,
        outcome = upstream_to_client => outcome,
    };
    if let Err(error) = finish_relay(state, request_id, candidate, retry_index, started, outcome).await {
        hook_tracing::warn_with_fields!("failed to finish websocket request candidate", error = error);
    }
}

async fn finish_relay(
    state: LlmProxyState,
    request_id: String,
    candidate: ProxyCandidate,
    retry_index: i32,
    started: Instant,
    outcome: RelayOutcome,
) -> Result<(), LlmProxyError> {
    let (status, error_type, error_message) = match &outcome {
        RelayOutcome::Success => ("success", None, None),
        RelayOutcome::Failed(message) => ("failed", Some("upstream_ws_error"), Some(message.as_str())),
    };
    record_attempt(
        &state,
        &request_id,
        AttemptRecordInput {
            candidate: &candidate,
            retry_index,
            status,
            status_code: Some(200),
            usage: None,
            latency_ms: Some(elapsed_ms(started)),
            first_byte_time_ms: None,
            error_type,
            error_message,
            response_body: None,
            finished: true,
        },
    )
    .await
}

fn elapsed_ms(started: Instant) -> i64 {
    started.elapsed().as_millis().try_into().unwrap_or(i64::MAX)
}

enum RelayOutcome {
    Success,
    Failed(String),
}

fn upstream_error_message(message: &UpstreamMessage) -> Option<String> {
    let UpstreamMessage::Text(text) = message else {
        return None;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return None;
    };
    if value.get("type").and_then(serde_json::Value::as_str) != Some("error") {
        return None;
    }
    let message = value
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("upstream websocket returned error");
    Some(message.to_owned())
}

fn client_message(message: ClientMessage) -> UpstreamMessage {
    match message {
        ClientMessage::Text(text) => UpstreamMessage::Text(text.to_string().into()),
        ClientMessage::Binary(bytes) => UpstreamMessage::Binary(bytes),
        ClientMessage::Ping(bytes) => UpstreamMessage::Ping(bytes),
        ClientMessage::Pong(bytes) => UpstreamMessage::Pong(bytes),
        ClientMessage::Close(_) => UpstreamMessage::Close(None),
    }
}

fn upstream_message(message: UpstreamMessage) -> ClientMessage {
    match message {
        UpstreamMessage::Text(text) => ClientMessage::Text(text.to_string().into()),
        UpstreamMessage::Binary(bytes) => ClientMessage::Binary(bytes),
        UpstreamMessage::Ping(bytes) => ClientMessage::Ping(bytes),
        UpstreamMessage::Pong(bytes) => ClientMessage::Pong(bytes),
        UpstreamMessage::Close(_) => ClientMessage::Close(None),
        UpstreamMessage::Frame(_) => ClientMessage::Close(None),
    }
}

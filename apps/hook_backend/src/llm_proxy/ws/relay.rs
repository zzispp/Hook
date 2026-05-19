use std::time::Instant;

use axum::extract::ws::{Message as ClientMessage, WebSocket};
use futures_util::{SinkExt, StreamExt};
use req::WebSocketMessage as UpstreamMessage;
use types::model::PatchField;

use super::connect::ConnectedUpstream;
use crate::llm_proxy::{
    LlmProxyError, LlmProxyState,
    audit::{AttemptRecordInput, TokenUsage, record_attempt},
    candidate::ProxyCandidate,
    proxy::usage,
};

pub(super) async fn relay(state: LlmProxyState, request_id: String, connected: ConnectedUpstream, started: Instant, client: WebSocket) {
    let candidate = connected.candidate;
    let retry_index = connected.retry_index;
    let upstream = connected.stream;
    let (mut client_sender, mut client_receiver) = client.split();
    let (mut upstream_sender, mut upstream_receiver) = upstream.split();
    let client_to_upstream = async move {
        while let Some(result) = client_receiver.next().await {
            let Ok(message) = result else {
                return RelayOutcome::Cancelled;
            };
            if matches!(message, ClientMessage::Close(_)) {
                return RelayOutcome::Cancelled;
            }
            if upstream_sender.send(client_message(message)).await.is_err() {
                return RelayOutcome::Failed("upstream websocket write failed".into());
            }
        }
        RelayOutcome::Cancelled
    };
    let upstream_to_client = async move {
        let mut usage_seen = None;
        while let Some(result) = upstream_receiver.next().await {
            let Ok(message) = result else {
                return RelayOutcome::Failed("upstream websocket read failed".into());
            };
            if let Some(error_message) = upstream_error_message(&message) {
                return RelayOutcome::Failed(error_message);
            }
            if let Some(usage) = realtime_usage(&message) {
                usage_seen = usage::merge(usage_seen, usage);
            }
            if client_sender.send(upstream_message(message)).await.is_err() {
                return RelayOutcome::Cancelled;
            }
        }
        RelayOutcome::Success { usage: Box::new(usage_seen) }
    };
    let outcome = tokio::select! {
        outcome = client_to_upstream => outcome,
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
    let latency_ms = Some(elapsed_ms(started));
    let input = match &outcome {
        RelayOutcome::Success { usage } => AttemptRecordInput {
            status_code: Some(101),
            usage: *usage.as_ref(),
            latency_ms,
            termination_origin: PatchField::Null,
            termination_reason: PatchField::Null,
            stream_end_reason: PatchField::Null,
            ..AttemptRecordInput::new(&candidate, retry_index, "success", true)
        },
        RelayOutcome::Cancelled => AttemptRecordInput {
            status_code: Some(499),
            latency_ms,
            error_type: Some("client_disconnected"),
            error_message: Some("client disconnected before websocket completed"),
            termination_origin: PatchField::Value("client".into()),
            termination_reason: PatchField::Value("disconnected".into()),
            stream_end_reason: PatchField::Value("client_disconnected".into()),
            ..AttemptRecordInput::new(&candidate, retry_index, "cancelled", true)
        },
        RelayOutcome::Failed(message) => AttemptRecordInput {
            status_code: Some(101),
            latency_ms,
            error_type: Some("upstream_ws_error"),
            error_message: Some(message.as_str()),
            termination_origin: PatchField::Null,
            termination_reason: PatchField::Null,
            stream_end_reason: PatchField::Null,
            ..AttemptRecordInput::new(&candidate, retry_index, "failed", true)
        },
    };
    record_attempt(&state, &request_id, input).await
}

fn elapsed_ms(started: Instant) -> i64 {
    started.elapsed().as_millis().try_into().unwrap_or(i64::MAX)
}

enum RelayOutcome {
    Success { usage: Box<Option<TokenUsage>> },
    Cancelled,
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

fn realtime_usage(message: &UpstreamMessage) -> Option<TokenUsage> {
    let UpstreamMessage::Text(text) = message else {
        return None;
    };
    let value = serde_json::from_str::<serde_json::Value>(text).ok()?;
    if value.get("type").and_then(serde_json::Value::as_str) != Some("response.done") {
        return None;
    }
    realtime_usage_payload(&value)
}

fn realtime_usage_payload(value: &serde_json::Value) -> Option<TokenUsage> {
    let object = value.get("response")?.get("usage")?.as_object()?;
    let input_details = object.get("input_token_details").or_else(|| object.get("input_tokens_details"));
    let output_details = object.get("output_token_details").or_else(|| object.get("output_tokens_details"));
    usage::merge(
        None,
        TokenUsage {
            prompt_tokens: number(object.get("input_tokens")),
            completion_tokens: number(object.get("output_tokens")),
            total_tokens: number(object.get("total_tokens")),
            cache_read_input_tokens: nested_number(input_details, "cached_tokens"),
            input_text_tokens: nested_number(input_details, "text_tokens"),
            input_audio_tokens: nested_number(input_details, "audio_tokens"),
            input_image_tokens: nested_number(input_details, "image_tokens"),
            output_text_tokens: nested_number(output_details, "text_tokens"),
            output_audio_tokens: nested_number(output_details, "audio_tokens"),
            output_image_tokens: nested_number(output_details, "image_tokens"),
            reasoning_tokens: nested_number(output_details, "reasoning_tokens"),
            usage_source: Some("openai"),
            usage_semantic: Some("realtime"),
            ..TokenUsage::default()
        },
    )
}

fn nested_number(value: Option<&serde_json::Value>, key: &str) -> Option<i64> {
    number(value?.get(key))
}

fn number(value: Option<&serde_json::Value>) -> Option<i64> {
    value?.as_i64().or_else(|| value?.as_u64().and_then(|number| i64::try_from(number).ok()))
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

#[cfg(test)]
mod tests {
    use req::WebSocketMessage;

    use super::realtime_usage;

    #[test]
    fn extracts_realtime_response_done_usage() {
        let message = WebSocketMessage::Text(
            r#"{
                "type": "response.done",
                "response": {
                    "usage": {
                        "input_tokens": 14,
                        "output_tokens": 6,
                        "total_tokens": 20,
                        "input_token_details": {
                            "cached_tokens": 3,
                            "text_tokens": 8,
                            "audio_tokens": 2
                        },
                        "output_token_details": {
                            "text_tokens": 4,
                            "audio_tokens": 1,
                            "reasoning_tokens": 1
                        }
                    }
                }
            }"#
            .into(),
        );

        let usage = realtime_usage(&message).expect("usage should be extracted");

        assert_eq!(usage.prompt_tokens, Some(14));
        assert_eq!(usage.completion_tokens, Some(6));
        assert_eq!(usage.total_tokens, Some(20));
        assert_eq!(usage.cache_read_input_tokens, Some(3));
        assert_eq!(usage.input_text_tokens, Some(8));
        assert_eq!(usage.input_audio_tokens, Some(2));
        assert_eq!(usage.output_text_tokens, Some(4));
        assert_eq!(usage.output_audio_tokens, Some(1));
        assert_eq!(usage.reasoning_tokens, Some(1));
        assert_eq!(usage.usage_source, Some("openai"));
        assert_eq!(usage.usage_semantic, Some("realtime"));
    }
}

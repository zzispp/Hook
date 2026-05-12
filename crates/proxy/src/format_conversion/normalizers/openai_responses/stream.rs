use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalStreamEvent, StreamConversionState};

use super::response::to_internal as response_to_internal;

pub fn to_internal(chunks: &[Value]) -> Result<Vec<InternalStreamEvent>, FormatConversionError> {
    let mut state = StreamConversionState::default();
    let mut events = Vec::new();
    for chunk in chunks {
        events.extend(chunk_to_internal(chunk, &mut state)?);
    }
    Ok(events)
}

pub fn from_internal(events: &[InternalStreamEvent]) -> Result<Vec<Value>, FormatConversionError> {
    let mut state = StreamConversionState::default();
    let mut output = Vec::new();
    for event in events {
        output.extend(event_from_internal(event, &mut state)?);
    }
    Ok(output)
}

pub fn chunk_to_internal(chunk: &Value, state: &mut StreamConversionState) -> Result<Vec<InternalStreamEvent>, FormatConversionError> {
    let mut events = Vec::new();
    parse_chunk(chunk, &mut state.openai_responses_started, &mut events)?;
    Ok(events)
}

pub fn event_from_internal(event: &InternalStreamEvent, state: &mut StreamConversionState) -> Result<Vec<Value>, FormatConversionError> {
    if state.target_openai_responses_id.is_empty() {
        state.target_openai_responses_id = "resp_unknown".to_owned();
    }
    if state.target_openai_responses_model.is_empty() {
        state.target_openai_responses_model = "openai-responses-unknown".to_owned();
    }
    let mut output = Vec::new();
    push_event(event, state, &mut output);
    Ok(output)
}

fn parse_chunk(chunk: &Value, started: &mut bool, output: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    match chunk.get("type").and_then(Value::as_str).unwrap_or_default() {
        "response.created" => {
            let response = chunk.get("response");
            output.push(InternalStreamEvent::Start {
                id: nested_string(response, "id"),
                model: nested_string(response, "model"),
            });
            *started = true;
        }
        "response.output_text.delta" => push_delta(chunk, started, output),
        "response.completed" => push_completed(chunk, output)?,
        _ => {}
    }
    Ok(())
}

fn push_delta(chunk: &Value, started: &mut bool, output: &mut Vec<InternalStreamEvent>) {
    if !*started {
        output.push(InternalStreamEvent::Start { id: None, model: None });
        *started = true;
    }
    if let Some(delta) = chunk.get("delta").and_then(Value::as_str).filter(|value| !value.is_empty()) {
        output.push(InternalStreamEvent::TextDelta(delta.to_owned()));
    }
}

fn push_completed(chunk: &Value, output: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    let usage = match chunk.get("response") {
        Some(response) => response_to_internal(response)?.usage,
        None => None,
    };
    output.push(InternalStreamEvent::Done { reason: None, usage });
    Ok(())
}

fn push_event(event: &InternalStreamEvent, state: &mut StreamConversionState, output: &mut Vec<Value>) {
    match event {
        InternalStreamEvent::Start {
            id: event_id,
            model: event_model,
        } => {
            state.target_openai_responses_id = event_id.clone().unwrap_or_else(|| state.target_openai_responses_id.clone());
            state.target_openai_responses_model = event_model.clone().unwrap_or_else(|| state.target_openai_responses_model.clone());
            output
                .push(json!({"type": "response.created", "response": {"id": state.target_openai_responses_id, "model": state.target_openai_responses_model}}));
        }
        InternalStreamEvent::TextDelta(text) => output.push(json!({"type": "response.output_text.delta", "delta": text})),
        InternalStreamEvent::Done { .. } => {
            output.push(
                json!({"type": "response.completed", "response": {"id": state.target_openai_responses_id, "model": state.target_openai_responses_model}}),
            );
        }
    }
}

fn nested_string(value: Option<&Value>, key: &str) -> Option<String> {
    value?.get(key).and_then(Value::as_str).map(str::to_owned)
}

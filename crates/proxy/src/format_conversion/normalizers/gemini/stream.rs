use serde_json::Value;

use crate::format_conversion::{FormatConversionError, InternalStreamEvent, StreamConversionState};

use super::common::{content_chunk, map_gemini_stop_reason, optional_string, parts_text, required_array, required_object, usage_from_gemini};

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
    parse_chunk(chunk, state, &mut events)?;
    Ok(events)
}

pub fn event_from_internal(event: &InternalStreamEvent, state: &mut StreamConversionState) -> Result<Vec<Value>, FormatConversionError> {
    if state.target_gemini_model.is_empty() {
        state.target_gemini_model = "gemini-unknown".to_owned();
    }
    let mut output = Vec::new();
    push_stream_event(event, state, &mut output);
    Ok(output)
}

fn parse_chunk(chunk: &Value, state: &mut StreamConversionState, events: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    let model = optional_string(chunk, "modelVersion");
    if !state.gemini_started {
        events.push(InternalStreamEvent::Start {
            id: Some("gemini_1".to_owned()),
            model: model.clone(),
        });
        state.gemini_started = true;
    }
    let candidate = first_candidate(chunk)?;
    let text = candidate_text(candidate)?;
    let delta = text.strip_prefix(state.gemini_previous_text.as_str()).unwrap_or(&text);
    if !delta.is_empty() {
        events.push(InternalStreamEvent::TextDelta(delta.to_owned()));
    }
    state.gemini_previous_text = text;
    if let Some(reason) = candidate.get("finishReason").and_then(Value::as_str) {
        events.push(InternalStreamEvent::Done {
            reason: Some(map_gemini_stop_reason(reason)),
            usage: usage_from_gemini(chunk.get("usageMetadata")),
        });
    }
    Ok(())
}

fn push_stream_event(event: &InternalStreamEvent, state: &mut StreamConversionState, output: &mut Vec<Value>) {
    match event {
        InternalStreamEvent::Start { model: event_model, .. } => {
            state.target_gemini_model = event_model.clone().unwrap_or_else(|| state.target_gemini_model.clone());
        }
        InternalStreamEvent::TextDelta(text) => output.push(content_chunk(text, &state.target_gemini_model, None, None)),
        InternalStreamEvent::Done { reason, usage } => output.push(content_chunk("", &state.target_gemini_model, reason.as_ref(), usage.as_ref())),
    }
}

fn first_candidate(value: &Value) -> Result<&serde_json::Map<String, Value>, FormatConversionError> {
    let candidates = required_array(value, "candidates", "$.candidates")?;
    let first = candidates
        .first()
        .ok_or_else(|| FormatConversionError::invalid_payload("gemini", "$.candidates[0]"))?;
    required_object(Some(first), "$.candidates[0]")
}

fn candidate_text(candidate: &serde_json::Map<String, Value>) -> Result<String, FormatConversionError> {
    let content = required_object(candidate.get("content"), "$.candidates[0].content")?;
    parts_text(content.get("parts"), "$.candidates[0].content.parts")
}

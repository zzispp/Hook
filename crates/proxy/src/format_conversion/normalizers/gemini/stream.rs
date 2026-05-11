use serde_json::Value;

use crate::format_conversion::{FormatConversionError, InternalStreamEvent};

use super::common::{content_chunk, map_gemini_stop_reason, optional_string, parts_text, required_array, required_object, usage_from_gemini};

pub fn to_internal(chunks: &[Value]) -> Result<Vec<InternalStreamEvent>, FormatConversionError> {
    let mut events = Vec::new();
    let mut started = false;
    let mut previous_text = String::new();
    for chunk in chunks {
        parse_chunk(chunk, &mut started, &mut previous_text, &mut events)?;
    }
    Ok(events)
}

pub fn from_internal(events: &[InternalStreamEvent]) -> Result<Vec<Value>, FormatConversionError> {
    let mut output = Vec::new();
    let mut model = "gemini-unknown".to_owned();
    for event in events {
        push_stream_event(event, &mut model, &mut output);
    }
    Ok(output)
}

fn parse_chunk(chunk: &Value, started: &mut bool, previous_text: &mut String, events: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    let model = optional_string(chunk, "modelVersion");
    if !*started {
        events.push(InternalStreamEvent::Start {
            id: Some("gemini_1".to_owned()),
            model: model.clone(),
        });
        *started = true;
    }
    let candidate = first_candidate(chunk)?;
    let text = candidate_text(candidate)?;
    let delta = text.strip_prefix(previous_text.as_str()).unwrap_or(&text);
    if !delta.is_empty() {
        events.push(InternalStreamEvent::TextDelta(delta.to_owned()));
    }
    *previous_text = text;
    if let Some(reason) = candidate.get("finishReason").and_then(Value::as_str) {
        events.push(InternalStreamEvent::Done {
            reason: Some(map_gemini_stop_reason(reason)),
            usage: usage_from_gemini(chunk.get("usageMetadata")),
        });
    }
    Ok(())
}

fn push_stream_event(event: &InternalStreamEvent, model: &mut String, output: &mut Vec<Value>) {
    match event {
        InternalStreamEvent::Start { model: event_model, .. } => {
            *model = event_model.clone().unwrap_or_else(|| model.clone());
        }
        InternalStreamEvent::TextDelta(text) => output.push(content_chunk(text, model, None, None)),
        InternalStreamEvent::Done { reason, usage } => output.push(content_chunk("", model, reason.as_ref(), usage.as_ref())),
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

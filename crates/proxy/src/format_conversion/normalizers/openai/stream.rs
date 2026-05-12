use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalStreamEvent, InternalUsage, StreamConversionState};

use super::common::{first_choice, map_openai_stop_reason, openai_finish_reason, optional_string, optional_string_value, required_object, usage_from_openai};

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
    parse_stream_chunk(chunk, &mut state.openai_started, &mut events)?;
    Ok(events)
}

pub fn event_from_internal(event: &InternalStreamEvent, state: &mut StreamConversionState) -> Result<Vec<Value>, FormatConversionError> {
    if state.target_openai_id.is_empty() {
        state.target_openai_id = "chatcmpl_unknown".to_owned();
    }
    if state.target_openai_model.is_empty() {
        state.target_openai_model = "openai-unknown".to_owned();
    }
    let mut output = Vec::new();
    push_stream_event(event, state, &mut output);
    Ok(output)
}

fn parse_stream_chunk(chunk: &Value, started: &mut bool, events: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    let id = optional_string(chunk, "id");
    let model = optional_string(chunk, "model");
    let choice = first_choice(chunk, "$.choices")?;
    let delta = required_object(choice.get("delta"), "$.choices[0].delta")?;
    emit_start_if_needed(started, events, &id, &model, delta.get("role").and_then(Value::as_str) == Some("assistant"));
    if let Some(content) = delta.get("content").and_then(Value::as_str) {
        emit_start_if_needed(started, events, &id, &model, true);
        if !content.is_empty() {
            events.push(InternalStreamEvent::TextDelta(content.to_owned()));
        }
    }
    if let Some(reason) = optional_string_value(choice.get("finish_reason")) {
        events.push(InternalStreamEvent::Done {
            reason: Some(map_openai_stop_reason(&reason)),
            usage: usage_from_openai(chunk.get("usage")),
        });
    }
    Ok(())
}

fn emit_start_if_needed(started: &mut bool, events: &mut Vec<InternalStreamEvent>, id: &Option<String>, model: &Option<String>, should_start: bool) {
    if should_start && !*started {
        events.push(InternalStreamEvent::Start {
            id: id.clone(),
            model: model.clone(),
        });
        *started = true;
    }
}

fn push_stream_event(event: &InternalStreamEvent, state: &mut StreamConversionState, output: &mut Vec<Value>) {
    match event {
        InternalStreamEvent::Start {
            id: event_id,
            model: event_model,
        } => {
            state.target_openai_id = event_id.clone().unwrap_or_else(|| state.target_openai_id.clone());
            state.target_openai_model = event_model.clone().unwrap_or_else(|| state.target_openai_model.clone());
            output.push(openai_stream_chunk(
                &state.target_openai_id,
                &state.target_openai_model,
                json!({"role": "assistant"}),
                None,
                None,
            ));
        }
        InternalStreamEvent::TextDelta(text) => {
            output.push(openai_stream_chunk(
                &state.target_openai_id,
                &state.target_openai_model,
                json!({"content": text}),
                None,
                None,
            ));
        }
        InternalStreamEvent::Done { reason, usage } => {
            let finish_reason = reason.as_ref().map(openai_finish_reason);
            output.push(openai_stream_chunk(
                &state.target_openai_id,
                &state.target_openai_model,
                json!({}),
                finish_reason,
                usage_json(usage.as_ref()),
            ));
        }
    }
}

fn usage_json(usage: Option<&InternalUsage>) -> Option<Value> {
    let complete = usage.cloned()?.with_total();
    Some(json!({
        "prompt_tokens": complete.prompt_tokens,
        "completion_tokens": complete.completion_tokens,
        "total_tokens": complete.total_tokens,
    }))
}

fn openai_stream_chunk(id: &str, model: &str, delta: Value, finish_reason: Option<&str>, usage: Option<Value>) -> Value {
    let mut chunk = json!({
        "id": id,
        "model": model,
        "object": "chat.completion.chunk",
        "choices": [{
            "index": 0,
            "delta": delta,
            "finish_reason": finish_reason,
        }]
    });
    if let Some(usage_payload) = usage {
        chunk["usage"] = usage_payload;
    }
    chunk
}

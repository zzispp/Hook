use serde_json::Value;

use crate::format_conversion::{FormatConversionError, InternalStreamEvent, PendingStreamDone, StreamConversionState};

use super::common::{first_choice, map_openai_stop_reason, optional_string, optional_string_value, required_object, usage_from_openai};

pub(super) fn chunk_to_internal(chunk: &Value, state: &mut StreamConversionState) -> Result<Vec<InternalStreamEvent>, FormatConversionError> {
    let mut events = Vec::new();
    parse_stream_chunk(chunk, state, &mut events)?;
    Ok(events)
}

pub(super) fn flush_to_internal(state: &mut StreamConversionState) -> Vec<InternalStreamEvent> {
    match state.openai_pending_done.take() {
        Some(done) => vec![InternalStreamEvent::Done {
            reason: done.reason,
            usage: done.usage,
        }],
        None => Vec::new(),
    }
}

fn parse_stream_chunk(chunk: &Value, state: &mut StreamConversionState, events: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    if openai_choices_empty(chunk)? {
        merge_pending_usage(chunk, state);
        events.extend(flush_to_internal(state));
        return Ok(());
    }
    let id = optional_string(chunk, "id");
    let model = optional_string(chunk, "model");
    let choice = first_choice(chunk, "$.choices")?;
    let delta = required_object(choice.get("delta"), "$.choices[0].delta")?;
    emit_start_if_needed(
        &mut state.openai_started,
        events,
        &id,
        &model,
        delta.get("role").and_then(Value::as_str) == Some("assistant"),
    );
    emit_text_delta(delta, &id, &model, state, events);
    emit_tool_call_events(delta.get("tool_calls"), state, events);
    close_if_finished(choice, chunk, state, events);
    Ok(())
}

fn emit_text_delta(
    delta: &serde_json::Map<String, Value>,
    id: &Option<String>,
    model: &Option<String>,
    state: &mut StreamConversionState,
    events: &mut Vec<InternalStreamEvent>,
) {
    let Some(content) = delta.get("content").and_then(Value::as_str) else {
        return;
    };
    emit_start_if_needed(&mut state.openai_started, events, id, model, true);
    if !content.is_empty() {
        start_openai_text_block(state, events);
        events.push(InternalStreamEvent::TextDelta(content.to_owned()));
    }
}

fn close_if_finished(choice: &serde_json::Map<String, Value>, chunk: &Value, state: &mut StreamConversionState, events: &mut Vec<InternalStreamEvent>) {
    let Some(reason) = optional_string_value(choice.get("finish_reason")) else {
        return;
    };
    close_openai_text_blocks(state, events);
    close_tool_blocks(state, events);
    state.openai_pending_done = Some(PendingStreamDone {
        reason: Some(map_openai_stop_reason(&reason)),
        usage: usage_from_openai(chunk.get("usage")),
    });
    if chunk.get("usage").is_some() {
        events.extend(flush_to_internal(state));
    }
}

fn start_openai_text_block(state: &mut StreamConversionState, events: &mut Vec<InternalStreamEvent>) {
    if state.openai_text_started {
        return;
    }
    let index = reserve_openai_block_index(&mut state.openai_text_block_index, &mut state.openai_next_block_index);
    state.openai_text_started = true;
    events.push(InternalStreamEvent::ContentBlockStart {
        index,
        block: crate::format_conversion::InternalContentBlock::text(String::new()),
    });
}

fn close_openai_text_blocks(state: &mut StreamConversionState, events: &mut Vec<InternalStreamEvent>) {
    if state.openai_text_started && !state.openai_text_stopped {
        state.openai_text_stopped = true;
        if let Some(index) = state.openai_text_block_index {
            events.push(InternalStreamEvent::ContentBlockStop { index });
        }
    }
    if state.openai_thinking_started && !state.openai_thinking_stopped {
        state.openai_thinking_stopped = true;
        if let Some(index) = state.openai_thinking_block_index {
            events.push(InternalStreamEvent::ContentBlockStop { index });
        }
    }
}

fn reserve_openai_block_index(slot: &mut Option<u32>, next: &mut u32) -> u32 {
    if let Some(index) = *slot {
        return index;
    }
    let index = *next;
    *next = next.saturating_add(1);
    *slot = Some(index);
    index
}

fn emit_tool_call_events(value: Option<&Value>, state: &mut StreamConversionState, events: &mut Vec<InternalStreamEvent>) {
    let Some(calls) = value.and_then(Value::as_array) else {
        return;
    };
    for call in calls {
        emit_tool_call_event(call, state, events);
    }
}

fn emit_tool_call_event(call: &Value, state: &mut StreamConversionState, events: &mut Vec<InternalStreamEvent>) {
    let Some(object) = call.as_object() else {
        return;
    };
    let tool_index = object
        .get("index")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(0);
    let function = object.get("function").and_then(Value::as_object);
    let id = object.get("id").and_then(Value::as_str).map(str::to_owned);
    let name = function.and_then(|value| value.get("name")).and_then(Value::as_str).map(str::to_owned);
    let arguments_delta = function.and_then(|value| value.get("arguments")).and_then(Value::as_str).unwrap_or_default();
    let block_index = ensure_tool_block(tool_index, id.as_deref(), name.as_deref(), state, events);
    if !arguments_delta.is_empty() {
        events.push(InternalStreamEvent::ToolCallDelta {
            index: block_index,
            id,
            name,
            arguments_delta: arguments_delta.to_owned(),
        });
    }
}

fn ensure_tool_block(tool_index: u32, id: Option<&str>, name: Option<&str>, state: &mut StreamConversionState, events: &mut Vec<InternalStreamEvent>) -> u32 {
    if let Some(position) = state.openai_tool_blocks.iter().position(|tool| tool.tool_index == tool_index) {
        update_tool_block(position, id, name, state);
        return state.openai_tool_blocks[position].block_index;
    }
    let block_index = state.openai_next_block_index;
    state.openai_next_block_index = state.openai_next_block_index.saturating_add(1);
    state.openai_tool_blocks.push(crate::format_conversion::OpenAiToolStreamItem {
        tool_index,
        block_index,
        id: id.unwrap_or_default().to_owned(),
        name: name.unwrap_or_default().to_owned(),
    });
    events.push(InternalStreamEvent::ContentBlockStart {
        index: block_index,
        block: crate::format_conversion::InternalContentBlock::ToolUse {
            id: id.unwrap_or_default().to_owned(),
            name: name.unwrap_or_default().to_owned(),
            input: serde_json::json!({}),
            kind: crate::format_conversion::InternalToolKind::Function,
        },
    });
    block_index
}

fn update_tool_block(position: usize, id: Option<&str>, name: Option<&str>, state: &mut StreamConversionState) {
    let tool = &mut state.openai_tool_blocks[position];
    if let Some(id) = id.filter(|value| !value.is_empty()) {
        tool.id = id.to_owned();
    }
    if let Some(name) = name.filter(|value| !value.is_empty()) {
        tool.name = name.to_owned();
    }
}

fn close_tool_blocks(state: &mut StreamConversionState, events: &mut Vec<InternalStreamEvent>) {
    for tool in state.openai_tool_blocks.drain(..) {
        events.push(InternalStreamEvent::ContentBlockStop { index: tool.block_index });
    }
}

fn openai_choices_empty(chunk: &Value) -> Result<bool, FormatConversionError> {
    Ok(super::common::required_array(chunk, "choices", "$.choices")?.is_empty())
}

fn merge_pending_usage(chunk: &Value, state: &mut StreamConversionState) {
    let usage = usage_from_openai(chunk.get("usage"));
    match state.openai_pending_done.as_mut() {
        Some(done) => done.usage = usage.or_else(|| done.usage.take()),
        None if usage.is_some() => {
            state.openai_pending_done = Some(PendingStreamDone { reason: None, usage });
        }
        None => {}
    }
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

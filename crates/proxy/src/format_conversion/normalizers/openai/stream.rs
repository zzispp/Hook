use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalStreamEvent, InternalUsage, PendingStreamDone, StreamConversionState};

use super::common::{first_choice, map_openai_stop_reason, openai_finish_reason, optional_string, optional_string_value, required_object, usage_from_openai};

pub fn to_internal(chunks: &[Value]) -> Result<Vec<InternalStreamEvent>, FormatConversionError> {
    let mut state = StreamConversionState::default();
    let mut events = Vec::new();
    for chunk in chunks {
        events.extend(chunk_to_internal(chunk, &mut state)?);
    }
    events.extend(flush_to_internal(&mut state));
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
    parse_stream_chunk(chunk, state, &mut events)?;
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

pub fn flush_to_internal(state: &mut StreamConversionState) -> Vec<InternalStreamEvent> {
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
    if let Some(content) = delta.get("content").and_then(Value::as_str) {
        emit_start_if_needed(&mut state.openai_started, events, &id, &model, true);
        if !content.is_empty() {
            start_openai_text_block(state, events);
            events.push(InternalStreamEvent::TextDelta(content.to_owned()));
        }
    }
    emit_tool_call_events(delta.get("tool_calls"), state, events);
    if let Some(reason) = optional_string_value(choice.get("finish_reason")) {
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
    Ok(())
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
        let Some(object) = call.as_object() else {
            continue;
        };
        let tool_index = object
            .get("index")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(0);
        let function = object.get("function").and_then(Value::as_object);
        let id = object.get("id").and_then(Value::as_str).map(str::to_owned);
        let name = function.and_then(|value| value.get("name")).and_then(Value::as_str).map(str::to_owned);
        let arguments_delta = function
            .and_then(|value| value.get("arguments"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let block_index = ensure_tool_block(tool_index, id.as_deref(), name.as_deref(), state, events);
        if !arguments_delta.is_empty() {
            events.push(InternalStreamEvent::ToolCallDelta {
                index: block_index,
                id,
                name,
                arguments_delta,
            });
        }
    }
}

fn ensure_tool_block(tool_index: u32, id: Option<&str>, name: Option<&str>, state: &mut StreamConversionState, events: &mut Vec<InternalStreamEvent>) -> u32 {
    if let Some(position) = state.openai_tool_blocks.iter().position(|tool| tool.tool_index == tool_index) {
        let tool = &mut state.openai_tool_blocks[position];
        if let Some(id) = id.filter(|value| !value.is_empty()) {
            tool.id = id.to_owned();
        }
        if let Some(name) = name.filter(|value| !value.is_empty()) {
            tool.name = name.to_owned();
        }
        return tool.block_index;
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
        },
    });
    block_index
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
        InternalStreamEvent::ThinkingDelta { text, .. } => output.push(openai_stream_chunk(
            &state.target_openai_id,
            &state.target_openai_model,
            json!({"reasoning_content": text}),
            None,
            None,
        )),
        InternalStreamEvent::ToolCallDelta {
            index,
            id,
            name,
            arguments_delta,
        } => output.push(openai_stream_chunk(
            &state.target_openai_id,
            &state.target_openai_model,
            json!({"tool_calls": [{
                "index": index,
                "id": id,
                "type": "function",
                "function": { "name": name, "arguments": arguments_delta },
            }]}),
            None,
            None,
        )),
        InternalStreamEvent::Usage(usage) => output.push(openai_stream_chunk(
            &state.target_openai_id,
            &state.target_openai_model,
            json!({}),
            None,
            usage_json(Some(usage)),
        )),
        InternalStreamEvent::ContentBlockStart { .. } | InternalStreamEvent::ContentBlockStop { .. } => {}
        InternalStreamEvent::Error(error) => output.push(json!({ "error": {
            "message": error.message,
            "type": error.error_type,
            "code": error.code,
            "param": error.param,
        }})),
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
        "prompt_tokens_details": {
            "cached_tokens": complete.cache_read_tokens,
            "cache_creation_tokens": complete.cache_creation_tokens,
        },
        "completion_tokens_details": {
            "reasoning_tokens": complete.reasoning_tokens,
        },
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

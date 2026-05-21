use serde_json::Value;

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalStreamEvent, StreamConversionState};

use super::{response::usage_from_response, stream_output};

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
    Ok(stream_output::event_from_internal(event, state))
}

fn parse_chunk(chunk: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    if chunk.get("error").is_some() {
        output.push(InternalStreamEvent::Error(crate::format_conversion::error_codec::to_internal(chunk, None)));
        return Ok(());
    }
    match chunk.get("type").and_then(Value::as_str).unwrap_or_default() {
        "response.created" => push_start(chunk, state, output),
        "response.output_item.added" => push_output_item_start(chunk, state, output),
        "response.output_item.done" => push_output_item_done(chunk, state, output),
        "response.output_text.delta" => push_delta(chunk, state, output),
        "response.output_text.done" => push_text_done(state, output),
        "response.function_call_arguments.delta" => push_function_call_delta(chunk, state, output),
        "response.function_call_arguments.done" => push_function_call_done(chunk, state, output),
        "response.failed" => output.push(InternalStreamEvent::Error(crate::format_conversion::error_codec::to_internal(chunk, None))),
        "response.completed" => push_completed(chunk, state, output)?,
        _ => {}
    }
    Ok(())
}

fn push_start(chunk: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    let response = chunk.get("response");
    output.push(InternalStreamEvent::Start {
        id: nested_string(response, "id"),
        model: nested_string(response, "model"),
    });
    state.openai_responses_started = true;
}

fn push_delta(chunk: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    if !state.openai_responses_started {
        output.push(InternalStreamEvent::Start { id: None, model: None });
        state.openai_responses_started = true;
    }
    if let Some(delta) = chunk.get("delta").and_then(Value::as_str).filter(|value| !value.is_empty()) {
        start_text_block(state, output);
        output.push(InternalStreamEvent::TextDelta(delta.to_owned()));
    }
}

fn push_text_done(state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    if state.openai_responses_text_started && !state.openai_responses_text_stopped {
        state.openai_responses_text_stopped = true;
        if let Some(index) = state.openai_responses_text_block_index {
            output.push(InternalStreamEvent::ContentBlockStop { index });
        }
    }
}

fn start_text_block(state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    if state.openai_responses_text_started {
        return;
    }
    let index = reserve_block_index(
        &mut state.openai_responses_text_block_index,
        &mut state.openai_responses_next_source_block_index,
    );
    state.openai_responses_text_started = true;
    output.push(InternalStreamEvent::ContentBlockStart {
        index,
        block: InternalContentBlock::text(String::new()),
    });
}

fn push_output_item_start(chunk: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    let Some(item) = chunk.get("item") else {
        return;
    };
    match item.get("type").and_then(Value::as_str) {
        Some("reasoning") => push_reasoning_start(chunk, item, output),
        Some("function_call") => push_function_call_start(item, state, output),
        _ => {}
    }
}

fn push_reasoning_start(chunk: &Value, item: &Value, output: &mut Vec<InternalStreamEvent>) {
    output.push(InternalStreamEvent::ContentBlockStart {
        index: chunk
            .get("output_index")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(0),
        block: InternalContentBlock::Thinking {
            text: String::new(),
            signature: item.get("encrypted_content").and_then(Value::as_str).map(str::to_owned),
        },
    });
}

fn push_function_call_start(item: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    let item_id = item.get("id").and_then(Value::as_str).unwrap_or_default().to_owned();
    let call_id = item
        .get("call_id")
        .or_else(|| item.get("id"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let name = item.get("name").and_then(Value::as_str).unwrap_or_default().to_owned();
    let block_index = state.openai_responses_next_source_block_index;
    state.openai_responses_next_source_block_index = state.openai_responses_next_source_block_index.saturating_add(1);
    state
        .openai_responses_source_tools
        .push(crate::format_conversion::OpenAiResponsesSourceToolStreamItem {
            item_id,
            call_id: call_id.clone(),
            block_index,
            name: name.clone(),
            arguments: String::new(),
            stopped: false,
        });
    output.push(InternalStreamEvent::ContentBlockStart {
        index: block_index,
        block: InternalContentBlock::ToolUse {
            id: call_id,
            name,
            input: serde_json::json!({}),
        },
    });
    sync_function_call_arguments(item.get("arguments"), block_index, state, output);
}

fn push_function_call_delta(chunk: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    let delta = chunk.get("delta").and_then(Value::as_str).unwrap_or_default();
    if delta.is_empty() {
        return;
    }
    let position = tool_position(chunk, state).unwrap_or_else(|| create_tool_from_chunk(chunk, state, output));
    let tool = &mut state.openai_responses_source_tools[position];
    tool.arguments.push_str(delta);
    output.push(InternalStreamEvent::ToolCallDelta {
        index: tool.block_index,
        id: Some(tool.call_id.clone()),
        name: Some(tool.name.clone()),
        arguments_delta: delta.to_owned(),
    });
}

fn push_function_call_done(chunk: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    let position = tool_position(chunk, state).unwrap_or_else(|| create_tool_from_chunk(chunk, state, output));
    let block_index = state.openai_responses_source_tools[position].block_index;
    sync_function_call_arguments(chunk.get("arguments"), block_index, state, output);
}

fn push_output_item_done(chunk: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    let Some(item) = chunk
        .get("item")
        .filter(|value| value.get("type").and_then(Value::as_str) == Some("function_call"))
    else {
        return;
    };
    let position = tool_position_for_ref(item.get("call_id").or_else(|| item.get("id")), state).unwrap_or_else(|| create_tool_from_item(item, state, output));
    let block_index = state.openai_responses_source_tools[position].block_index;
    sync_function_call_arguments(item.get("arguments"), block_index, state, output);
    stop_tool_block(position, state, output);
}

fn push_completed(chunk: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    push_text_done(state, output);
    stop_open_tools(state, output);
    let usage = chunk
        .get("response")
        .and_then(|response| usage_from_response(response.get("usage")))
        .or_else(|| usage_from_response(chunk.get("usage")));
    let reason = state
        .openai_responses_source_tools
        .is_empty()
        .then_some(crate::format_conversion::StopReason::EndTurn)
        .or(Some(crate::format_conversion::StopReason::ToolUse));
    output.push(InternalStreamEvent::Done { reason, usage });
    Ok(())
}

fn sync_function_call_arguments(value: Option<&Value>, block_index: u32, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    let Some(snapshot) = value.and_then(Value::as_str).filter(|value| !value.is_empty()) else {
        return;
    };
    let Some(position) = state.openai_responses_source_tools.iter().position(|tool| tool.block_index == block_index) else {
        return;
    };
    let current = state.openai_responses_source_tools[position].arguments.clone();
    if snapshot == current {
        return;
    }
    let delta = snapshot.strip_prefix(&current).unwrap_or(snapshot).to_owned();
    state.openai_responses_source_tools[position].arguments = snapshot.to_owned();
    if !delta.is_empty() {
        output.push(InternalStreamEvent::ToolCallDelta {
            index: block_index,
            id: Some(state.openai_responses_source_tools[position].call_id.clone()),
            name: Some(state.openai_responses_source_tools[position].name.clone()),
            arguments_delta: delta,
        });
    }
}

fn tool_position(chunk: &Value, state: &StreamConversionState) -> Option<usize> {
    tool_position_for_ref(chunk.get("item_id").or_else(|| chunk.get("call_id")).or_else(|| chunk.get("id")), state)
}

fn tool_position_for_ref(value: Option<&Value>, state: &StreamConversionState) -> Option<usize> {
    let reference = value.and_then(Value::as_str).unwrap_or_default();
    state
        .openai_responses_source_tools
        .iter()
        .position(|tool| !reference.is_empty() && (tool.item_id == reference || tool.call_id == reference))
}

fn create_tool_from_chunk(chunk: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) -> usize {
    let call_id = chunk.get("item_id").and_then(Value::as_str).unwrap_or_default();
    create_tool(call_id, call_id, "", state, output)
}

fn create_tool_from_item(item: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) -> usize {
    let item_id = item.get("id").and_then(Value::as_str).unwrap_or_default();
    let call_id = item.get("call_id").or_else(|| item.get("id")).and_then(Value::as_str).unwrap_or_default();
    let name = item.get("name").and_then(Value::as_str).unwrap_or_default();
    create_tool(item_id, call_id, name, state, output)
}

fn create_tool(item_id: &str, call_id: &str, name: &str, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) -> usize {
    let block_index = state.openai_responses_next_source_block_index;
    state.openai_responses_next_source_block_index = state.openai_responses_next_source_block_index.saturating_add(1);
    state
        .openai_responses_source_tools
        .push(crate::format_conversion::OpenAiResponsesSourceToolStreamItem {
            item_id: item_id.to_owned(),
            call_id: call_id.to_owned(),
            block_index,
            name: name.to_owned(),
            arguments: String::new(),
            stopped: false,
        });
    output.push(InternalStreamEvent::ContentBlockStart {
        index: block_index,
        block: InternalContentBlock::ToolUse {
            id: call_id.to_owned(),
            name: name.to_owned(),
            input: serde_json::json!({}),
        },
    });
    state.openai_responses_source_tools.len().saturating_sub(1)
}

fn stop_tool_block(position: usize, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    let Some(tool) = state.openai_responses_source_tools.get_mut(position) else {
        return;
    };
    if tool.stopped {
        return;
    }
    tool.stopped = true;
    output.push(InternalStreamEvent::ContentBlockStop { index: tool.block_index });
}

fn stop_open_tools(state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    for index in 0..state.openai_responses_source_tools.len() {
        stop_tool_block(index, state, output);
    }
}

fn reserve_block_index(slot: &mut Option<u32>, next: &mut u32) -> u32 {
    if let Some(index) = *slot {
        return index;
    }
    let index = *next;
    *next = next.saturating_add(1);
    *slot = Some(index);
    index
}

fn nested_string(value: Option<&Value>, key: &str) -> Option<String> {
    value?.get(key).and_then(Value::as_str).map(str::to_owned)
}

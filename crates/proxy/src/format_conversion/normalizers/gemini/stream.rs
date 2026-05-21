use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalStreamEvent, InternalToolKind, StreamConversionState};

use super::common::{
    complete_function_call_chunk, content_chunk, map_gemini_stop_reason, optional_string, parts_text, required_array, required_object, terminal_chunk,
    usage_from_gemini,
};

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
    reject_custom_tool_event(event)?;
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
    let has_parts = parse_candidate_parts(candidate, state, events)?;
    if !has_parts {
        let text = candidate_text(candidate)?;
        let delta = text.strip_prefix(state.gemini_previous_text.as_str()).unwrap_or(&text);
        if !delta.is_empty() {
            start_gemini_text_block(state, events);
            events.push(InternalStreamEvent::TextDelta(delta.to_owned()));
        }
        state.gemini_previous_text = text;
    }
    if let Some(reason) = candidate.get("finishReason").and_then(Value::as_str) {
        close_gemini_text_blocks(state, events);
        events.push(InternalStreamEvent::Done {
            reason: Some(map_gemini_stop_reason(reason)),
            usage: usage_from_gemini(chunk.get("usageMetadata")),
        });
    }
    Ok(())
}

fn parse_candidate_parts(
    candidate: &serde_json::Map<String, Value>,
    state: &mut StreamConversionState,
    events: &mut Vec<InternalStreamEvent>,
) -> Result<bool, FormatConversionError> {
    let Some(parts) = candidate.get("content").and_then(|content| content.get("parts")).and_then(Value::as_array) else {
        return Ok(false);
    };
    let mut handled = false;
    for part in parts {
        let object = required_object(Some(part), "$.candidates[0].content.parts[]")?;
        if let Some(function_call) = object.get("functionCall").or_else(|| object.get("function_call")) {
            emit_function_call(function_call, state, events)?;
            handled = true;
            continue;
        }
        if object.get("thought").and_then(Value::as_bool) == Some(true) {
            let text = object.get("text").and_then(Value::as_str).unwrap_or_default();
            if !text.is_empty() {
                events.push(InternalStreamEvent::ThinkingDelta {
                    text: text.to_owned(),
                    signature: object.get("thoughtSignature").and_then(Value::as_str).map(str::to_owned),
                });
            }
            handled = true;
            continue;
        }
        if let Some(text) = object.get("text").and_then(Value::as_str) {
            let delta = text.strip_prefix(state.gemini_previous_text.as_str()).unwrap_or(text);
            if !delta.is_empty() {
                start_gemini_text_block(state, events);
                events.push(InternalStreamEvent::TextDelta(delta.to_owned()));
            }
            state.gemini_previous_text = text.to_owned();
            handled = true;
        }
    }
    Ok(handled)
}

fn start_gemini_text_block(state: &mut StreamConversionState, events: &mut Vec<InternalStreamEvent>) {
    if state.gemini_text_started {
        return;
    }
    let index = reserve_gemini_block_index(&mut state.gemini_text_block_index, &mut state.gemini_next_block_index);
    state.gemini_text_started = true;
    events.push(InternalStreamEvent::ContentBlockStart {
        index,
        block: crate::format_conversion::InternalContentBlock::text(String::new()),
    });
}

fn close_gemini_text_blocks(state: &mut StreamConversionState, events: &mut Vec<InternalStreamEvent>) {
    if state.gemini_text_started && !state.gemini_text_stopped {
        state.gemini_text_stopped = true;
        if let Some(index) = state.gemini_text_block_index {
            events.push(InternalStreamEvent::ContentBlockStop { index });
        }
    }
    if state.gemini_thinking_started && !state.gemini_thinking_stopped {
        state.gemini_thinking_stopped = true;
        if let Some(index) = state.gemini_thinking_block_index {
            events.push(InternalStreamEvent::ContentBlockStop { index });
        }
    }
}

fn reserve_gemini_block_index(slot: &mut Option<u32>, next: &mut u32) -> u32 {
    if let Some(index) = *slot {
        return index;
    }
    let index = *next;
    *next = next.saturating_add(1);
    *slot = Some(index);
    index
}

fn emit_function_call(value: &Value, state: &mut StreamConversionState, events: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    let object = required_object(Some(value), "$.candidates[0].content.parts[].functionCall")?;
    let id = object.get("id").and_then(Value::as_str).map(str::to_owned);
    let name = object.get("name").and_then(Value::as_str).unwrap_or_default().to_owned();
    let args = object.get("args").cloned().unwrap_or_else(|| json!({}));
    let block_index = state.gemini_next_block_index;
    state.gemini_next_block_index = state.gemini_next_block_index.saturating_add(1);
    events.push(InternalStreamEvent::ContentBlockStart {
        index: block_index,
        block: crate::format_conversion::InternalContentBlock::ToolUse {
            id: id.clone().unwrap_or_default(),
            name: name.clone(),
            input: json!({}),
            kind: crate::format_conversion::InternalToolKind::Function,
        },
    });
    events.push(InternalStreamEvent::ToolCallDelta {
        index: block_index,
        id,
        name: Some(name),
        arguments_delta: serde_json::to_string(&args).map_err(|error| FormatConversionError::invalid_payload(super::common::FORMAT, error.to_string()))?,
    });
    events.push(InternalStreamEvent::ContentBlockStop { index: block_index });
    Ok(())
}

fn push_stream_event(event: &InternalStreamEvent, state: &mut StreamConversionState, output: &mut Vec<Value>) {
    match event {
        InternalStreamEvent::Start { model: event_model, .. } => {
            state.target_gemini_model = event_model.clone().unwrap_or_else(|| state.target_gemini_model.clone());
        }
        InternalStreamEvent::TextDelta(text) => output.push(content_chunk(text, &state.target_gemini_model, None, None)),
        InternalStreamEvent::ThinkingDelta { text, signature } => {
            output.push(super::common::thought_chunk(text, signature.as_deref(), &state.target_gemini_model));
        }
        InternalStreamEvent::ToolCallDelta {
            index,
            id,
            name,
            arguments_delta,
        } => push_tool_delta(*index, id.as_deref(), name.as_deref(), arguments_delta, state),
        InternalStreamEvent::Usage(usage) => output.push(terminal_chunk(&state.target_gemini_model, None, Some(usage))),
        InternalStreamEvent::ContentBlockStart { index, block } => push_block_start(*index, block, state),
        InternalStreamEvent::ContentBlockStop { index } => push_block_stop(*index, state, output),
        InternalStreamEvent::Error(error) => output.push(serde_json::json!({"error": {"message": error.message, "code": error.code}})),
        InternalStreamEvent::Done { reason, usage } => output.push(terminal_chunk(&state.target_gemini_model, reason.as_ref(), usage.as_ref())),
    }
}

fn push_block_start(index: u32, block: &crate::format_conversion::InternalContentBlock, state: &mut StreamConversionState) {
    let crate::format_conversion::InternalContentBlock::ToolUse { id, name, .. } = block else {
        return;
    };
    state.target_gemini_tools.push(crate::format_conversion::GeminiToolStreamItem {
        block_index: index,
        id: id.clone(),
        name: name.clone(),
        arguments: String::new(),
    });
}

fn push_tool_delta(index: u32, id: Option<&str>, name: Option<&str>, arguments_delta: &str, state: &mut StreamConversionState) {
    let position = tool_position(index, id, state).unwrap_or_else(|| create_tool(index, id, name, state));
    let tool = &mut state.target_gemini_tools[position];
    if let Some(name) = name.filter(|value| !value.is_empty()) {
        tool.name = name.to_owned();
    }
    tool.arguments.push_str(arguments_delta);
}

fn push_block_stop(index: u32, state: &mut StreamConversionState, output: &mut Vec<Value>) {
    let Some(position) = tool_position(index, None, state) else {
        return;
    };
    let tool = state.target_gemini_tools[position].clone();
    output.push(complete_function_call_chunk(&tool, &state.target_gemini_model));
}

fn create_tool(index: u32, id: Option<&str>, name: Option<&str>, state: &mut StreamConversionState) -> usize {
    state.target_gemini_tools.push(crate::format_conversion::GeminiToolStreamItem {
        block_index: index,
        id: id.unwrap_or_default().to_owned(),
        name: name.unwrap_or_default().to_owned(),
        arguments: String::new(),
    });
    state.target_gemini_tools.len().saturating_sub(1)
}

fn tool_position(index: u32, id: Option<&str>, state: &StreamConversionState) -> Option<usize> {
    state
        .target_gemini_tools
        .iter()
        .position(|tool| id.is_some_and(|value| value == tool.id) || tool.block_index == index)
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

fn reject_custom_tool_event(event: &InternalStreamEvent) -> Result<(), FormatConversionError> {
    let InternalStreamEvent::ContentBlockStart { block, .. } = event else {
        return Ok(());
    };
    if matches!(
        block,
        InternalContentBlock::ToolUse {
            kind: InternalToolKind::Custom,
            ..
        }
    ) {
        return Err(FormatConversionError::unsupported_content(
            super::common::FORMAT,
            "Gemini stream cannot represent custom tool calls",
        ));
    }
    Ok(())
}

use serde_json::{Value, json};

use crate::format_conversion::{
    FormatConversionError, InternalContentBlock, InternalStreamEvent, InternalToolKind, OpenAiResponsesSourceToolStreamItem, StreamConversionState,
};

const FORMAT: &str = "openai_responses";

pub(super) fn push_tool_start(item: &Value, kind: InternalToolKind, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    let item_id = item.get("id").and_then(Value::as_str).unwrap_or_default();
    let call_id = item.get("call_id").or_else(|| item.get("id")).and_then(Value::as_str).unwrap_or_default();
    let name = item.get("name").and_then(Value::as_str).unwrap_or_default();
    let block_index = state.openai_responses_next_source_block_index;
    state.openai_responses_next_source_block_index = state.openai_responses_next_source_block_index.saturating_add(1);
    state
        .openai_responses_source_tools
        .push(source_tool_item(item_id, call_id, block_index, name, kind.clone()));
    output.push(InternalStreamEvent::ContentBlockStart {
        index: block_index,
        block: InternalContentBlock::ToolUse {
            id: call_id.to_owned(),
            name: name.to_owned(),
            input: start_input(item, &kind),
            kind,
        },
    });
}

pub(super) fn push_tool_delta(
    chunk: &Value,
    kind: InternalToolKind,
    state: &mut StreamConversionState,
    output: &mut Vec<InternalStreamEvent>,
) -> Result<(), FormatConversionError> {
    let delta = chunk.get("delta").and_then(Value::as_str).unwrap_or_default();
    if delta.is_empty() {
        return Ok(());
    }
    let position = tool_position(chunk, state).unwrap_or_else(|| create_tool_from_chunk(chunk, kind.clone(), state, output));
    let delta = append_tool_delta(position, delta, &kind, state)?;
    if !delta.is_empty() {
        let tool = &state.openai_responses_source_tools[position];
        output.push(InternalStreamEvent::ToolCallDelta {
            index: tool.block_index,
            id: Some(tool.call_id.clone()),
            name: Some(tool.name.clone()),
            arguments_delta: delta,
        });
    }
    Ok(())
}

pub(super) fn sync_tool_arguments(
    value: Option<&Value>,
    block_index: u32,
    state: &mut StreamConversionState,
    output: &mut Vec<InternalStreamEvent>,
) -> Result<(), FormatConversionError> {
    let Some(snapshot) = value.and_then(Value::as_str).filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    let Some(position) = state.openai_responses_source_tools.iter().position(|tool| tool.block_index == block_index) else {
        return Ok(());
    };
    let kind = state.openai_responses_source_tools[position].kind.clone();
    let delta = sync_tool_snapshot(position, snapshot, &kind, state)?;
    if !delta.is_empty() {
        output.push(InternalStreamEvent::ToolCallDelta {
            index: block_index,
            id: Some(state.openai_responses_source_tools[position].call_id.clone()),
            name: Some(state.openai_responses_source_tools[position].name.clone()),
            arguments_delta: delta,
        });
    }
    Ok(())
}

pub(super) fn finish_tool_item(item: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    let kind = item_kind(item)?;
    let position = tool_position_for_ref(item.get("call_id").or_else(|| item.get("id")), state)
        .unwrap_or_else(|| create_tool_from_item(item, kind.clone(), state, output));
    let block_index = state.openai_responses_source_tools[position].block_index;
    let snapshot = match kind {
        InternalToolKind::Function => item.get("arguments"),
        InternalToolKind::Custom => item.get("input"),
    };
    sync_tool_arguments(snapshot, block_index, state, output)?;
    close_custom_tool_input(position, state, output)?;
    stop_tool_block(position, state, output);
    Ok(())
}

pub(super) fn stop_open_tools(state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    for index in 0..state.openai_responses_source_tools.len() {
        let _ = close_custom_tool_input(index, state, output);
        stop_tool_block(index, state, output);
    }
}

pub(super) fn tool_kind_from_item_type(item_type: &str) -> Option<InternalToolKind> {
    match item_type {
        "function_call" => Some(InternalToolKind::Function),
        "custom_tool_call" => Some(InternalToolKind::Custom),
        _ => None,
    }
}

fn item_kind(item: &Value) -> Result<InternalToolKind, FormatConversionError> {
    let item_type = item.get("type").and_then(Value::as_str).unwrap_or_default();
    tool_kind_from_item_type(item_type).ok_or_else(|| FormatConversionError::unsupported_content(FORMAT, format!("unsupported output item type {item_type}")))
}

fn start_input(item: &Value, kind: &InternalToolKind) -> Value {
    match kind {
        InternalToolKind::Function => json!({}),
        InternalToolKind::Custom => json!({ "_raw": item.get("input").and_then(Value::as_str).unwrap_or_default() }),
    }
}

fn append_tool_delta(position: usize, delta: &str, kind: &InternalToolKind, state: &mut StreamConversionState) -> Result<String, FormatConversionError> {
    let was_empty = state.openai_responses_source_tools[position].arguments.is_empty();
    state.openai_responses_source_tools[position].arguments.push_str(delta);
    if *kind == InternalToolKind::Custom {
        return custom_stream_delta(delta, was_empty);
    }
    Ok(delta.to_owned())
}

fn sync_tool_snapshot(position: usize, snapshot: &str, kind: &InternalToolKind, state: &mut StreamConversionState) -> Result<String, FormatConversionError> {
    let current = state.openai_responses_source_tools[position].arguments.clone();
    if snapshot == current || snapshot.is_empty() {
        return Ok(String::new());
    }
    let Some(delta) = snapshot.strip_prefix(&current) else {
        return Err(FormatConversionError::invalid_payload(
            FORMAT,
            "custom tool stream input snapshot does not extend prior deltas",
        ));
    };
    state.openai_responses_source_tools[position].arguments = snapshot.to_owned();
    if *kind == InternalToolKind::Custom {
        return custom_stream_delta(delta, current.is_empty());
    }
    Ok(delta.to_owned())
}

fn close_custom_tool_input(position: usize, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    let Some(tool) = state.openai_responses_source_tools.get(position) else {
        return Ok(());
    };
    if tool.kind != InternalToolKind::Custom || tool.stopped {
        return Ok(());
    }
    let arguments_delta = if tool.arguments.is_empty() {
        "{\"_raw\":\"\"}".to_owned()
    } else {
        "\"}".to_owned()
    };
    output.push(InternalStreamEvent::ToolCallDelta {
        index: tool.block_index,
        id: Some(tool.call_id.clone()),
        name: Some(tool.name.clone()),
        arguments_delta,
    });
    Ok(())
}

fn custom_stream_delta(delta: &str, include_prefix: bool) -> Result<String, FormatConversionError> {
    let fragment = escaped_json_string_fragment(delta)?;
    if include_prefix {
        return Ok(format!("{{\"_raw\":\"{fragment}"));
    }
    Ok(fragment)
}

fn escaped_json_string_fragment(value: &str) -> Result<String, FormatConversionError> {
    let quoted = serde_json::to_string(value).map_err(|error| FormatConversionError::invalid_payload(FORMAT, error.to_string()))?;
    Ok(quoted.trim_matches('"').to_owned())
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

fn create_tool_from_chunk(chunk: &Value, kind: InternalToolKind, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) -> usize {
    let call_id = chunk
        .get("call_id")
        .or_else(|| chunk.get("item_id"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    create_tool(call_id, call_id, "", kind, state, output)
}

fn create_tool_from_item(item: &Value, kind: InternalToolKind, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) -> usize {
    let item_id = item.get("id").and_then(Value::as_str).unwrap_or_default();
    let call_id = item.get("call_id").or_else(|| item.get("id")).and_then(Value::as_str).unwrap_or_default();
    let name = item.get("name").and_then(Value::as_str).unwrap_or_default();
    create_tool(item_id, call_id, name, kind, state, output)
}

fn create_tool(
    item_id: &str,
    call_id: &str,
    name: &str,
    kind: InternalToolKind,
    state: &mut StreamConversionState,
    output: &mut Vec<InternalStreamEvent>,
) -> usize {
    let block_index = state.openai_responses_next_source_block_index;
    state.openai_responses_next_source_block_index = state.openai_responses_next_source_block_index.saturating_add(1);
    state
        .openai_responses_source_tools
        .push(source_tool_item(item_id, call_id, block_index, name, kind.clone()));
    output.push(InternalStreamEvent::ContentBlockStart {
        index: block_index,
        block: InternalContentBlock::ToolUse {
            id: call_id.to_owned(),
            name: name.to_owned(),
            input: start_input(&json!({}), &kind),
            kind,
        },
    });
    state.openai_responses_source_tools.len().saturating_sub(1)
}

fn source_tool_item(item_id: &str, call_id: &str, block_index: u32, name: &str, kind: InternalToolKind) -> OpenAiResponsesSourceToolStreamItem {
    OpenAiResponsesSourceToolStreamItem {
        item_id: item_id.to_owned(),
        call_id: call_id.to_owned(),
        block_index,
        name: name.to_owned(),
        arguments: String::new(),
        kind,
        stopped: false,
    }
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

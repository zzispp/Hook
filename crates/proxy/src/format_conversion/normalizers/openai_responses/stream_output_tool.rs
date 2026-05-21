use serde_json::{Value, json};

use crate::format_conversion::{InternalContentBlock, OpenAiResponsesToolStreamItem, StreamConversionState};

use super::stream_output_common::{allocate_output_index, function_call_item, next_sequence};

pub(super) fn push_block_start(index: u32, block: &InternalContentBlock, state: &mut StreamConversionState, output: &mut Vec<Value>) {
    if let InternalContentBlock::ToolUse { id, name, .. } = block {
        let output_index = allocate_output_index(state);
        let call_id = if id.is_empty() { format!("call_{output_index}") } else { id.clone() };
        let item_id = normalize_call_item_id(&call_id);
        state.target_openai_responses_tools.push(OpenAiResponsesToolStreamItem {
            block_index: index,
            output_index,
            call_id: call_id.clone(),
            item_id: item_id.clone(),
            name: name.clone(),
            arguments: String::new(),
        });
        output.push(json!({
            "type": "response.output_item.added",
            "sequence_number": next_sequence(state),
            "output_index": output_index,
            "item": function_call_item(&call_id, &item_id, name, "", "in_progress"),
        }));
    }
}

pub(super) fn push_tool_delta(
    index: u32,
    id: Option<&str>,
    name: Option<&str>,
    arguments_delta: &str,
    state: &mut StreamConversionState,
    output: &mut Vec<Value>,
) {
    let position = tool_position(index, id, state).unwrap_or_else(|| create_tool(index, id, name, state));
    {
        let tool = &mut state.target_openai_responses_tools[position];
        if let Some(name) = name.filter(|value| !value.is_empty()) {
            tool.name = name.to_owned();
        }
        tool.arguments.push_str(arguments_delta);
    }
    output.push(json!({
        "type": "response.function_call_arguments.delta",
        "sequence_number": next_sequence(state),
        "item_id": state.target_openai_responses_tools[position].item_id,
        "output_index": state.target_openai_responses_tools[position].output_index,
        "delta": arguments_delta,
    }));
}

pub(super) fn push_block_stop(index: u32, state: &mut StreamConversionState, output: &mut Vec<Value>) {
    let Some(position) = tool_position(index, None, state) else {
        return;
    };
    let tool = state.target_openai_responses_tools[position].clone();
    output.push(json!({
        "type": "response.function_call_arguments.done",
        "sequence_number": next_sequence(state),
        "item_id": tool.item_id,
        "output_index": tool.output_index,
        "arguments": tool.arguments,
    }));
    output.push(json!({
        "type": "response.output_item.done",
        "sequence_number": next_sequence(state),
        "output_index": tool.output_index,
        "item": function_call_item(&tool.call_id, &tool.item_id, &tool.name, &tool.arguments, "completed"),
    }));
}

pub(super) fn final_tool_items(state: &StreamConversionState) -> Vec<(u32, Value)> {
    state
        .target_openai_responses_tools
        .iter()
        .map(|tool| {
            (
                tool.output_index,
                function_call_item(&tool.call_id, &tool.item_id, &tool.name, &tool.arguments, "completed"),
            )
        })
        .collect()
}

fn create_tool(index: u32, id: Option<&str>, name: Option<&str>, state: &mut StreamConversionState) -> usize {
    let output_index = allocate_output_index(state);
    let call_id = id
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("call_{output_index}"));
    let item_id = normalize_call_item_id(&call_id);
    state.target_openai_responses_tools.push(OpenAiResponsesToolStreamItem {
        block_index: index,
        output_index,
        call_id,
        item_id,
        name: name.unwrap_or_default().to_owned(),
        arguments: String::new(),
    });
    state.target_openai_responses_tools.len().saturating_sub(1)
}

fn tool_position(index: u32, id: Option<&str>, state: &StreamConversionState) -> Option<usize> {
    state
        .target_openai_responses_tools
        .iter()
        .position(|tool| id.is_some_and(|value| value == tool.call_id) || tool.block_index == index)
}

fn normalize_call_item_id(value: &str) -> String {
    if value.starts_with("fc") {
        value.to_owned()
    } else if let Some(suffix) = value.strip_prefix("call_") {
        format!("fc_{suffix}")
    } else if let Some(suffix) = value.strip_prefix("call") {
        format!("fc_{suffix}")
    } else {
        value.to_owned()
    }
}

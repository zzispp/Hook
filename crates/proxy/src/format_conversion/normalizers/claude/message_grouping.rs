use serde_json::Value;

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalMessage, InternalRole};

use super::request_codec::{content_from_internal, message_role};

pub(super) fn messages_from_internal(messages: &[InternalMessage]) -> Result<Vec<Value>, FormatConversionError> {
    let mut output = Vec::new();
    for message in messages
        .iter()
        .filter(|message| !matches!(message.role, InternalRole::System | InternalRole::Developer))
    {
        push_grouped_message(&mut output, message)?;
    }
    remove_orphaned_tool_uses(&mut output);
    merge_adjacent_messages(&mut output);
    prepend_empty_user_when_first_is_assistant(&mut output);
    validate_tool_result_pairing(&output)?;
    Ok(output)
}

fn push_grouped_message(output: &mut Vec<Value>, message: &InternalMessage) -> Result<(), FormatConversionError> {
    let role = message_role(&message.role);
    let content = content_from_internal(&ordered_content_blocks(&message.content, role))?;
    if merge_into_previous(output, role, content.clone()) {
        return Ok(());
    }
    output.push(serde_json::json!({ "role": role, "content": content }));
    Ok(())
}

fn ordered_content_blocks(blocks: &[InternalContentBlock], role: &str) -> Vec<InternalContentBlock> {
    if role == "user" {
        return ordered_user_blocks(blocks);
    }
    if role == "assistant" {
        return ordered_assistant_blocks(blocks);
    }
    blocks.to_vec()
}

fn ordered_assistant_blocks(blocks: &[InternalContentBlock]) -> Vec<InternalContentBlock> {
    let mut ordered = Vec::with_capacity(blocks.len());
    push_matching_blocks(&mut ordered, blocks, is_thinking_block);
    push_matching_blocks(&mut ordered, blocks, is_text_block);
    push_remaining_blocks(&mut ordered, blocks);
    ordered
}

fn ordered_user_blocks(blocks: &[InternalContentBlock]) -> Vec<InternalContentBlock> {
    if !blocks.iter().any(is_tool_result_block) {
        return blocks.to_vec();
    }
    let mut ordered = Vec::with_capacity(blocks.len());
    push_matching_blocks(&mut ordered, blocks, is_tool_result_block);
    push_remaining_user_blocks(&mut ordered, blocks);
    ordered
}

fn merge_into_previous(output: &mut [Value], role: &str, content: Value) -> bool {
    let Some(previous) = output.last_mut() else {
        return false;
    };
    if previous.get("role").and_then(Value::as_str) != Some(role) {
        return false;
    }
    let (Some(previous_content), Some(new_content)) = (previous.get_mut("content").and_then(Value::as_array_mut), content.as_array()) else {
        return false;
    };
    previous_content.extend(new_content.iter().cloned());
    if role == "assistant" {
        sort_assistant_content(previous_content);
    }
    true
}

fn push_matching_blocks(ordered: &mut Vec<InternalContentBlock>, blocks: &[InternalContentBlock], predicate: fn(&InternalContentBlock) -> bool) {
    ordered.extend(blocks.iter().filter(|block| predicate(block)).cloned());
}

fn push_remaining_blocks(ordered: &mut Vec<InternalContentBlock>, blocks: &[InternalContentBlock]) {
    ordered.extend(blocks.iter().filter(|block| !is_thinking_block(block) && !is_text_block(block)).cloned());
}

fn push_remaining_user_blocks(ordered: &mut Vec<InternalContentBlock>, blocks: &[InternalContentBlock]) {
    ordered.extend(blocks.iter().filter(|block| !is_tool_result_block(block)).cloned());
}

fn is_thinking_block(block: &InternalContentBlock) -> bool {
    matches!(block, InternalContentBlock::Thinking { .. })
}

fn is_text_block(block: &InternalContentBlock) -> bool {
    matches!(block, InternalContentBlock::Text { .. })
}

fn is_tool_result_block(block: &InternalContentBlock) -> bool {
    matches!(block, InternalContentBlock::ToolResult { .. })
}

fn sort_assistant_content(content: &mut [Value]) {
    content.sort_by_key(assistant_block_order);
}

fn assistant_block_order(value: &Value) -> u8 {
    match value.get("type").and_then(Value::as_str) {
        Some("thinking") => 0,
        Some("text") => 1,
        _ => 2,
    }
}

fn validate_tool_result_pairing(messages: &[Value]) -> Result<(), FormatConversionError> {
    for (index, message) in messages.iter().enumerate() {
        let ids = tool_use_ids(message);
        if ids.is_empty() {
            continue;
        }
        let Some(next) = messages
            .get(index + 1)
            .filter(|value| value.get("role").and_then(Value::as_str) == Some("user"))
        else {
            return Err(missing_tool_result_error(ids));
        };
        let present = leading_tool_result_ids(next);
        let missing = ids.into_iter().filter(|id| !present.iter().any(|present| present == id)).collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(missing_tool_result_error(missing));
        }
    }
    Ok(())
}

fn remove_orphaned_tool_uses(messages: &mut Vec<Value>) {
    let mut removals = Vec::new();
    for (index, message) in messages.iter().enumerate() {
        let ids = tool_use_ids(message);
        if ids.is_empty() {
            continue;
        }
        let present = messages.get(index + 1).map(leading_tool_result_ids).unwrap_or_default();
        let orphaned = ids.into_iter().filter(|id| !present.iter().any(|present| present == id)).collect::<Vec<_>>();
        if !orphaned.is_empty() {
            removals.push((index, orphaned));
        }
    }
    for (index, ids) in removals {
        remove_tool_use_blocks(&mut messages[index], &ids);
    }
    messages.retain(message_has_content);
}

fn remove_tool_use_blocks(message: &mut Value, ids: &[String]) {
    let Some(content) = message.get_mut("content").and_then(Value::as_array_mut) else {
        return;
    };
    content.retain(|block| {
        if block.get("type").and_then(Value::as_str) != Some("tool_use") {
            return true;
        }
        let Some(id) = block.get("id").and_then(Value::as_str) else {
            return true;
        };
        !ids.iter().any(|item| item == id)
    });
}

fn message_has_content(message: &Value) -> bool {
    match message.get("content") {
        Some(Value::String(text)) => !text.is_empty(),
        Some(Value::Array(items)) => !items.is_empty(),
        _ => false,
    }
}

fn merge_adjacent_messages(messages: &mut Vec<Value>) {
    let mut merged = Vec::with_capacity(messages.len());
    for message in std::mem::take(messages) {
        let role = message.get("role").and_then(Value::as_str).unwrap_or_default();
        let content = message.get("content").cloned().unwrap_or(Value::Array(Vec::new()));
        if merge_into_previous(&mut merged, role, content.clone()) {
            continue;
        }
        merged.push(serde_json::json!({ "role": role, "content": content }));
    }
    *messages = merged;
}

fn prepend_empty_user_when_first_is_assistant(messages: &mut Vec<Value>) {
    let Some(first) = messages.first() else {
        return;
    };
    if first.get("role").and_then(Value::as_str) == Some("user") {
        return;
    }
    messages.insert(0, serde_json::json!({ "role": "user", "content": [] }));
}

fn tool_use_ids(message: &Value) -> Vec<String> {
    if message.get("role").and_then(Value::as_str) != Some("assistant") {
        return Vec::new();
    }
    message
        .get("content")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|block| block.get("type").and_then(Value::as_str) == Some("tool_use"))
        .filter_map(|block| block.get("id").and_then(Value::as_str).map(str::to_owned))
        .collect()
}

fn leading_tool_result_ids(message: &Value) -> Vec<String> {
    message
        .get("content")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take_while(|block| block.get("type").and_then(Value::as_str) == Some("tool_result"))
        .filter_map(|block| block.get("tool_use_id").and_then(Value::as_str).map(str::to_owned))
        .collect()
}

fn missing_tool_result_error(ids: Vec<String>) -> FormatConversionError {
    FormatConversionError::unsupported_content(
        "claude",
        format!("Claude Messages requires tool_result blocks immediately after tool_use ids: {}", ids.join(", ")),
    )
}

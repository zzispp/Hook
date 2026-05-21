use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalMessage, InternalRole};

use super::common::{FORMAT, text_from_blocks};
use super::request_codec::openai_role;

pub(super) fn request_messages_from_internal(messages: &[InternalMessage]) -> Result<Vec<Value>, FormatConversionError> {
    let mut output = Vec::new();
    for message in messages {
        output.extend(messages_from_internal_message(message)?);
    }
    Ok(output)
}

fn messages_from_internal_message(message: &InternalMessage) -> Result<Vec<Value>, FormatConversionError> {
    if matches!(message.role, InternalRole::User | InternalRole::Tool) && has_tool_result(&message.content) {
        return user_and_tool_messages(message);
    }
    let mut output = json!({
        "role": openai_role(&message.role),
        "content": content_from_internal(&message.content)?,
    });
    if matches!(message.role, InternalRole::Assistant) {
        let reasoning = thinking_text(&message.content);
        if !reasoning.is_empty() {
            output["reasoning_content"] = Value::String(reasoning);
        }
    }
    let tool_calls = tool_calls_from_blocks(&message.content)?;
    if !tool_calls.is_empty() {
        output["tool_calls"] = Value::Array(tool_calls);
        if output["content"].as_array().is_some_and(Vec::is_empty) {
            output["content"] = Value::Null;
        }
    }
    Ok(vec![output])
}

fn user_and_tool_messages(message: &InternalMessage) -> Result<Vec<Value>, FormatConversionError> {
    let mut output = Vec::new();
    let mut pending = Vec::new();
    for block in &message.content {
        if let InternalContentBlock::ToolResult { tool_use_id, content, .. } = block {
            push_pending_user_message(&mut output, &mut pending)?;
            output.push(tool_result_message(tool_use_id, content)?);
            continue;
        }
        if !matches!(block, InternalContentBlock::ToolUse { .. }) {
            pending.push(block.clone());
        }
    }
    push_pending_user_message(&mut output, &mut pending)?;
    if output.is_empty() {
        output.push(json!({ "role": openai_role(&message.role), "content": "" }));
    }
    Ok(output)
}

fn push_pending_user_message(output: &mut Vec<Value>, pending: &mut Vec<InternalContentBlock>) -> Result<(), FormatConversionError> {
    if pending.is_empty() {
        return Ok(());
    }
    let content = content_from_internal(pending)?;
    output.push(json!({ "role": "user", "content": content }));
    pending.clear();
    Ok(())
}

fn content_from_internal(blocks: &[InternalContentBlock]) -> Result<Value, FormatConversionError> {
    if blocks.iter().all(|block| matches!(block, InternalContentBlock::Text { .. })) {
        return text_from_blocks(blocks).map(Value::String);
    }
    let mut values = Vec::new();
    for block in blocks {
        if !matches!(block, InternalContentBlock::Thinking { .. } | InternalContentBlock::ToolUse { .. }) {
            values.push(block_from_internal(block)?);
        }
    }
    Ok(Value::Array(values))
}

fn block_from_internal(block: &InternalContentBlock) -> Result<Value, FormatConversionError> {
    match block {
        InternalContentBlock::Text { text, .. } => Ok(json!({ "type": "text", "text": text })),
        InternalContentBlock::Image { url: Some(url), .. } => Ok(json!({ "type": "image_url", "image_url": { "url": url } })),
        InternalContentBlock::Image {
            data: Some(data), media_type, ..
        } => Ok(json!({
            "type": "image_url",
            "image_url": { "url": crate::format_conversion::data_url::format_base64_data_url(media_type.as_deref(), data, FORMAT)? },
        })),
        InternalContentBlock::Audio { data, format, .. } => Ok(json!({ "type": "input_audio", "input_audio": { "data": data, "format": format } })),
        InternalContentBlock::File { file_id, data, filename, .. } => {
            Ok(json!({ "type": "file", "file": { "file_id": file_id, "file_data": data, "filename": filename } }))
        }
        InternalContentBlock::ToolResult { content, .. } => Ok(Value::String(text_from_blocks(content)?)),
        InternalContentBlock::Thinking { .. } => Err(FormatConversionError::unsupported_content(
            FORMAT,
            "thinking must be encoded as reasoning_content",
        )),
        InternalContentBlock::Image { .. } => Err(FormatConversionError::unsupported_content(
            FORMAT,
            "content block cannot be represented in OpenAI Chat",
        )),
        InternalContentBlock::ToolUse { .. } => Err(FormatConversionError::unsupported_content(FORMAT, "tool_use must be encoded as tool_calls")),
    }
}

fn tool_result_message(tool_use_id: &str, content: &[InternalContentBlock]) -> Result<Value, FormatConversionError> {
    Ok(json!({
        "role": "tool",
        "tool_call_id": tool_use_id,
        "content": text_from_blocks(content)?,
    }))
}

fn has_tool_result(blocks: &[InternalContentBlock]) -> bool {
    blocks.iter().any(|block| matches!(block, InternalContentBlock::ToolResult { .. }))
}

fn tool_calls_from_blocks(blocks: &[InternalContentBlock]) -> Result<Vec<Value>, FormatConversionError> {
    let mut calls = Vec::new();
    for block in blocks {
        if let InternalContentBlock::ToolUse { id, name, input } = block {
            calls.push(json!({
                "id": id,
                "type": "function",
                "function": {
                    "name": name,
                    "arguments": serde_json::to_string(input).map_err(|error| FormatConversionError::invalid_payload(FORMAT, error.to_string()))?,
                },
            }));
        }
    }
    Ok(calls)
}

fn thinking_text(blocks: &[InternalContentBlock]) -> String {
    blocks
        .iter()
        .filter_map(|block| match block {
            InternalContentBlock::Thinking { text, .. } if !text.is_empty() => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
}

use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalToolKind};

use super::request_content::tool_output_from_internal;
use super::request_items::custom_tool_input_text;

const FORMAT: &str = "openai_responses";

pub(super) fn output_items(blocks: &[InternalContentBlock]) -> Result<Vec<Value>, FormatConversionError> {
    let mut items = Vec::new();
    let mut message_content = Vec::new();
    for block in blocks {
        append_block(block, &mut items, &mut message_content)?;
    }
    push_message_item(&mut items, &mut message_content);
    Ok(items)
}

fn append_block(block: &InternalContentBlock, items: &mut Vec<Value>, message_content: &mut Vec<Value>) -> Result<(), FormatConversionError> {
    match block {
        InternalContentBlock::Text { text, .. } => message_content.push(output_text_block(text)),
        InternalContentBlock::Thinking { text, signature } => items.push(reasoning_item(text, signature)),
        InternalContentBlock::ToolUse { id, name, input, kind } => {
            push_message_item(items, message_content);
            items.push(tool_call_item(id, name, input, kind)?);
        }
        InternalContentBlock::ToolResult {
            tool_use_id,
            tool_name,
            tool_kind,
            content,
            ..
        } => {
            push_message_item(items, message_content);
            items.push(tool_result_item(tool_use_id, tool_name, tool_kind, content)?);
        }
        _ => return Err(unsupported_response_block()),
    }
    Ok(())
}

fn push_message_item(items: &mut Vec<Value>, message_content: &mut Vec<Value>) {
    if message_content.is_empty() {
        return;
    }
    items.push(json!({
        "type": "message",
        "role": "assistant",
        "content": std::mem::take(message_content),
    }));
}

fn output_text_block(text: &str) -> Value {
    json!({ "type": "output_text", "text": text })
}

fn reasoning_item(text: &str, signature: &Option<String>) -> Value {
    let mut item = json!({
        "type": "reasoning",
        "summary": reasoning_summary(text),
    });
    if let Some(signature) = signature {
        item["encrypted_content"] = Value::String(signature.clone());
    }
    item
}

fn tool_call_item(id: &str, name: &str, input: &Value, kind: &InternalToolKind) -> Result<Value, FormatConversionError> {
    if *kind == InternalToolKind::Custom {
        return Ok(json!({
            "type": "custom_tool_call",
            "call_id": id,
            "name": name,
            "input": custom_tool_input_text(input)?,
        }));
    }
    Ok(json!({
        "type": "function_call",
        "call_id": id,
        "name": name,
        "arguments": serde_json::to_string(input).map_err(|error| FormatConversionError::invalid_payload(FORMAT, error.to_string()))?,
    }))
}

fn tool_result_item(
    tool_use_id: &str,
    tool_name: &Option<String>,
    tool_kind: &InternalToolKind,
    content: &[InternalContentBlock],
) -> Result<Value, FormatConversionError> {
    if *tool_kind == InternalToolKind::Custom {
        return custom_tool_result_item(tool_use_id, tool_name, content);
    }
    Ok(json!({
        "type": "function_call_output",
        "call_id": tool_use_id,
        "output": tool_output_from_internal(content)?,
    }))
}

fn custom_tool_result_item(tool_use_id: &str, tool_name: &Option<String>, content: &[InternalContentBlock]) -> Result<Value, FormatConversionError> {
    let mut item = json!({
        "type": "custom_tool_call_output",
        "call_id": tool_use_id,
        "output": tool_output_from_internal(content)?,
    });
    if let Some(tool_name) = tool_name {
        item["name"] = Value::String(tool_name.clone());
    }
    Ok(item)
}

fn reasoning_summary(value: &str) -> Vec<Value> {
    if value.is_empty() {
        Vec::new()
    } else {
        vec![json!({ "type": "summary_text", "text": value })]
    }
}

fn unsupported_response_block() -> FormatConversionError {
    FormatConversionError::unsupported_content(FORMAT, "response content block cannot be represented in OpenAI Responses")
}

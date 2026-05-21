use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalMessage, InternalRole, InternalToolKind};

use super::request_content::{content_blocks, content_from_internal, tool_output_from_internal};
use super::request_fields::FORMAT;
use super::request_items::{arguments_json, custom_tool_input, custom_tool_input_text};

pub(super) fn input_messages(value: Option<&Value>) -> Result<Vec<InternalMessage>, FormatConversionError> {
    match value {
        Some(Value::String(text)) => Ok(vec![InternalMessage::text(InternalRole::User, text)]),
        Some(Value::Array(items)) => items.iter().enumerate().map(|(index, value)| input_item(value, index)).collect(),
        Some(_) | None => Err(FormatConversionError::invalid_payload(FORMAT, "$.input")),
    }
}

pub(super) fn messages_from_internal(messages: &[InternalMessage]) -> Result<Vec<Value>, FormatConversionError> {
    let mut output = Vec::new();
    for message in messages
        .iter()
        .filter(|message| !matches!(message.role, InternalRole::System | InternalRole::Developer))
    {
        output.extend(message_from_internal(message)?);
    }
    Ok(output)
}

fn input_item(value: &Value, index: usize) -> Result<InternalMessage, FormatConversionError> {
    let object = value
        .as_object()
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("$.input[{index}]")))?;
    match object.get("type").and_then(Value::as_str).unwrap_or("message") {
        "message" => message_item(object, index),
        "function_call" => function_call_item(object),
        "custom_tool_call" => custom_tool_call_item(object),
        "function_call_output" => function_call_output_item(object, index),
        "custom_tool_call_output" => custom_tool_call_output_item(object, index),
        "reasoning" => reasoning_item(object),
        other => Err(FormatConversionError::unsupported_content(
            FORMAT,
            format!("$.input[{index}]: unsupported item type {other}"),
        )),
    }
}

fn message_item(object: &Map<String, Value>, index: usize) -> Result<InternalMessage, FormatConversionError> {
    let role = object.get("role").and_then(Value::as_str).unwrap_or("user");
    Ok(InternalMessage {
        role: role_from_str(role),
        content: content_blocks(object.get("content"), &format!("$.input[{index}].content"))?,
    })
}

fn function_call_item(object: &Map<String, Value>) -> Result<InternalMessage, FormatConversionError> {
    Ok(InternalMessage {
        role: InternalRole::Assistant,
        content: vec![InternalContentBlock::ToolUse {
            id: object.get("call_id").and_then(Value::as_str).unwrap_or_default().to_owned(),
            name: object.get("name").and_then(Value::as_str).unwrap_or_default().to_owned(),
            input: arguments_json(object.get("arguments"))?,
            kind: InternalToolKind::Function,
        }],
    })
}

fn custom_tool_call_item(object: &Map<String, Value>) -> Result<InternalMessage, FormatConversionError> {
    Ok(InternalMessage {
        role: InternalRole::Assistant,
        content: vec![InternalContentBlock::ToolUse {
            id: object.get("call_id").and_then(Value::as_str).unwrap_or_default().to_owned(),
            name: object.get("name").and_then(Value::as_str).unwrap_or_default().to_owned(),
            input: custom_tool_input(object.get("input")),
            kind: InternalToolKind::Custom,
        }],
    })
}

fn function_call_output_item(object: &Map<String, Value>, index: usize) -> Result<InternalMessage, FormatConversionError> {
    Ok(InternalMessage {
        role: InternalRole::User,
        content: vec![InternalContentBlock::ToolResult {
            tool_use_id: object.get("call_id").and_then(Value::as_str).unwrap_or_default().to_owned(),
            tool_name: None,
            tool_kind: InternalToolKind::Function,
            content: content_blocks(object.get("output"), &format!("$.input[{index}].output"))?,
            is_error: false,
        }],
    })
}

fn custom_tool_call_output_item(object: &Map<String, Value>, index: usize) -> Result<InternalMessage, FormatConversionError> {
    Ok(InternalMessage {
        role: InternalRole::User,
        content: vec![InternalContentBlock::ToolResult {
            tool_use_id: object.get("call_id").and_then(Value::as_str).unwrap_or_default().to_owned(),
            tool_name: object.get("name").and_then(Value::as_str).map(str::to_owned),
            tool_kind: InternalToolKind::Custom,
            content: content_blocks(object.get("output"), &format!("$.input[{index}].output"))?,
            is_error: false,
        }],
    })
}

fn reasoning_item(object: &Map<String, Value>) -> Result<InternalMessage, FormatConversionError> {
    let text = reasoning_summary_text(object.get("summary"));
    Ok(InternalMessage {
        role: InternalRole::Assistant,
        content: vec![InternalContentBlock::Thinking {
            text,
            signature: object.get("encrypted_content").and_then(Value::as_str).map(str::to_owned),
        }],
    })
}

fn message_from_internal(message: &InternalMessage) -> Result<Vec<Value>, FormatConversionError> {
    let mut output = Vec::new();
    for block in &message.content {
        match block {
            InternalContentBlock::ToolUse { id, name, input, kind } => output.push(tool_call_from_block(id, name, input, kind)?),
            InternalContentBlock::ToolResult {
                tool_use_id,
                tool_name,
                tool_kind,
                content,
                ..
            } => output.push(tool_output_from_block(tool_use_id, tool_name, tool_kind, content)?),
            InternalContentBlock::Thinking { text, signature } => output.push(reasoning_from_block(text, signature)),
            _ => {}
        }
    }
    if let Some(message_item) = message_content_item(message)? {
        output.push(message_item);
    }
    Ok(output)
}

fn message_content_item(message: &InternalMessage) -> Result<Option<Value>, FormatConversionError> {
    let content = message
        .content
        .iter()
        .filter(|block| {
            !matches!(
                block,
                InternalContentBlock::ToolUse { .. } | InternalContentBlock::ToolResult { .. } | InternalContentBlock::Thinking { .. }
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    if content.is_empty() {
        return Ok(None);
    }
    Ok(Some(json!({
        "type": "message",
        "role": role_as_str(&message.role),
        "content": content_from_internal(&content)?,
    })))
}

fn tool_call_from_block(id: &str, name: &str, input: &Value, kind: &InternalToolKind) -> Result<Value, FormatConversionError> {
    if *kind == InternalToolKind::Custom {
        return custom_tool_call_from_block(id, name, input);
    }
    function_call_from_block(id, name, input)
}

fn function_call_from_block(id: &str, name: &str, input: &Value) -> Result<Value, FormatConversionError> {
    Ok(json!({
        "type": "function_call",
        "call_id": id,
        "name": name,
        "arguments": serde_json::to_string(input).map_err(|error| FormatConversionError::invalid_payload(FORMAT, error.to_string()))?,
    }))
}

fn custom_tool_call_from_block(id: &str, name: &str, input: &Value) -> Result<Value, FormatConversionError> {
    Ok(json!({
        "type": "custom_tool_call",
        "call_id": id,
        "name": name,
        "input": custom_tool_input_text(input)?,
    }))
}

fn tool_output_from_block(
    tool_use_id: &str,
    tool_name: &Option<String>,
    tool_kind: &InternalToolKind,
    content: &[InternalContentBlock],
) -> Result<Value, FormatConversionError> {
    if *tool_kind == InternalToolKind::Custom {
        return custom_tool_call_output_from_block(tool_use_id, tool_name, content);
    }
    function_call_output_from_block(tool_use_id, content)
}

fn function_call_output_from_block(tool_use_id: &str, content: &[InternalContentBlock]) -> Result<Value, FormatConversionError> {
    Ok(json!({
        "type": "function_call_output",
        "call_id": tool_use_id,
        "output": tool_output_from_internal(content)?,
    }))
}

fn custom_tool_call_output_from_block(tool_use_id: &str, tool_name: &Option<String>, content: &[InternalContentBlock]) -> Result<Value, FormatConversionError> {
    let mut output = json!({
        "type": "custom_tool_call_output",
        "call_id": tool_use_id,
        "output": tool_output_from_internal(content)?,
    });
    if let Some(tool_name) = tool_name {
        output["name"] = Value::String(tool_name.clone());
    }
    Ok(output)
}

fn reasoning_from_block(text: &str, signature: &Option<String>) -> Value {
    let mut item = json!({
        "type": "reasoning",
        "summary": reasoning_summary(text),
    });
    if let Some(signature) = signature {
        item["encrypted_content"] = Value::String(signature.clone());
    }
    item
}

fn reasoning_summary(value: &str) -> Vec<Value> {
    if value.is_empty() {
        Vec::new()
    } else {
        vec![json!({ "type": "summary_text", "text": value })]
    }
}

fn reasoning_summary_text(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(items)) => items.iter().filter_map(reasoning_summary_part_text).collect::<Vec<_>>().join("\n"),
        _ => String::new(),
    }
}

fn reasoning_summary_part_text(value: &Value) -> Option<&str> {
    match value {
        Value::String(text) if !text.is_empty() => Some(text.as_str()),
        Value::Object(object) => object.get("text").and_then(Value::as_str).filter(|text| !text.is_empty()),
        _ => None,
    }
}

fn role_from_str(value: &str) -> InternalRole {
    match value {
        "assistant" => InternalRole::Assistant,
        "system" => InternalRole::System,
        "developer" => InternalRole::Developer,
        "tool" => InternalRole::Tool,
        _ => InternalRole::User,
    }
}

fn role_as_str(role: &InternalRole) -> &'static str {
    match role {
        InternalRole::System => "system",
        InternalRole::Developer => "developer",
        InternalRole::User => "user",
        InternalRole::Assistant => "assistant",
        InternalRole::Tool => "tool",
        InternalRole::Unknown(_) => "user",
    }
}

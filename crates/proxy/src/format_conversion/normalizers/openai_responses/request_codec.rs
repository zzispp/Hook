use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalMessage, InternalRole};

use super::request_fields::FORMAT;

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
        "function_call_output" => function_call_output_item(object),
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
        }],
    })
}

fn function_call_output_item(object: &Map<String, Value>) -> Result<InternalMessage, FormatConversionError> {
    Ok(InternalMessage {
        role: InternalRole::User,
        content: vec![InternalContentBlock::ToolResult {
            tool_use_id: object.get("call_id").and_then(Value::as_str).unwrap_or_default().to_owned(),
            tool_name: None,
            content: vec![InternalContentBlock::text(
                object.get("output").and_then(Value::as_str).unwrap_or_default().to_owned(),
            )],
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

fn content_blocks(value: Option<&Value>, path: &str) -> Result<Vec<InternalContentBlock>, FormatConversionError> {
    match value {
        Some(Value::String(text)) => Ok(vec![InternalContentBlock::text(text.clone())]),
        Some(Value::Array(items)) => items
            .iter()
            .enumerate()
            .map(|(index, item)| content_block(item, &format!("{path}[{index}]")))
            .collect(),
        Some(_) | None => Err(FormatConversionError::invalid_payload(FORMAT, path)),
    }
}

fn content_block(value: &Value, path: &str) -> Result<InternalContentBlock, FormatConversionError> {
    let object = value.as_object().ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, path))?;
    match object.get("type").and_then(Value::as_str).unwrap_or_default() {
        "input_text" | "output_text" | "text" => Ok(InternalContentBlock::text(required_text(object, path, "text")?.to_owned())),
        "input_image" => Ok(InternalContentBlock::Image {
            url: object.get("image_url").and_then(Value::as_str).map(str::to_owned),
            data: None,
            media_type: None,
        }),
        "input_file" => Ok(InternalContentBlock::File {
            file_id: object.get("file_id").and_then(Value::as_str).map(str::to_owned),
            file_url: object.get("file_url").and_then(Value::as_str).map(str::to_owned),
            data: object.get("file_data").and_then(Value::as_str).map(str::to_owned),
            media_type: None,
            filename: object.get("filename").and_then(Value::as_str).map(str::to_owned),
        }),
        other => Err(FormatConversionError::unsupported_content(
            FORMAT,
            format!("{path}: unsupported block type {other}"),
        )),
    }
}

fn message_from_internal(message: &InternalMessage) -> Result<Vec<Value>, FormatConversionError> {
    let mut output = Vec::new();
    for block in &message.content {
        match block {
            InternalContentBlock::ToolUse { id, name, input } => output.push(function_call_from_block(id, name, input)?),
            InternalContentBlock::ToolResult { tool_use_id, content, .. } => output.push(function_call_output_from_block(tool_use_id, content)?),
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

fn content_from_internal(blocks: &[InternalContentBlock]) -> Result<Value, FormatConversionError> {
    Ok(Value::Array(blocks.iter().map(block_from_internal).collect::<Result<Vec<_>, _>>()?))
}

fn block_from_internal(block: &InternalContentBlock) -> Result<Value, FormatConversionError> {
    match block {
        InternalContentBlock::Text { text, .. } => Ok(json!({ "type": "input_text", "text": text })),
        InternalContentBlock::Image { url: Some(url), .. } => Ok(json!({ "type": "input_image", "image_url": url })),
        InternalContentBlock::Image {
            data: Some(data), media_type, ..
        } => Ok(json!({
            "type": "input_image",
            "image_url": crate::format_conversion::data_url::format_base64_data_url(media_type.as_deref(), data, FORMAT)?,
        })),
        InternalContentBlock::File {
            file_id,
            file_url,
            data,
            filename,
            ..
        } => Ok(json!({ "type": "input_file", "file_id": file_id, "file_url": file_url, "file_data": data, "filename": filename })),
        InternalContentBlock::ToolUse { .. } | InternalContentBlock::ToolResult { .. } => Err(FormatConversionError::unsupported_content(
            FORMAT,
            "tool blocks must be top-level Responses items",
        )),
        InternalContentBlock::Thinking { .. } => Err(FormatConversionError::unsupported_content(
            FORMAT,
            "thinking blocks must be top-level Responses reasoning items",
        )),
        InternalContentBlock::Audio { .. } | InternalContentBlock::Image { .. } => Err(FormatConversionError::unsupported_content(
            FORMAT,
            "content block cannot be represented in OpenAI Responses",
        )),
    }
}

fn function_call_from_block(id: &str, name: &str, input: &Value) -> Result<Value, FormatConversionError> {
    Ok(json!({
        "type": "function_call",
        "call_id": id,
        "name": name,
        "arguments": serde_json::to_string(input).map_err(|error| FormatConversionError::invalid_payload(FORMAT, error.to_string()))?,
    }))
}

fn function_call_output_from_block(tool_use_id: &str, content: &[InternalContentBlock]) -> Result<Value, FormatConversionError> {
    Ok(json!({
        "type": "function_call_output",
        "call_id": tool_use_id,
        "output": text_from_blocks(content)?,
    }))
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

fn arguments_json(value: Option<&Value>) -> Result<Value, FormatConversionError> {
    value
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(|text| {
            serde_json::from_str(text)
                .map(|parsed| match parsed {
                    Value::Object(_) => parsed,
                    other => json!({ "_raw": other }),
                })
                .or_else(|_| Ok(json!({ "_raw": text })))
        })
        .transpose()
        .map(|value| value.unwrap_or_else(|| json!({})))
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

fn text_from_blocks(blocks: &[InternalContentBlock]) -> Result<String, FormatConversionError> {
    let mut output = String::new();
    for block in blocks {
        let Some(text) = block.plain_text() else {
            return Err(FormatConversionError::unsupported_content(FORMAT, "non-text block cannot be flattened"));
        };
        output.push_str(text);
    }
    Ok(output)
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

fn required_text<'a>(object: &'a Map<String, Value>, path: &str, key: &str) -> Result<&'a str, FormatConversionError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("{path}.{key}")))
}

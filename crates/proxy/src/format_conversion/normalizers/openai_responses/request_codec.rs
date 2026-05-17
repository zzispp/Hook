use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalMessage, InternalRole, InternalTool, InternalToolChoice};

use super::request_fields::FORMAT;

pub(super) fn input_messages(value: Option<&Value>) -> Result<Vec<InternalMessage>, FormatConversionError> {
    match value {
        Some(Value::String(text)) => Ok(vec![InternalMessage::text(InternalRole::User, text)]),
        Some(Value::Array(items)) => items.iter().enumerate().map(|(index, value)| input_item(value, index)).collect(),
        Some(_) | None => Err(FormatConversionError::invalid_payload(FORMAT, "$.input")),
    }
}

pub(super) fn messages_from_internal(messages: &[InternalMessage]) -> Result<Vec<Value>, FormatConversionError> {
    messages.iter().map(message_from_internal).collect()
}

pub(super) fn parse_tools(value: Option<&Value>) -> Result<Vec<InternalTool>, FormatConversionError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    value
        .as_array()
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, "$.tools"))?
        .iter()
        .enumerate()
        .map(parse_tool)
        .collect()
}

pub(super) fn parse_tool_choice(value: Option<&Value>) -> Result<Option<InternalToolChoice>, FormatConversionError> {
    match value {
        None => Ok(None),
        Some(Value::String(text)) if text == "auto" => Ok(Some(InternalToolChoice::Auto)),
        Some(Value::String(text)) if text == "none" => Ok(Some(InternalToolChoice::None)),
        Some(Value::String(text)) if text == "required" => Ok(Some(InternalToolChoice::Required)),
        Some(Value::Object(object)) => Ok(object.get("name").and_then(Value::as_str).map(|name| InternalToolChoice::Tool(name.to_owned()))),
        Some(_) => Err(FormatConversionError::invalid_payload(FORMAT, "$.tool_choice")),
    }
}

pub(super) fn tools_from_internal(tools: &[InternalTool]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            json!({
                "type": "function",
                "name": tool.name,
                "description": tool.description,
                "parameters": tool.parameters,
            })
        })
        .collect()
}

pub(super) fn tool_choice_from_internal(choice: &InternalToolChoice) -> Value {
    match choice {
        InternalToolChoice::Auto => Value::String("auto".into()),
        InternalToolChoice::None => Value::String("none".into()),
        InternalToolChoice::Required => Value::String("required".into()),
        InternalToolChoice::Tool(name) => json!({ "type": "function", "name": name }),
    }
}

fn input_item(value: &Value, index: usize) -> Result<InternalMessage, FormatConversionError> {
    let object = value
        .as_object()
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("$.input[{index}]")))?;
    match object.get("type").and_then(Value::as_str).unwrap_or("message") {
        "message" => message_item(object, index),
        "function_call" => function_call_item(object),
        "function_call_output" => function_call_output_item(object),
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
        role: InternalRole::Tool,
        content: vec![InternalContentBlock::ToolResult {
            tool_use_id: object.get("call_id").and_then(Value::as_str).unwrap_or_default().to_owned(),
            tool_name: None,
            content: vec![InternalContentBlock::Text(
                object.get("output").and_then(Value::as_str).unwrap_or_default().to_owned(),
            )],
            is_error: false,
        }],
    })
}

fn content_blocks(value: Option<&Value>, path: &str) -> Result<Vec<InternalContentBlock>, FormatConversionError> {
    match value {
        Some(Value::String(text)) => Ok(vec![InternalContentBlock::Text(text.clone())]),
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
        "input_text" | "output_text" | "text" => Ok(InternalContentBlock::Text(required_text(object, path, "text")?.to_owned())),
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

fn message_from_internal(message: &InternalMessage) -> Result<Value, FormatConversionError> {
    if let Some(tool_use) = function_call_from_internal(&message.content)? {
        return Ok(tool_use);
    }
    if let Some(tool_result) = function_call_output_from_internal(&message.content)? {
        return Ok(tool_result);
    }
    Ok(json!({
        "type": "message",
        "role": role_as_str(&message.role),
        "content": content_from_internal(&message.content)?,
    }))
}

fn content_from_internal(blocks: &[InternalContentBlock]) -> Result<Value, FormatConversionError> {
    Ok(Value::Array(blocks.iter().map(block_from_internal).collect::<Result<Vec<_>, _>>()?))
}

fn block_from_internal(block: &InternalContentBlock) -> Result<Value, FormatConversionError> {
    match block {
        InternalContentBlock::Text(text) => Ok(json!({ "type": "input_text", "text": text })),
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
        _ => Err(FormatConversionError::unsupported_content(
            FORMAT,
            "content block cannot be represented in OpenAI Responses",
        )),
    }
}

fn function_call_from_internal(blocks: &[InternalContentBlock]) -> Result<Option<Value>, FormatConversionError> {
    let Some(InternalContentBlock::ToolUse { id, name, input }) = blocks.first() else {
        return Ok(None);
    };
    Ok(Some(json!({
        "type": "function_call",
        "call_id": id,
        "name": name,
        "arguments": serde_json::to_string(input).map_err(|error| FormatConversionError::invalid_payload(FORMAT, error.to_string()))?,
    })))
}

fn function_call_output_from_internal(blocks: &[InternalContentBlock]) -> Result<Option<Value>, FormatConversionError> {
    let Some(InternalContentBlock::ToolResult { tool_use_id, content, .. }) = blocks.first() else {
        return Ok(None);
    };
    Ok(Some(json!({
        "type": "function_call_output",
        "call_id": tool_use_id,
        "output": text_from_blocks(content)?,
    })))
}

fn parse_tool(value: (usize, &Value)) -> Result<InternalTool, FormatConversionError> {
    let (index, value) = value;
    let object = value
        .as_object()
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("$.tools[{index}]")))?;
    Ok(InternalTool {
        name: required_text(object, &format!("$.tools[{index}]"), "name")?.to_owned(),
        description: object.get("description").and_then(Value::as_str).map(str::to_owned),
        parameters: object.get("parameters").cloned(),
    })
}

fn arguments_json(value: Option<&Value>) -> Result<Value, FormatConversionError> {
    value
        .and_then(Value::as_str)
        .map(|text| serde_json::from_str(text).map_err(|error| FormatConversionError::invalid_payload(FORMAT, error.to_string())))
        .transpose()
        .map(|value| value.unwrap_or_else(|| json!({})))
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
        "system" | "developer" => InternalRole::System,
        "tool" => InternalRole::Tool,
        _ => InternalRole::User,
    }
}

fn role_as_str(role: &InternalRole) -> &'static str {
    match role {
        InternalRole::System => "system",
        InternalRole::User => "user",
        InternalRole::Assistant => "assistant",
        InternalRole::Tool => "tool",
    }
}

fn required_text<'a>(object: &'a Map<String, Value>, path: &str, key: &str) -> Result<&'a str, FormatConversionError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("{path}.{key}")))
}

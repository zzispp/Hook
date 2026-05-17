use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalMessage, InternalRole, InternalTool, InternalToolChoice};

use super::common::{FORMAT, content_blocks, required_array, required_object, required_string, text_from_blocks};

pub(super) fn parse_request_messages(request: &Value) -> Result<Vec<InternalMessage>, FormatConversionError> {
    let source = required_array(request, "messages", "$.messages")?;
    let mut messages = Vec::with_capacity(source.len());
    for (index, value) in source.iter().enumerate() {
        messages.push(parse_request_message(value, index)?);
    }
    Ok(messages)
}

pub(super) fn request_messages_from_internal(messages: &[InternalMessage]) -> Result<Vec<Value>, FormatConversionError> {
    messages
        .iter()
        .map(|message| {
            let mut output = json!({
                "role": openai_role(&message.role),
                "content": content_from_internal(&message.content, &message.role)?,
            });
            if let Some(tool_call_id) = tool_result_id(message) {
                output["tool_call_id"] = Value::String(tool_call_id);
            }
            let tool_calls = tool_calls_from_blocks(&message.content)?;
            if !tool_calls.is_empty() {
                output["tool_calls"] = Value::Array(tool_calls);
            }
            Ok(output)
        })
        .collect()
}

pub(super) fn parse_tools(value: Option<&Value>) -> Result<Vec<InternalTool>, FormatConversionError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let tools = value.as_array().ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, "$.tools"))?;
    tools.iter().enumerate().map(parse_tool).collect()
}

pub(super) fn parse_tool_choice(value: Option<&Value>) -> Result<Option<InternalToolChoice>, FormatConversionError> {
    match value {
        None => Ok(None),
        Some(Value::String(text)) if text == "auto" => Ok(Some(InternalToolChoice::Auto)),
        Some(Value::String(text)) if text == "none" => Ok(Some(InternalToolChoice::None)),
        Some(Value::String(text)) if text == "required" => Ok(Some(InternalToolChoice::Required)),
        Some(Value::Object(object)) => {
            let function = required_object(object.get("function"), "$.tool_choice.function")?;
            Ok(function
                .get("name")
                .and_then(Value::as_str)
                .map(|name| InternalToolChoice::Tool(name.to_owned())))
        }
        Some(_) => Err(FormatConversionError::invalid_payload(FORMAT, "$.tool_choice")),
    }
}

pub(super) fn tools_from_internal(tools: &[InternalTool]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.parameters,
                },
            })
        })
        .collect()
}

pub(super) fn tool_choice_from_internal(choice: &InternalToolChoice) -> Value {
    match choice {
        InternalToolChoice::Auto => Value::String("auto".into()),
        InternalToolChoice::None => Value::String("none".into()),
        InternalToolChoice::Required => Value::String("required".into()),
        InternalToolChoice::Tool(name) => json!({ "type": "function", "function": { "name": name } }),
    }
}

fn parse_request_message(value: &Value, index: usize) -> Result<InternalMessage, FormatConversionError> {
    let object = required_object(Some(value), "$.messages[]")?;
    if object.get("tool_calls").is_some() {
        return parse_assistant_tool_calls(object, index);
    }
    let role = required_string(value, "role", &format!("$.messages[{index}].role"))?;
    Ok(InternalMessage {
        role: map_openai_role(&role)?,
        content: openai_message_content(object, role.as_str(), index)?,
    })
}

fn parse_assistant_tool_calls(object: &Map<String, Value>, index: usize) -> Result<InternalMessage, FormatConversionError> {
    let mut content = content_blocks(object.get("content"), &format!("$.messages[{index}].content"))?;
    for (tool_index, tool_call) in required_array(&Value::Object(object.clone()), "tool_calls", &format!("$.messages[{index}].tool_calls"))?
        .iter()
        .enumerate()
    {
        content.push(parse_tool_call(tool_call, index, tool_index)?);
    }
    Ok(InternalMessage {
        role: InternalRole::Assistant,
        content,
    })
}

fn openai_message_content(object: &Map<String, Value>, role: &str, index: usize) -> Result<Vec<InternalContentBlock>, FormatConversionError> {
    if role != "tool" {
        return content_blocks(object.get("content"), &format!("$.messages[{index}].content"));
    }
    let tool_use_id = required_string(&Value::Object(object.clone()), "tool_call_id", &format!("$.messages[{index}].tool_call_id"))?;
    Ok(vec![InternalContentBlock::ToolResult {
        tool_use_id,
        tool_name: None,
        content: content_blocks(object.get("content"), &format!("$.messages[{index}].content"))?,
        is_error: false,
    }])
}

fn parse_tool_call(value: &Value, message_index: usize, tool_index: usize) -> Result<InternalContentBlock, FormatConversionError> {
    let object = required_object(Some(value), &format!("$.messages[{message_index}].tool_calls[{tool_index}]"))?;
    let function = required_object(
        object.get("function"),
        &format!("$.messages[{message_index}].tool_calls[{tool_index}].function"),
    )?;
    let arguments = function
        .get("arguments")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(|text| serde_json::from_str(text).map_err(|error| FormatConversionError::invalid_payload(FORMAT, error.to_string())))
        .transpose()?
        .unwrap_or_else(|| json!({}));
    Ok(InternalContentBlock::ToolUse {
        id: object.get("id").and_then(Value::as_str).unwrap_or_default().to_owned(),
        name: function.get("name").and_then(Value::as_str).unwrap_or_default().to_owned(),
        input: arguments,
    })
}

fn parse_tool(value: (usize, &Value)) -> Result<InternalTool, FormatConversionError> {
    let (index, value) = value;
    let object = required_object(Some(value), &format!("$.tools[{index}]"))?;
    let function = required_object(object.get("function"), &format!("$.tools[{index}].function"))?;
    Ok(InternalTool {
        name: function.get("name").and_then(Value::as_str).unwrap_or_default().to_owned(),
        description: function.get("description").and_then(Value::as_str).map(str::to_owned),
        parameters: function.get("parameters").cloned(),
    })
}

fn content_from_internal(blocks: &[InternalContentBlock], role: &InternalRole) -> Result<Value, FormatConversionError> {
    if matches!(role, InternalRole::Tool) {
        return Ok(Value::String(tool_result_text(blocks)?));
    }
    if blocks.iter().all(|block| matches!(block, InternalContentBlock::Text(_))) {
        return text_from_blocks(blocks).map(Value::String);
    }
    let mut values = Vec::new();
    for block in blocks {
        if !matches!(block, InternalContentBlock::ToolUse { .. }) {
            values.push(block_from_internal(block)?);
        }
    }
    Ok(Value::Array(values))
}

fn block_from_internal(block: &InternalContentBlock) -> Result<Value, FormatConversionError> {
    match block {
        InternalContentBlock::Text(text) => Ok(json!({ "type": "text", "text": text })),
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
        InternalContentBlock::Thinking { .. } | InternalContentBlock::Image { .. } => Err(FormatConversionError::unsupported_content(
            FORMAT,
            "content block cannot be represented in OpenAI Chat",
        )),
        InternalContentBlock::ToolUse { .. } => Err(FormatConversionError::unsupported_content(FORMAT, "tool_use must be encoded as tool_calls")),
    }
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

fn tool_result_id(message: &InternalMessage) -> Option<String> {
    message.content.iter().find_map(|block| match block {
        InternalContentBlock::ToolResult { tool_use_id, .. } => Some(tool_use_id.clone()),
        _ => None,
    })
}

fn tool_result_text(blocks: &[InternalContentBlock]) -> Result<String, FormatConversionError> {
    let Some(InternalContentBlock::ToolResult { content, .. }) = blocks.first() else {
        return text_from_blocks(blocks);
    };
    text_from_blocks(content)
}

fn map_openai_role(value: &str) -> Result<InternalRole, FormatConversionError> {
    match value {
        "system" | "developer" => Ok(InternalRole::System),
        "user" => Ok(InternalRole::User),
        "assistant" => Ok(InternalRole::Assistant),
        "tool" => Ok(InternalRole::Tool),
        _ => Err(FormatConversionError::invalid_payload(FORMAT, format!("unknown role: {value}"))),
    }
}

fn openai_role(role: &InternalRole) -> &'static str {
    match role {
        InternalRole::System => "system",
        InternalRole::User => "user",
        InternalRole::Assistant => "assistant",
        InternalRole::Tool => "tool",
    }
}

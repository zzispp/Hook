use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalMessage, InternalRole, InternalToolKind};

use super::common::{FORMAT, content_blocks, required_array, required_object, required_string};
pub(super) use super::request_messages::request_messages_from_internal;
pub(super) use super::request_tools::{parse_tool_choice, parse_tools, tool_choice_from_internal, tools_from_internal};

pub(super) fn parse_request_messages(request: &Value) -> Result<Vec<InternalMessage>, FormatConversionError> {
    let source = required_array(request, "messages", "$.messages")?;
    let mut messages = Vec::with_capacity(source.len());
    for (index, value) in source.iter().enumerate() {
        messages.push(parse_request_message(value, index)?);
    }
    Ok(messages)
}

fn parse_request_message(value: &Value, index: usize) -> Result<InternalMessage, FormatConversionError> {
    let object = required_object(Some(value), "$.messages[]")?;
    if object.get("tool_calls").is_some() {
        return parse_assistant_tool_calls(object, index);
    }
    if object.get("function_call").is_some() {
        return parse_assistant_function_call(object, index);
    }
    let role = required_string(value, "role", &format!("$.messages[{index}].role"))?;
    Ok(InternalMessage {
        role: map_openai_role(&role)?,
        content: openai_message_content(object, role.as_str(), index)?,
    })
}

fn parse_assistant_tool_calls(object: &Map<String, Value>, index: usize) -> Result<InternalMessage, FormatConversionError> {
    let mut content = content_blocks(object.get("content"), &format!("$.messages[{index}].content"))?;
    if let Some(reasoning) = reasoning_content(object) {
        content.insert(0, reasoning);
    }
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

fn parse_assistant_function_call(object: &Map<String, Value>, index: usize) -> Result<InternalMessage, FormatConversionError> {
    let mut content = content_blocks(object.get("content"), &format!("$.messages[{index}].content"))?;
    if let Some(reasoning) = reasoning_content(object) {
        content.insert(0, reasoning);
    }
    let function_call = required_object(object.get("function_call"), &format!("$.messages[{index}].function_call"))?;
    if let Some(tool_call) = parse_legacy_function_call(function_call)? {
        content.push(tool_call);
    }
    Ok(InternalMessage {
        role: InternalRole::Assistant,
        content,
    })
}

fn openai_message_content(object: &Map<String, Value>, role: &str, index: usize) -> Result<Vec<InternalContentBlock>, FormatConversionError> {
    if role != "tool" {
        let mut blocks = content_blocks(object.get("content"), &format!("$.messages[{index}].content"))?;
        if role == "assistant" {
            if let Some(reasoning) = reasoning_content(object) {
                blocks.insert(0, reasoning);
            }
        }
        return Ok(blocks);
    }
    Ok(vec![InternalContentBlock::ToolResult {
        tool_use_id: object.get("tool_call_id").and_then(Value::as_str).unwrap_or_default().to_owned(),
        tool_name: None,
        tool_kind: InternalToolKind::Function,
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
    let arguments = function_arguments(function.get("arguments"))?;
    Ok(InternalContentBlock::ToolUse {
        id: object.get("id").and_then(Value::as_str).unwrap_or_default().to_owned(),
        name: function.get("name").and_then(Value::as_str).unwrap_or_default().to_owned(),
        input: arguments,
        kind: InternalToolKind::Function,
    })
}

fn parse_legacy_function_call(function: &Map<String, Value>) -> Result<Option<InternalContentBlock>, FormatConversionError> {
    let Some(name) = function.get("name").and_then(Value::as_str).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let input = function_arguments(function.get("arguments"))?;
    Ok(Some(InternalContentBlock::ToolUse {
        id: "call_0".to_owned(),
        name: name.to_owned(),
        input,
        kind: InternalToolKind::Function,
    }))
}

fn function_arguments(value: Option<&Value>) -> Result<Value, FormatConversionError> {
    value
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(|text| {
            serde_json::from_str(text)
                .map(|parsed| match parsed {
                    Value::Object(_) => parsed,
                    other => json!({ "raw": other }),
                })
                .or_else(|_| Ok(json!({ "raw": text })))
        })
        .transpose()
        .map(|value| value.unwrap_or_else(|| json!({})))
}

fn reasoning_content(object: &Map<String, Value>) -> Option<InternalContentBlock> {
    object
        .get("reasoning_content")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty() && *text != "[undefined]")
        .map(|text| InternalContentBlock::Thinking {
            text: text.to_owned(),
            signature: None,
        })
}

fn map_openai_role(value: &str) -> Result<InternalRole, FormatConversionError> {
    match value {
        "system" => Ok(InternalRole::System),
        "developer" => Ok(InternalRole::Developer),
        "user" => Ok(InternalRole::User),
        "assistant" => Ok(InternalRole::Assistant),
        "tool" => Ok(InternalRole::Tool),
        _ => Err(FormatConversionError::invalid_payload(FORMAT, format!("unknown role: {value}"))),
    }
}

pub(super) fn openai_role(role: &InternalRole) -> &'static str {
    match role {
        InternalRole::System => "system",
        InternalRole::Developer => "developer",
        InternalRole::User => "user",
        InternalRole::Assistant => "assistant",
        InternalRole::Tool => "tool",
        InternalRole::Unknown(_) => "user",
    }
}

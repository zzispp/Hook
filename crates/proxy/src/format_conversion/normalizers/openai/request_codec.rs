use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalMessage, InternalRole, InternalTool, InternalToolChoice};

use super::common::{FORMAT, content_blocks, required_array, required_object, required_string};
pub(super) use super::request_messages::request_messages_from_internal;

pub(super) fn parse_request_messages(request: &Value) -> Result<Vec<InternalMessage>, FormatConversionError> {
    let source = required_array(request, "messages", "$.messages")?;
    let mut messages = Vec::with_capacity(source.len());
    for (index, value) in source.iter().enumerate() {
        messages.push(parse_request_message(value, index)?);
    }
    Ok(messages)
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
        Some(Value::String(_)) => Ok(Some(InternalToolChoice::Auto)),
        Some(Value::Object(object)) => {
            if object.get("type").and_then(Value::as_str) == Some("function") {
                let function = required_object(object.get("function"), "$.tool_choice.function")?;
                return Ok(function
                    .get("name")
                    .and_then(Value::as_str)
                    .map(|name| InternalToolChoice::Tool(name.to_owned())));
            }
            if object.get("type").and_then(Value::as_str) == Some("custom") {
                return Ok(custom_tool_name(object).map(InternalToolChoice::Tool).or(Some(InternalToolChoice::Auto)));
            }
            Ok(Some(InternalToolChoice::Auto))
        }
        Some(_) => Ok(Some(InternalToolChoice::Auto)),
    }
}

pub(super) fn tools_from_internal(tools: &[InternalTool]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            if let Some(raw) = tool.extra.get("openai_chat_raw_tool").and_then(Value::as_object) {
                return Value::Object(raw.clone());
            }
            if let Some(raw) = tool.extra.get("openai_responses_raw_tool").and_then(Value::as_object) {
                if let Some(tool) = responses_tool_to_chat_tool(raw) {
                    return tool;
                }
            }
            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.parameters.as_ref().map(crate::format_conversion::schema_utils::openai_schema_with_object_fixes),
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

fn parse_tool(value: (usize, &Value)) -> Result<InternalTool, FormatConversionError> {
    let (index, value) = value;
    let object = required_object(Some(value), &format!("$.tools[{index}]"))?;
    if object.get("type").and_then(Value::as_str) == Some("custom") {
        return parse_custom_tool(object, index);
    }
    if object.get("type").and_then(Value::as_str) != Some("function") {
        return Err(FormatConversionError::invalid_payload(FORMAT, format!("$.tools[{index}].function")));
    }
    let function = required_object(object.get("function"), &format!("$.tools[{index}].function"))?;
    Ok(InternalTool {
        name: function.get("name").and_then(Value::as_str).unwrap_or_default().to_owned(),
        description: function.get("description").and_then(Value::as_str).map(str::to_owned),
        parameters: function.get("parameters").cloned(),
        extra: Map::new(),
    })
}

fn parse_custom_tool(object: &Map<String, Value>, index: usize) -> Result<InternalTool, FormatConversionError> {
    let custom = required_object(object.get("custom"), &format!("$.tools[{index}].custom"))?;
    let name = custom
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("$.tools[{index}].custom.name")))?
        .to_owned();
    let mut extra = Map::new();
    extra.insert("openai_chat_raw_tool".into(), Value::Object(object.clone()));
    Ok(InternalTool {
        name,
        description: custom.get("description").and_then(Value::as_str).map(str::to_owned),
        parameters: None,
        extra,
    })
}

fn custom_tool_name(object: &Map<String, Value>) -> Option<String> {
    object
        .get("custom")
        .and_then(Value::as_object)
        .and_then(|custom| custom.get("name"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn responses_tool_to_chat_tool(tool: &Map<String, Value>) -> Option<Value> {
    match tool.get("type").and_then(Value::as_str)? {
        "function" => responses_function_tool_to_chat(tool),
        "custom" => responses_custom_tool_to_chat(tool),
        _ => None,
    }
}

fn responses_function_tool_to_chat(tool: &Map<String, Value>) -> Option<Value> {
    let name = tool.get("name").and_then(Value::as_str).filter(|value| !value.is_empty())?;
    let mut function = Map::new();
    function.insert("name".into(), Value::String(name.to_owned()));
    insert_optional_clone(&mut function, tool, "description");
    insert_optional_clone(&mut function, tool, "parameters");
    insert_optional_clone(&mut function, tool, "strict");
    Some(json!({ "type": "function", "function": function }))
}

fn responses_custom_tool_to_chat(tool: &Map<String, Value>) -> Option<Value> {
    let name = tool.get("name").and_then(Value::as_str).filter(|value| !value.is_empty())?;
    let mut custom = Map::new();
    custom.insert("name".into(), Value::String(name.to_owned()));
    insert_optional_clone(&mut custom, tool, "description");
    insert_optional_clone(&mut custom, tool, "format");
    Some(json!({ "type": "custom", "custom": custom }))
}

fn insert_optional_clone(output: &mut Map<String, Value>, source: &Map<String, Value>, key: &str) {
    if let Some(value) = source.get(key) {
        output.insert(key.to_owned(), value.clone());
    }
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

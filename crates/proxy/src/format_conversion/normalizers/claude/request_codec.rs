use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalMessage, InternalRole, InternalTool, InternalToolChoice};

use super::common::{FORMAT, required_array, required_object};

pub(super) fn system_messages(request: &Value) -> Result<Vec<InternalMessage>, FormatConversionError> {
    match request.get("system") {
        Some(Value::String(text)) if !text.is_empty() => Ok(vec![InternalMessage::text(InternalRole::System, text)]),
        Some(Value::Array(blocks)) => Ok(vec![InternalMessage {
            role: InternalRole::System,
            content: claude_blocks(blocks, "$.system")?,
        }]),
        Some(_) => Err(FormatConversionError::invalid_payload(FORMAT, "$.system")),
        None => Ok(Vec::new()),
    }
}

pub(super) fn parse_messages(request: &Value) -> Result<Vec<InternalMessage>, FormatConversionError> {
    required_array(request, "messages", "$.messages")?
        .iter()
        .enumerate()
        .map(|(index, value)| parse_message(value, index))
        .collect()
}

pub(super) fn messages_from_internal(messages: &[InternalMessage]) -> Result<Vec<Value>, FormatConversionError> {
    messages
        .iter()
        .filter(|message| message.role != InternalRole::System)
        .map(|message| {
            Ok(json!({
                "role": message_role(&message.role),
                "content": content_from_internal(&message.content)?,
            }))
        })
        .collect()
}

pub(super) fn joined_system(messages: &[InternalMessage]) -> Result<Option<String>, FormatConversionError> {
    let system = messages
        .iter()
        .filter(|message| message.role == InternalRole::System)
        .map(InternalMessage::text_content)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>();
    Ok((!system.is_empty()).then(|| system.join("\n\n")))
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
        Some(Value::Object(object)) => match object.get("type").and_then(Value::as_str).unwrap_or_default() {
            "auto" => Ok(Some(InternalToolChoice::Auto)),
            "none" => Ok(Some(InternalToolChoice::None)),
            "any" => Ok(Some(InternalToolChoice::Required)),
            "tool" => Ok(object.get("name").and_then(Value::as_str).map(|name| InternalToolChoice::Tool(name.to_owned()))),
            _ => Err(FormatConversionError::invalid_payload(FORMAT, "$.tool_choice.type")),
        },
        Some(_) => Err(FormatConversionError::invalid_payload(FORMAT, "$.tool_choice")),
    }
}

pub(super) fn tools_from_internal(tools: &[InternalTool]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "input_schema": tool.parameters,
            })
        })
        .collect()
}

pub(super) fn tool_choice_from_internal(choice: &InternalToolChoice) -> Value {
    match choice {
        InternalToolChoice::Auto => json!({ "type": "auto" }),
        InternalToolChoice::None => json!({ "type": "none" }),
        InternalToolChoice::Required => json!({ "type": "any" }),
        InternalToolChoice::Tool(name) => json!({ "type": "tool", "name": name }),
    }
}

fn parse_message(value: &Value, index: usize) -> Result<InternalMessage, FormatConversionError> {
    let object = required_object(Some(value), &format!("$.messages[{index}]"))?;
    let role = claude_role(object.get("role").and_then(Value::as_str).unwrap_or("user"));
    Ok(InternalMessage {
        role,
        content: claude_content(object.get("content"), &format!("$.messages[{index}].content"))?,
    })
}

fn claude_content(value: Option<&Value>, path: &str) -> Result<Vec<InternalContentBlock>, FormatConversionError> {
    match value {
        Some(Value::String(text)) => Ok(vec![InternalContentBlock::Text(text.to_owned())]),
        Some(Value::Array(blocks)) => claude_blocks(blocks, path),
        Some(_) => Err(FormatConversionError::invalid_payload(FORMAT, path)),
        None => Ok(Vec::new()),
    }
}

fn claude_blocks(blocks: &[Value], path: &str) -> Result<Vec<InternalContentBlock>, FormatConversionError> {
    blocks
        .iter()
        .enumerate()
        .map(|(index, block)| claude_block(block, &format!("{path}[{index}]")))
        .collect()
}

fn claude_block(value: &Value, path: &str) -> Result<InternalContentBlock, FormatConversionError> {
    let object = required_object(Some(value), path)?;
    match object.get("type").and_then(Value::as_str).unwrap_or("text") {
        "text" => Ok(InternalContentBlock::Text(required_text(object, path, "text")?.to_owned())),
        "thinking" => Ok(InternalContentBlock::Thinking {
            text: required_text(object, path, "thinking")?.to_owned(),
            signature: object.get("signature").and_then(Value::as_str).map(str::to_owned),
        }),
        "image" => claude_image_block(object, path),
        "document" => claude_file_block(object, path),
        "tool_use" => Ok(InternalContentBlock::ToolUse {
            id: required_text(object, path, "id")?.to_owned(),
            name: required_text(object, path, "name")?.to_owned(),
            input: object.get("input").cloned().unwrap_or_else(|| json!({})),
        }),
        "tool_result" => Ok(InternalContentBlock::ToolResult {
            tool_use_id: required_text(object, path, "tool_use_id")?.to_owned(),
            tool_name: None,
            content: claude_content(object.get("content"), &format!("{path}.content"))?,
            is_error: object.get("is_error").and_then(Value::as_bool).unwrap_or(false),
        }),
        other => Err(FormatConversionError::unsupported_content(
            FORMAT,
            format!("{path}: unsupported block type {other}"),
        )),
    }
}

fn claude_image_block(object: &Map<String, Value>, path: &str) -> Result<InternalContentBlock, FormatConversionError> {
    let source = required_object(object.get("source"), &format!("{path}.source"))?;
    Ok(InternalContentBlock::Image {
        url: source.get("url").and_then(Value::as_str).map(str::to_owned),
        data: source.get("data").and_then(Value::as_str).map(str::to_owned),
        media_type: source.get("media_type").and_then(Value::as_str).map(str::to_owned),
    })
}

fn claude_file_block(object: &Map<String, Value>, path: &str) -> Result<InternalContentBlock, FormatConversionError> {
    let source = required_object(object.get("source"), &format!("{path}.source"))?;
    Ok(InternalContentBlock::File {
        file_id: None,
        file_url: source.get("url").and_then(Value::as_str).map(str::to_owned),
        data: source.get("data").and_then(Value::as_str).map(str::to_owned),
        media_type: source.get("media_type").and_then(Value::as_str).map(str::to_owned),
        filename: object.get("title").and_then(Value::as_str).map(str::to_owned),
    })
}

fn content_from_internal(blocks: &[InternalContentBlock]) -> Result<Value, FormatConversionError> {
    Ok(Value::Array(blocks.iter().map(block_from_internal).collect::<Result<Vec<_>, _>>()?))
}

fn block_from_internal(block: &InternalContentBlock) -> Result<Value, FormatConversionError> {
    match block {
        InternalContentBlock::Text(text) => Ok(json!({ "type": "text", "text": text })),
        InternalContentBlock::Thinking { text, signature } => Ok(json!({ "type": "thinking", "thinking": text, "signature": signature })),
        InternalContentBlock::Image { url: Some(url), .. } => Ok(json!({ "type": "image", "source": { "type": "url", "url": url } })),
        InternalContentBlock::Image {
            data: Some(data), media_type, ..
        } => Ok(json!({ "type": "image", "source": { "type": "base64", "media_type": media_type, "data": data } })),
        InternalContentBlock::File {
            file_url: Some(url),
            media_type,
            filename,
            ..
        } => Ok(json!({ "type": "document", "title": filename, "source": { "type": "url", "media_type": media_type, "url": url } })),
        InternalContentBlock::File {
            data: Some(data),
            media_type,
            filename,
            ..
        } => Ok(json!({ "type": "document", "title": filename, "source": { "type": "base64", "media_type": media_type, "data": data } })),
        InternalContentBlock::ToolUse { id, name, input } => Ok(json!({ "type": "tool_use", "id": id, "name": name, "input": input })),
        InternalContentBlock::ToolResult {
            tool_use_id,
            content,
            is_error,
            ..
        } => Ok(json!({
            "type": "tool_result",
            "tool_use_id": tool_use_id,
            "content": content_from_internal(content)?,
            "is_error": is_error,
        })),
        InternalContentBlock::Audio { .. } | InternalContentBlock::Image { .. } | InternalContentBlock::File { .. } => Err(
            FormatConversionError::unsupported_content(FORMAT, "content block cannot be represented in Claude Messages"),
        ),
    }
}

fn parse_tool(value: (usize, &Value)) -> Result<InternalTool, FormatConversionError> {
    let (index, value) = value;
    let object = required_object(Some(value), &format!("$.tools[{index}]"))?;
    Ok(InternalTool {
        name: required_text(object, &format!("$.tools[{index}]"), "name")?.to_owned(),
        description: object.get("description").and_then(Value::as_str).map(str::to_owned),
        parameters: object.get("input_schema").cloned(),
    })
}

fn claude_role(value: &str) -> InternalRole {
    if value == "assistant" { InternalRole::Assistant } else { InternalRole::User }
}

fn message_role(role: &InternalRole) -> &'static str {
    match role {
        InternalRole::Assistant => "assistant",
        _ => "user",
    }
}

fn required_text<'a>(object: &'a Map<String, Value>, path: &str, key: &str) -> Result<&'a str, FormatConversionError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("{path}.{key}")))
}

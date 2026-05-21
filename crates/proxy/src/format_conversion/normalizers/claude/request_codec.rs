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
    super::message_grouping::messages_from_internal(messages)
}

pub(super) fn system_from_internal(messages: &[InternalMessage]) -> Result<Option<Value>, FormatConversionError> {
    let system_messages = messages
        .iter()
        .filter(|message| matches!(message.role, InternalRole::System | InternalRole::Developer))
        .collect::<Vec<_>>();
    if system_messages.is_empty() {
        return Ok(None);
    }
    if system_messages.iter().any(|message| message.content.iter().any(text_has_cache_control)) {
        let blocks = system_messages
            .iter()
            .flat_map(|message| message.content.iter())
            .filter_map(system_text_block)
            .collect::<Vec<_>>();
        return Ok((!blocks.is_empty()).then(|| Value::Array(blocks)));
    }
    let system = system_messages
        .iter()
        .map(|message| message.text_content())
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>();
    Ok((!system.is_empty()).then(|| Value::String(system.join("\n\n"))))
}

fn text_has_cache_control(block: &InternalContentBlock) -> bool {
    matches!(block, InternalContentBlock::Text { cache_control: Some(_), .. })
}

fn system_text_block(block: &InternalContentBlock) -> Option<Value> {
    let InternalContentBlock::Text { text, cache_control } = block else {
        return None;
    };
    if text.is_empty() {
        return None;
    }
    Some(claude_text_from_internal(text, cache_control.as_ref()))
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
            "any" | "required" => Ok(Some(InternalToolChoice::Required)),
            "tool" | "tool_use" => Ok(object.get("name").and_then(Value::as_str).map(|name| InternalToolChoice::Tool(name.to_owned()))),
            _ => Ok(Some(InternalToolChoice::Auto)),
        },
        Some(_) => Ok(Some(InternalToolChoice::Auto)),
    }
}

pub(super) fn tools_from_internal(tools: &[InternalTool]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            if let Some(tool) = web_search_tool_from_internal(tool) {
                return tool;
            }
            json!({
                "name": tool.name,
                "description": tool.description,
                "input_schema": tool.parameters.clone().unwrap_or_else(empty_object_schema),
            })
        })
        .collect()
}

pub(super) fn web_search_tool_from_options(options: &Value) -> Value {
    let context_size = options.get("search_context_size").and_then(Value::as_str).unwrap_or("medium");
    let max_uses = web_search_max_uses(context_size);
    let mut output = Map::new();
    output.insert("type".into(), Value::String("web_search_20250305".into()));
    output.insert("name".into(), Value::String("web_search".into()));
    output.insert("max_uses".into(), Value::Number(max_uses.into()));
    if let Some(user_location) = options.get("user_location") {
        output.insert("user_location".into(), user_location.clone());
    }
    Value::Object(output)
}

fn empty_object_schema() -> Value {
    json!({ "type": "object", "properties": {} })
}

fn web_search_tool_from_internal(tool: &InternalTool) -> Option<Value> {
    Some(web_search_tool_from_options(tool.extra.get("web_search_options")?))
}

fn web_search_max_uses(context_size: &str) -> u64 {
    match context_size {
        "low" => 1,
        "high" => 10,
        _ => 5,
    }
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
        content: content_blocks_from_claude(object.get("content"), &format!("$.messages[{index}].content"))?,
    })
}

pub(super) fn content_blocks_from_claude(value: Option<&Value>, path: &str) -> Result<Vec<InternalContentBlock>, FormatConversionError> {
    match value {
        Some(Value::String(text)) => Ok(vec![InternalContentBlock::text(text.to_owned())]),
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
        "text" => Ok(claude_text_block(object, path)?),
        "thinking" => Ok(InternalContentBlock::Thinking {
            text: required_text(object, path, "thinking")?.to_owned(),
            signature: object.get("signature").and_then(Value::as_str).map(str::to_owned),
        }),
        "image" => claude_image_block(object, path),
        "document" => claude_file_block(object, path),
        "tool_use" => Ok(InternalContentBlock::ToolUse {
            id: required_text(object, path, "id")?.to_owned(),
            name: required_text(object, path, "name")?.to_owned(),
            input: claude_tool_input(object.get("input")),
        }),
        "tool_result" => claude_tool_result_block(object, path),
        other => Err(FormatConversionError::unsupported_content(
            FORMAT,
            format!("{path}: unsupported block type {other}"),
        )),
    }
}

fn claude_tool_input(value: Option<&Value>) -> Value {
    match value {
        Some(Value::Object(_)) => value.cloned().unwrap_or_else(|| json!({})),
        Some(value) => json!({ "raw": value }),
        None => json!({ "raw": Value::Null }),
    }
}

fn claude_tool_result_block(object: &Map<String, Value>, path: &str) -> Result<InternalContentBlock, FormatConversionError> {
    Ok(InternalContentBlock::ToolResult {
        tool_use_id: required_text(object, path, "tool_use_id")?.to_owned(),
        tool_name: None,
        content: claude_tool_result_content(object.get("content"))?,
        is_error: object.get("is_error").and_then(Value::as_bool).unwrap_or(false),
    })
}

fn claude_tool_result_content(value: Option<&Value>) -> Result<Vec<InternalContentBlock>, FormatConversionError> {
    Ok(match value {
        Some(Value::String(text)) => Ok(vec![InternalContentBlock::text(text.to_owned())]),
        Some(Value::Array(parts)) => Ok(vec![InternalContentBlock::text(claude_tool_result_text_parts(parts))]),
        Some(value) => Ok(vec![InternalContentBlock::text(value.to_string())]),
        None => Ok(Vec::new()),
    }?)
}

fn claude_tool_result_text_parts(parts: &[Value]) -> String {
    parts
        .iter()
        .filter_map(|part| part.as_object())
        .filter(|part| part.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|part| part.get("text").and_then(Value::as_str))
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn claude_text_block(object: &Map<String, Value>, path: &str) -> Result<InternalContentBlock, FormatConversionError> {
    let text = required_text(object, path, "text")?.to_owned();
    match object.get("cache_control") {
        Some(Value::Object(_)) => Ok(InternalContentBlock::text_with_cache_control(text, object["cache_control"].clone())),
        _ => Ok(InternalContentBlock::text(text)),
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

pub(super) fn content_from_internal(blocks: &[InternalContentBlock]) -> Result<Value, FormatConversionError> {
    Ok(Value::Array(blocks.iter().map(block_from_internal).collect::<Result<Vec<_>, _>>()?))
}

fn block_from_internal(block: &InternalContentBlock) -> Result<Value, FormatConversionError> {
    match block {
        InternalContentBlock::Text { text, cache_control } => Ok(claude_text_from_internal(text, cache_control.as_ref())),
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
            "content": tool_result_content(content)?,
            "is_error": is_error,
        })),
        InternalContentBlock::Audio { .. } | InternalContentBlock::Image { .. } | InternalContentBlock::File { .. } => Err(
            FormatConversionError::unsupported_content(FORMAT, "content block cannot be represented in Claude Messages"),
        ),
    }
}

fn claude_text_from_internal(text: &str, cache_control: Option<&Value>) -> Value {
    let mut output = json!({ "type": "text", "text": text });
    if let Some(cache_control) = cache_control {
        output["cache_control"] = cache_control.clone();
    }
    output
}

fn tool_result_content(content: &[InternalContentBlock]) -> Result<Value, FormatConversionError> {
    if content.is_empty() {
        return Ok(Value::String(String::new()));
    }
    let text = content.iter().map(|block| block.plain_text().map(str::to_owned)).collect::<Option<Vec<_>>>();
    let Some(parts) = text else {
        return content_from_internal(content);
    };
    let joined = parts.join("\n\n");
    serde_json::from_str::<Value>(&joined).or(Ok(Value::String(joined)))
}

fn parse_tool(value: (usize, &Value)) -> Result<InternalTool, FormatConversionError> {
    let (index, value) = value;
    let object = required_object(Some(value), &format!("$.tools[{index}]"))?;
    Ok(InternalTool {
        name: required_text(object, &format!("$.tools[{index}]"), "name")?.to_owned(),
        description: object.get("description").and_then(Value::as_str).map(str::to_owned),
        parameters: object.get("input_schema").cloned(),
        extra: Map::new(),
    })
}

fn claude_role(value: &str) -> InternalRole {
    if value == "assistant" { InternalRole::Assistant } else { InternalRole::User }
}

pub(super) fn message_role(role: &InternalRole) -> &'static str {
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

use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalToolKind};

use super::common::{FORMAT, required_object};

pub(super) fn content_blocks_from_claude(value: Option<&Value>, path: &str) -> Result<Vec<InternalContentBlock>, FormatConversionError> {
    match value {
        Some(Value::String(text)) => Ok(vec![InternalContentBlock::text(text.to_owned())]),
        Some(Value::Array(blocks)) => claude_blocks(blocks, path),
        Some(_) => Err(FormatConversionError::invalid_payload(FORMAT, path)),
        None => Ok(Vec::new()),
    }
}

pub(super) fn content_from_internal(blocks: &[InternalContentBlock]) -> Result<Value, FormatConversionError> {
    Ok(Value::Array(blocks.iter().map(block_from_internal).collect::<Result<Vec<_>, _>>()?))
}

pub(super) fn claude_text_from_internal(text: &str, cache_control: Option<&Value>) -> Value {
    let mut output = json!({ "type": "text", "text": text });
    if let Some(cache_control) = cache_control {
        output["cache_control"] = cache_control.clone();
    }
    output
}

pub(super) fn required_text<'a>(object: &'a Map<String, Value>, path: &str, key: &str) -> Result<&'a str, FormatConversionError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("{path}.{key}")))
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
        "text" => claude_text_block(object, path),
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
            kind: InternalToolKind::Function,
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
        tool_kind: InternalToolKind::Function,
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
        InternalContentBlock::ToolUse { id, name, input, .. } => Ok(json!({ "type": "tool_use", "id": id, "name": name, "input": input })),
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

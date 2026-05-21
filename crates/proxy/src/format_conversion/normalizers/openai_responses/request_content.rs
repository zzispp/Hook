use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock};

use super::request_fields::FORMAT;

pub(super) fn content_blocks(value: Option<&Value>, path: &str) -> Result<Vec<InternalContentBlock>, FormatConversionError> {
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

pub(super) fn content_from_internal(blocks: &[InternalContentBlock]) -> Result<Value, FormatConversionError> {
    Ok(Value::Array(blocks.iter().map(block_from_internal).collect::<Result<Vec<_>, _>>()?))
}

pub(super) fn tool_output_from_internal(blocks: &[InternalContentBlock]) -> Result<Value, FormatConversionError> {
    match blocks {
        [] => Ok(Value::String(String::new())),
        [InternalContentBlock::Text { text, .. }] => Ok(Value::String(text.clone())),
        _ => Ok(Value::Array(blocks.iter().map(tool_output_block_from_internal).collect::<Result<Vec<_>, _>>()?)),
    }
}

fn content_block(value: &Value, path: &str) -> Result<InternalContentBlock, FormatConversionError> {
    let object = value.as_object().ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, path))?;
    match object.get("type").and_then(Value::as_str).unwrap_or_default() {
        "input_text" | "output_text" | "text" => Ok(InternalContentBlock::text(required_text(object, path, "text")?.to_owned())),
        "input_image" => image_block(object, path),
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

fn image_block(object: &Map<String, Value>, path: &str) -> Result<InternalContentBlock, FormatConversionError> {
    let image_url = required_text(object, path, "image_url")?;
    if let Some((media_type, data)) = crate::format_conversion::data_url::parse_base64_data_url(image_url, FORMAT, &format!("{path}.image_url"))? {
        return Ok(InternalContentBlock::Image {
            url: None,
            data: Some(data),
            media_type: Some(media_type),
        });
    }
    Ok(InternalContentBlock::Image {
        url: Some(image_url.to_owned()),
        data: None,
        media_type: None,
    })
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

fn tool_output_block_from_internal(block: &InternalContentBlock) -> Result<Value, FormatConversionError> {
    match block {
        InternalContentBlock::Text { text, .. } => Ok(json!({ "type": "input_text", "text": text })),
        InternalContentBlock::Image { url: Some(url), .. } => Ok(json!({ "type": "input_image", "image_url": url })),
        InternalContentBlock::Image {
            data: Some(data), media_type, ..
        } => Ok(json!({
            "type": "input_image",
            "image_url": crate::format_conversion::data_url::format_base64_data_url(media_type.as_deref(), data, FORMAT)?,
        })),
        _ => Err(FormatConversionError::unsupported_content(
            FORMAT,
            "tool output block cannot be represented in OpenAI Responses",
        )),
    }
}

fn required_text<'a>(object: &'a Map<String, Value>, path: &str, key: &str) -> Result<&'a str, FormatConversionError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("{path}.{key}")))
}

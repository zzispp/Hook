use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalToolKind};

use super::common::FORMAT;

pub(super) fn parts_from_internal(blocks: &[InternalContentBlock]) -> Result<Vec<Value>, FormatConversionError> {
    blocks.iter().map(part_from_internal).collect()
}

fn part_from_internal(block: &InternalContentBlock) -> Result<Value, FormatConversionError> {
    match block {
        InternalContentBlock::Text { text, .. } => Ok(json!({ "text": text })),
        InternalContentBlock::Thinking { text, signature } => Ok(json!({ "text": text, "thought": true, "thoughtSignature": signature })),
        InternalContentBlock::Image {
            data: Some(data), media_type, ..
        } => Ok(json!({ "inlineData": { "mimeType": media_type, "data": data } })),
        InternalContentBlock::Image {
            url: Some(url), media_type, ..
        } => file_data_part(media_type.as_deref(), url),
        InternalContentBlock::File {
            file_url: Some(url),
            media_type,
            ..
        } => file_data_part(media_type.as_deref(), url),
        InternalContentBlock::Audio { data, media_type, .. } => Ok(json!({ "inlineData": { "mimeType": media_type, "data": data } })),
        InternalContentBlock::ToolUse { id, name, input, kind } if *kind == InternalToolKind::Function => function_call_part(id, name, input),
        InternalContentBlock::ToolUse { .. } => Err(FormatConversionError::unsupported_content(
            FORMAT,
            "Gemini functionCall cannot represent custom tool calls",
        )),
        InternalContentBlock::ToolResult {
            tool_use_id,
            tool_name,
            tool_kind,
            content,
            ..
        } if *tool_kind == InternalToolKind::Function => function_response_part(tool_use_id, tool_name, content),
        InternalContentBlock::ToolResult { .. } => Err(FormatConversionError::unsupported_content(
            FORMAT,
            "Gemini functionResponse cannot represent custom tool results",
        )),
        InternalContentBlock::File {
            data: Some(data), media_type, ..
        } => Ok(json!({ "inlineData": { "mimeType": media_type, "data": data } })),
        InternalContentBlock::Image { .. } | InternalContentBlock::File { .. } => Err(FormatConversionError::unsupported_content(
            FORMAT,
            "content block cannot be represented in Gemini",
        )),
    }
}

fn function_response_part(tool_use_id: &str, tool_name: &Option<String>, content: &[InternalContentBlock]) -> Result<Value, FormatConversionError> {
    Ok(json!({
        "functionResponse": {
            "id": tool_use_id,
            "name": tool_name.clone().unwrap_or_else(|| tool_use_id.to_owned()),
            "response": tool_result_response(content)?,
        }
    }))
}

fn function_call_part(id: &str, name: &str, input: &Value) -> Result<Value, FormatConversionError> {
    let mut call = json!({ "name": name, "args": input });
    if !id.is_empty() {
        call["id"] = Value::String(id.to_owned());
    }
    Ok(json!({ "functionCall": call }))
}

fn file_data_part(media_type: Option<&str>, url: &str) -> Result<Value, FormatConversionError> {
    if media_type.is_none() {
        return Err(FormatConversionError::unsupported_content(
            FORMAT,
            "Gemini fileData requires media_type for URL content",
        ));
    }
    Ok(json!({ "fileData": { "mimeType": media_type, "fileUri": url } }))
}

fn tool_result_response(content: &[InternalContentBlock]) -> Result<Value, FormatConversionError> {
    let text = content
        .iter()
        .map(|block| block.plain_text().map(str::to_owned))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| FormatConversionError::unsupported_content(FORMAT, "Gemini functionResponse requires text-compatible tool result"))?
        .join("");
    serde_json::from_str::<Value>(&text)
        .map(|value| json!({ "result": value }))
        .or_else(|_| Ok(json!({ "result": text })))
}

use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalMessage, InternalRole};

use super::common::{FORMAT, required_array, required_object};

pub(super) fn parse_messages(request: &Value) -> Result<Vec<InternalMessage>, FormatConversionError> {
    let mut messages = Vec::new();
    if let Some(system) = system_instruction_blocks(request)? {
        messages.push(InternalMessage {
            role: InternalRole::System,
            content: system,
        });
    }
    for (index, content) in required_array(request, "contents", "$.contents")?.iter().enumerate() {
        messages.push(content_to_message(content, index)?);
    }
    Ok(messages)
}

pub(super) fn contents_from_internal(messages: &[InternalMessage]) -> Result<Vec<Value>, FormatConversionError> {
    messages
        .iter()
        .filter(|message| message.role != InternalRole::System)
        .map(|message| {
            Ok(json!({
                "role": content_role(&message.role),
                "parts": parts_from_internal(&message.content)?,
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

fn system_instruction_blocks(request: &Value) -> Result<Option<Vec<InternalContentBlock>>, FormatConversionError> {
    let value = request.get("systemInstruction").or_else(|| request.get("system_instruction"));
    match value {
        Some(instruction) => gemini_parts(instruction.get("parts"), "$.systemInstruction.parts").map(Some),
        None => Ok(None),
    }
}

fn content_to_message(content: &Value, index: usize) -> Result<InternalMessage, FormatConversionError> {
    let object = required_object(Some(content), &format!("$.contents[{index}]"))?;
    let role = object.get("role").and_then(Value::as_str).unwrap_or("user");
    Ok(InternalMessage {
        role: gemini_role(role),
        content: gemini_parts(object.get("parts"), &format!("$.contents[{index}].parts"))?,
    })
}

fn gemini_parts(value: Option<&Value>, path: &str) -> Result<Vec<InternalContentBlock>, FormatConversionError> {
    value
        .and_then(Value::as_array)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, path))?
        .iter()
        .enumerate()
        .map(|(index, part)| gemini_part(part, &format!("{path}[{index}]")))
        .collect()
}

fn gemini_part(value: &Value, path: &str) -> Result<InternalContentBlock, FormatConversionError> {
    let object = required_object(Some(value), path)?;
    if let Some(text) = object.get("text").and_then(Value::as_str) {
        return text_part(object, text);
    }
    if let Some(function_call) = object.get("functionCall").or_else(|| object.get("function_call")) {
        return function_call_block(function_call, path);
    }
    if let Some(function_response) = object.get("functionResponse").or_else(|| object.get("function_response")) {
        return function_response_block(function_response, path);
    }
    if let Some(inline_data) = object.get("inlineData").or_else(|| object.get("inline_data")) {
        return inline_data_block(inline_data, path);
    }
    if let Some(file_data) = object.get("fileData").or_else(|| object.get("file_data")) {
        return file_data_block(file_data, path);
    }
    Err(FormatConversionError::unsupported_content(FORMAT, format!("{path}: unsupported part")))
}

fn text_part(object: &Map<String, Value>, text: &str) -> Result<InternalContentBlock, FormatConversionError> {
    if object.get("thought").and_then(Value::as_bool) == Some(true) {
        return Ok(InternalContentBlock::Thinking {
            text: text.to_owned(),
            signature: object.get("thoughtSignature").and_then(Value::as_str).map(str::to_owned),
        });
    }
    Ok(InternalContentBlock::Text(text.to_owned()))
}

fn function_call_block(value: &Value, path: &str) -> Result<InternalContentBlock, FormatConversionError> {
    let object = required_object(Some(value), &format!("{path}.functionCall"))?;
    let name = required_text(object, path, "name")?.to_owned();
    Ok(InternalContentBlock::ToolUse {
        id: format!("gemini_call_{name}"),
        name,
        input: object.get("args").cloned().unwrap_or_else(|| json!({})),
    })
}

fn function_response_block(value: &Value, path: &str) -> Result<InternalContentBlock, FormatConversionError> {
    let object = required_object(Some(value), &format!("{path}.functionResponse"))?;
    let name = required_text(object, path, "name")?.to_owned();
    Ok(InternalContentBlock::ToolResult {
        tool_use_id: format!("gemini_call_{name}"),
        tool_name: Some(name),
        content: vec![InternalContentBlock::Text(
            object.get("response").cloned().unwrap_or_else(|| json!({})).to_string(),
        )],
        is_error: false,
    })
}

fn inline_data_block(value: &Value, path: &str) -> Result<InternalContentBlock, FormatConversionError> {
    let object = required_object(Some(value), &format!("{path}.inlineData"))?;
    let media_type = optional_mime_type(object);
    let data = object
        .get("data")
        .and_then(Value::as_str)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("{path}.inlineData.data")))?
        .to_owned();
    if media_type.as_deref().is_some_and(|value| value.starts_with("audio/")) {
        return Ok(InternalContentBlock::Audio {
            data,
            format: None,
            media_type,
        });
    }
    Ok(InternalContentBlock::Image {
        url: None,
        data: Some(data),
        media_type,
    })
}

fn file_data_block(value: &Value, path: &str) -> Result<InternalContentBlock, FormatConversionError> {
    let object = required_object(Some(value), &format!("{path}.fileData"))?;
    Ok(InternalContentBlock::File {
        file_id: None,
        file_url: object
            .get("fileUri")
            .or_else(|| object.get("file_uri"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        data: None,
        media_type: optional_mime_type(object),
        filename: None,
    })
}

fn parts_from_internal(blocks: &[InternalContentBlock]) -> Result<Vec<Value>, FormatConversionError> {
    blocks.iter().map(part_from_internal).collect()
}

fn part_from_internal(block: &InternalContentBlock) -> Result<Value, FormatConversionError> {
    match block {
        InternalContentBlock::Text(text) => Ok(json!({ "text": text })),
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
        InternalContentBlock::ToolUse { name, input, .. } => Ok(json!({ "functionCall": { "name": name, "args": input } })),
        InternalContentBlock::ToolResult { tool_name, content, .. } => Ok(json!({
            "functionResponse": {
                "name": tool_name.clone().unwrap_or_default(),
                "response": tool_result_response(content)?,
            }
        })),
        InternalContentBlock::File {
            data: Some(data), media_type, ..
        } => Ok(json!({ "inlineData": { "mimeType": media_type, "data": data } })),
        InternalContentBlock::Image { .. } | InternalContentBlock::File { .. } => Err(FormatConversionError::unsupported_content(
            FORMAT,
            "content block cannot be represented in Gemini",
        )),
    }
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
    serde_json::from_str(&text).or_else(|_| Ok(json!({ "content": text })))
}

fn optional_mime_type(object: &Map<String, Value>) -> Option<String> {
    object
        .get("mimeType")
        .or_else(|| object.get("mime_type"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn gemini_role(value: &str) -> InternalRole {
    if value == "model" { InternalRole::Assistant } else { InternalRole::User }
}

fn content_role(role: &InternalRole) -> &'static str {
    match role {
        InternalRole::Assistant => "model",
        _ => "user",
    }
}

fn required_text<'a>(object: &'a Map<String, Value>, path: &str, key: &str) -> Result<&'a str, FormatConversionError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("{path}.{key}")))
}

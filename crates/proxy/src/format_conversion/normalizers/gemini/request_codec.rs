use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalMessage, InternalRole};

use super::common::{FORMAT, required_array, required_object};
use super::content_compaction::compact_contents;
pub(super) use super::request_parts::parts_from_internal;
use super::request_tool_calls::GeminiToolCallTracker;

const DUMMY_THINKING_TEXT: &str = "Thinking...";

pub(super) fn parse_messages(request: &Value) -> Result<Vec<InternalMessage>, FormatConversionError> {
    let mut messages = Vec::new();
    let mut tool_calls = GeminiToolCallTracker::default();
    if let Some(system) = system_instruction_blocks(request)? {
        messages.push(InternalMessage {
            role: InternalRole::System,
            content: system,
        });
    }
    for (index, content) in required_array(request, "contents", "$.contents")?.iter().enumerate() {
        messages.push(content_to_message(content, index, &mut tool_calls)?);
    }
    Ok(messages)
}

pub(super) fn contents_from_internal(messages: &[InternalMessage]) -> Result<Vec<Value>, FormatConversionError> {
    let contents = messages
        .iter()
        .filter(|message| !matches!(message.role, InternalRole::System | InternalRole::Developer))
        .map(|message| {
            Ok(json!({
                "role": content_role(&message.role),
                "parts": parts_from_internal(&message.content)?,
            }))
        })
        .collect::<Result<Vec<_>, FormatConversionError>>()?;
    Ok(compact_contents(contents))
}

pub(super) fn joined_system(messages: &[InternalMessage]) -> Result<Option<String>, FormatConversionError> {
    let system = messages
        .iter()
        .filter(|message| matches!(message.role, InternalRole::System | InternalRole::Developer))
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
        Some(instruction) => parts_from_gemini(instruction.get("parts"), "$.systemInstruction.parts").map(Some),
        None => Ok(None),
    }
}

fn content_to_message(content: &Value, index: usize, tool_calls: &mut GeminiToolCallTracker) -> Result<InternalMessage, FormatConversionError> {
    let object = required_object(Some(content), &format!("$.contents[{index}]"))?;
    let role = object.get("role").and_then(Value::as_str).unwrap_or("user");
    Ok(InternalMessage {
        role: gemini_role(role),
        content: parts_from_gemini_with_tracker(object.get("parts"), &format!("$.contents[{index}].parts"), tool_calls)?,
    })
}

pub(super) fn parts_from_gemini(value: Option<&Value>, path: &str) -> Result<Vec<InternalContentBlock>, FormatConversionError> {
    let mut tool_calls = GeminiToolCallTracker::default();
    parts_from_gemini_with_tracker(value, path, &mut tool_calls)
}

fn parts_from_gemini_with_tracker(
    value: Option<&Value>,
    path: &str,
    tool_calls: &mut GeminiToolCallTracker,
) -> Result<Vec<InternalContentBlock>, FormatConversionError> {
    value
        .and_then(Value::as_array)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, path))?
        .iter()
        .enumerate()
        .try_fold(Vec::new(), |mut blocks, (index, part)| {
            let part_path = format!("{path}[{index}]");
            insert_thinking_from_part_signature(part, &mut blocks, &part_path)?;
            blocks.push(gemini_part(part, &part_path, tool_calls)?);
            Ok(blocks)
        })
}

fn gemini_part(value: &Value, path: &str, tool_calls: &mut GeminiToolCallTracker) -> Result<InternalContentBlock, FormatConversionError> {
    let object = required_object(Some(value), path)?;
    if let Some(text) = object.get("text").and_then(Value::as_str) {
        return text_part(object, text);
    }
    if let Some(function_call) = object.get("functionCall").or_else(|| object.get("function_call")) {
        return function_call_block(function_call, path, tool_calls);
    }
    if let Some(function_response) = object.get("functionResponse").or_else(|| object.get("function_response")) {
        return function_response_block(function_response, path, tool_calls);
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
    Ok(InternalContentBlock::text(text.to_owned()))
}

fn insert_thinking_from_part_signature(part: &Value, previous_blocks: &mut Vec<InternalContentBlock>, path: &str) -> Result<(), FormatConversionError> {
    let object = required_object(Some(part), path)?;
    if object.get("thought").and_then(Value::as_bool) == Some(true) {
        return Ok(());
    }
    let Some(signature) = object.get("thoughtSignature").and_then(Value::as_str).filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    let restored = previous_blocks.iter().enumerate().rev().find_map(|(index, block)| match block {
        InternalContentBlock::Text { text, .. } if !text.is_empty() => Some((index, text.clone())),
        _ => None,
    });
    let (index, text) = restored.unwrap_or((0, DUMMY_THINKING_TEXT.to_owned()));
    previous_blocks.insert(
        index,
        InternalContentBlock::Thinking {
            text,
            signature: Some(signature.to_owned()),
        },
    );
    Ok(())
}

fn function_call_block(value: &Value, path: &str, tool_calls: &mut GeminiToolCallTracker) -> Result<InternalContentBlock, FormatConversionError> {
    let object = required_object(Some(value), &format!("{path}.functionCall"))?;
    let name = required_text(object, path, "name")?.to_owned();
    let id = object
        .get("id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| tool_calls.synthetic_id(&name));
    tool_calls.push(name.clone(), id.clone());
    Ok(InternalContentBlock::ToolUse {
        id,
        name,
        input: object.get("args").filter(|value| value.is_object()).cloned().unwrap_or_else(|| json!({})),
    })
}

fn function_response_block(value: &Value, path: &str, tool_calls: &mut GeminiToolCallTracker) -> Result<InternalContentBlock, FormatConversionError> {
    let object = required_object(Some(value), &format!("{path}.functionResponse"))?;
    let name = required_text(object, path, "name")?.to_owned();
    let tool_use_id = object
        .get("id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| tool_calls.pop(&name));
    Ok(InternalContentBlock::ToolResult {
        tool_use_id,
        tool_name: Some(name),
        content: gemini_function_response_content(object.get("response")),
        is_error: false,
    })
}

fn gemini_function_response_content(value: Option<&Value>) -> Vec<InternalContentBlock> {
    let response = value.cloned().unwrap_or_else(|| json!({}));
    let result = response.as_object().and_then(|object| object.get("result")).cloned().unwrap_or(response);
    match result {
        Value::String(text) => vec![InternalContentBlock::text(text)],
        value => vec![InternalContentBlock::text(value.to_string())],
    }
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

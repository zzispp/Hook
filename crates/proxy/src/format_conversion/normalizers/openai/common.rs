use serde_json::{Map, Value};

use crate::format_conversion::{FormatConversionError, InternalUsage, StopReason};

pub const FORMAT: &str = "openai";

pub fn ensure_tools_disabled(request: &Value) -> Result<(), FormatConversionError> {
    if request.get("tools").is_some() {
        return Err(FormatConversionError::unsupported_feature(FORMAT, "tools"));
    }
    if request.get("tool_choice").is_some() {
        return Err(FormatConversionError::unsupported_feature(FORMAT, "tool_choice"));
    }
    Ok(())
}

pub fn parse_content(value: Option<&Value>, path: &str) -> Result<String, FormatConversionError> {
    match value {
        Some(Value::String(text)) => Ok(text.to_owned()),
        Some(Value::Array(blocks)) => parse_text_blocks(blocks, path),
        Some(_) => Err(FormatConversionError::invalid_payload(FORMAT, path)),
        None => Ok(String::new()),
    }
}

pub fn first_choice<'a>(value: &'a Value, path: &str) -> Result<&'a Map<String, Value>, FormatConversionError> {
    let choices = required_array(value, "choices", path)?;
    let first = choices
        .first()
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("{path}[0]")))?;
    required_object(Some(first), &format!("{path}[0]"))
}

pub fn usage_from_openai(value: Option<&Value>) -> Option<InternalUsage> {
    let object = value?.as_object()?;
    Some(
        InternalUsage {
            prompt_tokens: object.get("prompt_tokens").and_then(as_u32),
            completion_tokens: object.get("completion_tokens").and_then(as_u32),
            total_tokens: object.get("total_tokens").and_then(as_u32),
        }
        .with_total(),
    )
}

pub fn map_openai_stop_reason(value: &str) -> StopReason {
    match value {
        "stop" => StopReason::EndTurn,
        "length" => StopReason::MaxTokens,
        "tool_calls" | "function_call" => StopReason::ToolUse,
        "content_filter" => StopReason::ContentFiltered,
        _ => StopReason::Unknown,
    }
}

pub fn openai_finish_reason(reason: &StopReason) -> &'static str {
    match reason {
        StopReason::EndTurn => "stop",
        StopReason::MaxTokens => "length",
        StopReason::StopSequence => "stop",
        StopReason::ToolUse => "tool_calls",
        StopReason::ContentFiltered => "content_filter",
        StopReason::Unknown => "stop",
    }
}

pub fn required_object<'a>(value: Option<&'a Value>, path: &str) -> Result<&'a Map<String, Value>, FormatConversionError> {
    value
        .and_then(Value::as_object)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, path.to_owned()))
}

pub fn required_array<'a>(value: &'a Value, key: &str, path: &str) -> Result<&'a [Value], FormatConversionError> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, path.to_owned()))
}

pub fn required_string(value: &Value, key: &str, path: &str) -> Result<String, FormatConversionError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, path.to_owned()))
}

pub fn optional_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_owned)
}

pub fn optional_string_value(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(str::to_owned)
}

pub fn optional_f64(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

pub fn optional_u32(value: &Value, key: &str) -> Option<u32> {
    value.get(key).and_then(as_u32)
}

pub fn optional_bool(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

pub fn as_u32(value: &Value) -> Option<u32> {
    value.as_u64().and_then(|item| u32::try_from(item).ok())
}

pub fn insert_optional_number(map: &mut Map<String, Value>, key: &str, value: Option<f64>) {
    if let Some(number) = value.and_then(serde_json::Number::from_f64) {
        map.insert(key.to_owned(), Value::Number(number));
    }
}

pub fn insert_optional_integer(map: &mut Map<String, Value>, key: &str, value: Option<u32>) {
    if let Some(number) = value {
        map.insert(key.to_owned(), Value::Number(serde_json::Number::from(number)));
    }
}

fn parse_text_blocks(blocks: &[Value], path: &str) -> Result<String, FormatConversionError> {
    let mut result = String::new();
    for block in blocks {
        let object = required_object(Some(block), path)?;
        let block_type = optional_string_value(object.get("type")).unwrap_or_default();
        if block_type != "text" {
            return Err(FormatConversionError::unsupported_content(FORMAT, format!("{path}: non-text block")));
        }
        result.push_str(required_block_text(object, path)?);
    }
    Ok(result)
}

fn required_block_text<'a>(object: &'a Map<String, Value>, path: &str) -> Result<&'a str, FormatConversionError> {
    object
        .get("text")
        .and_then(Value::as_str)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("{path}.text")))
}

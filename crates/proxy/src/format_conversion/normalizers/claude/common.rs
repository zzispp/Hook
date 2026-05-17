use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalUsage, StopReason};

pub const FORMAT: &str = "claude";

pub fn content_text(value: Option<&Value>, path: &str) -> Result<String, FormatConversionError> {
    match value {
        Some(Value::String(text)) => Ok(text.to_owned()),
        Some(Value::Array(blocks)) => text_blocks(blocks, path),
        Some(_) => Err(FormatConversionError::invalid_payload(FORMAT, path)),
        None => Ok(String::new()),
    }
}

pub fn response_content(text: &str) -> Vec<Value> {
    if text.is_empty() {
        Vec::new()
    } else {
        vec![json!({ "type": "text", "text": text })]
    }
}

pub fn usage_from_claude(value: Option<&Value>) -> Option<InternalUsage> {
    let object = value?.as_object()?;
    Some(
        InternalUsage {
            prompt_tokens: object.get("input_tokens").and_then(as_u32),
            completion_tokens: object.get("output_tokens").and_then(as_u32),
            total_tokens: None,
        }
        .with_total(),
    )
}

pub fn claude_usage(usage: &InternalUsage) -> Value {
    let complete = usage.clone().with_total();
    json!({
        "input_tokens": complete.prompt_tokens,
        "output_tokens": complete.completion_tokens,
    })
}

pub fn map_claude_stop_reason(value: &str) -> StopReason {
    match value {
        "end_turn" => StopReason::EndTurn,
        "max_tokens" => StopReason::MaxTokens,
        "stop_sequence" => StopReason::StopSequence,
        "tool_use" => StopReason::ToolUse,
        "content_filtered" | "refusal" => StopReason::ContentFiltered,
        _ => StopReason::Unknown,
    }
}

pub fn claude_stop_reason(reason: &StopReason) -> &'static str {
    match reason {
        StopReason::MaxTokens => "max_tokens",
        StopReason::StopSequence => "stop_sequence",
        StopReason::ToolUse => "tool_use",
        StopReason::ContentFiltered => "content_filtered",
        _ => "end_turn",
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

pub fn optional_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_owned)
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

fn text_blocks(blocks: &[Value], path: &str) -> Result<String, FormatConversionError> {
    let mut text = String::new();
    for block in blocks {
        let object = required_object(Some(block), path)?;
        if object.get("type").and_then(Value::as_str) != Some("text") {
            return Err(FormatConversionError::unsupported_content(FORMAT, format!("{path}: non-text block")));
        }
        let value = object
            .get("text")
            .and_then(Value::as_str)
            .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("{path}.text")))?;
        text.push_str(value);
    }
    Ok(text)
}

fn as_u32(value: &Value) -> Option<u32> {
    value.as_u64().and_then(|item| u32::try_from(item).ok())
}

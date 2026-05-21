use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, GeminiToolStreamItem, InternalUsage, StopReason};

pub const FORMAT: &str = "gemini";

pub fn generation_config(value: &Value) -> Option<&Map<String, Value>> {
    value
        .get("generationConfig")
        .or_else(|| value.get("generation_config"))
        .and_then(Value::as_object)
}

pub fn generation_config_value<'a>(config: &'a Map<String, Value>, camel: &str, snake: &str) -> Option<&'a Value> {
    config.get(camel).or_else(|| config.get(snake))
}

pub fn parts_text(value: Option<&Value>, path: &str) -> Result<String, FormatConversionError> {
    let parts = value
        .and_then(Value::as_array)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, path))?;
    let mut output = String::new();
    for part in parts {
        let object = required_object(Some(part), path)?;
        if object.keys().any(|key| key != "text") {
            return Err(FormatConversionError::unsupported_content(FORMAT, format!("{path}: non-text part")));
        }
        let text = object
            .get("text")
            .and_then(Value::as_str)
            .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("{path}.text")))?;
        output.push_str(text);
    }
    Ok(output)
}

pub fn content_chunk(text: &str, model: &str, finish_reason: Option<&StopReason>, usage: Option<&InternalUsage>) -> Value {
    let mut candidate = json!({
        "index": 0,
        "content": {
            "role": "model",
            "parts": text_parts(text),
        },
    });
    if let Some(reason) = finish_reason {
        candidate["finishReason"] = Value::String(gemini_finish_reason(reason).to_owned());
    }
    let mut payload = json!({
        "modelVersion": model,
        "candidates": [candidate],
    });
    if let Some(usage_value) = usage_json(usage) {
        payload["usageMetadata"] = usage_value;
    }
    payload
}

pub fn terminal_chunk(model: &str, finish_reason: Option<&StopReason>, usage: Option<&InternalUsage>) -> Value {
    let mut candidate = json!({ "index": 0 });
    if let Some(reason) = finish_reason {
        candidate["finishReason"] = Value::String(gemini_finish_reason(reason).to_owned());
    }
    let mut payload = json!({
        "modelVersion": model,
        "candidates": [candidate],
    });
    if let Some(usage_value) = usage_json(usage) {
        payload["usageMetadata"] = usage_value;
    }
    payload
}

pub fn thought_chunk(text: &str, signature: Option<&str>, model: &str) -> Value {
    let mut part = json!({
        "text": text,
        "thought": true,
    });
    if let Some(signature) = signature.filter(|value| !value.is_empty()) {
        part["thoughtSignature"] = Value::String(signature.to_owned());
    }
    json!({
        "modelVersion": model,
        "candidates": [{
            "index": 0,
            "content": {
                "role": "model",
                "parts": [part],
            },
        }],
    })
}

pub fn complete_function_call_chunk(tool: &GeminiToolStreamItem, model: &str) -> Value {
    let mut function_call = json!({
        "name": tool.name,
        "args": parse_arguments(&tool.arguments),
    });
    if !tool.id.is_empty() {
        function_call["id"] = Value::String(tool.id.clone());
    }
    json!({
        "modelVersion": model,
        "candidates": [{
            "index": 0,
            "content": {
                "role": "model",
                "parts": [{ "functionCall": function_call }],
            },
        }],
    })
}

pub fn usage_from_gemini(value: Option<&Value>) -> Option<InternalUsage> {
    let object = value?.as_object()?;
    let candidates = object.get("candidatesTokenCount").and_then(as_u32);
    let thoughts = object.get("thoughtsTokenCount").and_then(as_u32);
    let completion_tokens = match (candidates, thoughts) {
        (Some(candidates), Some(thoughts)) => Some(candidates.saturating_add(thoughts)),
        (Some(candidates), None) => Some(candidates),
        (None, Some(thoughts)) => Some(thoughts),
        (None, None) => None,
    };
    Some(
        InternalUsage {
            prompt_tokens: object.get("promptTokenCount").and_then(as_u32),
            completion_tokens,
            total_tokens: object.get("totalTokenCount").and_then(as_u32),
            cache_read_tokens: object.get("cachedContentTokenCount").and_then(as_u32),
            cache_creation_tokens: None,
            reasoning_tokens: thoughts,
        }
        .with_total(),
    )
}

pub fn map_gemini_stop_reason(value: &str) -> StopReason {
    match value {
        "STOP" => StopReason::EndTurn,
        "MAX_TOKENS" => StopReason::MaxTokens,
        "SAFETY" | "RECITATION" | "LANGUAGE" | "BLOCKLIST" | "PROHIBITED_CONTENT" | "SPII" | "IMAGE_SAFETY" => StopReason::ContentFiltered,
        "MALFORMED_FUNCTION_CALL" => StopReason::ToolUse,
        _ => StopReason::Unknown,
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

pub fn optional_bool(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

pub fn optional_f64_from_config(config: Option<&Map<String, Value>>, camel: &str, snake: &str) -> Option<f64> {
    generation_config_value(config?, camel, snake).and_then(Value::as_f64)
}

pub fn optional_u32_from_config(config: Option<&Map<String, Value>>, camel: &str, snake: &str) -> Option<u32> {
    generation_config_value(config?, camel, snake).and_then(as_u32)
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

pub fn as_u32(value: &Value) -> Option<u32> {
    value.as_u64().and_then(|item| u32::try_from(item).ok())
}

fn text_parts(text: &str) -> Vec<Value> {
    if text.is_empty() { Vec::new() } else { vec![json!({ "text": text })] }
}

fn gemini_finish_reason(reason: &StopReason) -> &'static str {
    match reason {
        StopReason::MaxTokens => "MAX_TOKENS",
        StopReason::ContentFiltered => "SAFETY",
        StopReason::ToolUse => "STOP",
        _ => "STOP",
    }
}

fn usage_json(usage: Option<&InternalUsage>) -> Option<Value> {
    let complete = usage.cloned()?.with_total();
    let mut metadata = Map::new();
    insert_optional_usage(&mut metadata, "promptTokenCount", complete.prompt_tokens);
    insert_optional_usage(&mut metadata, "candidatesTokenCount", complete.completion_tokens);
    insert_optional_usage(&mut metadata, "totalTokenCount", complete.total_tokens);
    insert_optional_usage(&mut metadata, "cachedContentTokenCount", complete.cache_read_tokens);
    insert_optional_usage(&mut metadata, "thoughtsTokenCount", complete.reasoning_tokens);
    (!metadata.is_empty()).then_some(Value::Object(metadata))
}

fn insert_optional_usage(metadata: &mut Map<String, Value>, key: &str, value: Option<u32>) {
    if let Some(value) = value {
        metadata.insert(key.to_owned(), Value::Number(serde_json::Number::from(value)));
    }
}

fn parse_arguments(arguments: &str) -> Value {
    if arguments.is_empty() {
        return json!({});
    }
    serde_json::from_str(arguments).unwrap_or_else(|_| json!({}))
}

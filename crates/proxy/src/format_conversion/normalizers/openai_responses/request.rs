use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalMessage, InternalRequest, InternalRole};

const FORMAT: &str = "openai_responses";

pub fn to_internal(request: &Value) -> Result<InternalRequest, FormatConversionError> {
    ensure_tools_disabled(request)?;
    Ok(InternalRequest {
        model: string_field(request, "model", "$.model")?,
        messages: input_messages(request.get("input"))?,
        temperature: number_field(request, "temperature"),
        max_tokens: u32_field(request, "max_output_tokens").or(u32_field(request, "max_tokens")),
        stream: bool_field(request, "stream").unwrap_or(false),
    })
}

pub fn from_internal(internal: &InternalRequest) -> Result<Value, FormatConversionError> {
    let mut output = Map::new();
    output.insert("model".into(), Value::String(internal.model.clone()));
    output.insert("input".into(), Value::Array(messages_from_internal(&internal.messages)));
    insert_number(&mut output, "temperature", internal.temperature);
    insert_integer(&mut output, "max_output_tokens", internal.max_tokens);
    if internal.stream {
        output.insert("stream".into(), Value::Bool(true));
    }
    Ok(Value::Object(output))
}

fn ensure_tools_disabled(request: &Value) -> Result<(), FormatConversionError> {
    if request.get("tools").is_some() {
        return Err(FormatConversionError::unsupported_feature(FORMAT, "tools"));
    }
    if request.get("tool_choice").is_some() {
        return Err(FormatConversionError::unsupported_feature(FORMAT, "tool_choice"));
    }
    Ok(())
}

fn input_messages(value: Option<&Value>) -> Result<Vec<InternalMessage>, FormatConversionError> {
    match value {
        Some(Value::String(text)) => Ok(vec![message(InternalRole::User, text.clone())]),
        Some(Value::Array(items)) => items.iter().enumerate().map(|(index, value)| input_item(value, index)).collect(),
        Some(_) | None => Err(FormatConversionError::invalid_payload(FORMAT, "$.input")),
    }
}

fn input_item(value: &Value, index: usize) -> Result<InternalMessage, FormatConversionError> {
    let object = value
        .as_object()
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("$.input[{index}]")))?;
    let role = object.get("role").and_then(Value::as_str).unwrap_or("user");
    let content = content_text(object.get("content"), &format!("$.input[{index}].content"))?;
    Ok(message(role_from_str(role), content))
}

fn messages_from_internal(messages: &[InternalMessage]) -> Vec<Value> {
    messages
        .iter()
        .map(|item| {
            json!({
                "role": role_as_str(&item.role),
                "content": item.text,
            })
        })
        .collect()
}

fn content_text(value: Option<&Value>, path: &str) -> Result<String, FormatConversionError> {
    match value {
        Some(Value::String(text)) => Ok(text.clone()),
        Some(Value::Array(items)) => content_blocks_text(items, path),
        Some(_) | None => Err(FormatConversionError::invalid_payload(FORMAT, path)),
    }
}

fn content_blocks_text(items: &[Value], path: &str) -> Result<String, FormatConversionError> {
    let mut text = String::new();
    for item in items {
        let object = item.as_object().ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, path))?;
        let block_type = object.get("type").and_then(Value::as_str).unwrap_or_default();
        if !matches!(block_type, "input_text" | "output_text" | "text") {
            return Err(FormatConversionError::unsupported_content(FORMAT, format!("{path}: non-text block")));
        }
        text.push_str(
            object
                .get("text")
                .and_then(Value::as_str)
                .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("{path}.text")))?,
        );
    }
    Ok(text)
}

fn message(role: InternalRole, text: String) -> InternalMessage {
    InternalMessage { role, text }
}

fn role_from_str(value: &str) -> InternalRole {
    match value {
        "assistant" => InternalRole::Assistant,
        "system" | "developer" => InternalRole::System,
        _ => InternalRole::User,
    }
}

fn role_as_str(role: &InternalRole) -> &'static str {
    match role {
        InternalRole::System => "system",
        InternalRole::User => "user",
        InternalRole::Assistant => "assistant",
    }
}

fn string_field(value: &Value, key: &str, path: &str) -> Result<String, FormatConversionError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, path))
}

fn number_field(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

fn u32_field(value: &Value, key: &str) -> Option<u32> {
    value.get(key).and_then(Value::as_u64).and_then(|item| u32::try_from(item).ok())
}

fn bool_field(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

fn insert_number(map: &mut Map<String, Value>, key: &str, value: Option<f64>) {
    if let Some(number) = value.and_then(serde_json::Number::from_f64) {
        map.insert(key.to_owned(), Value::Number(number));
    }
}

fn insert_integer(map: &mut Map<String, Value>, key: &str, value: Option<u32>) {
    if let Some(number) = value {
        map.insert(key.to_owned(), Value::Number(serde_json::Number::from(number)));
    }
}

use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalMessage, InternalRequest, InternalRole};

use super::common::{
    content_text, ensure_tools_disabled, insert_optional_integer, insert_optional_number, optional_bool, optional_f64, optional_string, optional_u32,
    required_array, required_object,
};

pub fn to_internal(request: &Value) -> Result<InternalRequest, FormatConversionError> {
    ensure_tools_disabled(request)?;
    let mut messages = system_messages(request)?;
    messages.extend(parse_messages(request)?);
    Ok(InternalRequest {
        model: optional_string(request, "model").unwrap_or_default(),
        messages,
        temperature: optional_f64(request, "temperature"),
        max_tokens: optional_u32(request, "max_tokens"),
        stream: optional_bool(request, "stream").unwrap_or(false),
    })
}

pub fn from_internal(internal: &InternalRequest) -> Result<Value, FormatConversionError> {
    let mut output = Map::new();
    output.insert("model".into(), Value::String(internal.model.clone()));
    output.insert("messages".into(), Value::Array(messages_from_internal(&internal.messages)));
    if let Some(system) = joined_system(&internal.messages) {
        output.insert("system".into(), Value::String(system));
    }
    insert_optional_integer(&mut output, "max_tokens", internal.max_tokens);
    insert_optional_number(&mut output, "temperature", internal.temperature);
    if internal.stream {
        output.insert("stream".into(), Value::Bool(true));
    }
    Ok(Value::Object(output))
}

fn system_messages(request: &Value) -> Result<Vec<InternalMessage>, FormatConversionError> {
    match request.get("system") {
        Some(Value::String(text)) if !text.is_empty() => Ok(vec![system_message(text)]),
        Some(Value::Array(_)) => Ok(vec![system_message(&content_text(request.get("system"), "$.system")?)]),
        Some(_) => Err(FormatConversionError::invalid_payload("claude", "$.system")),
        None => Ok(Vec::new()),
    }
}

fn system_message(text: &str) -> InternalMessage {
    InternalMessage {
        role: InternalRole::System,
        text: text.to_owned(),
    }
}

fn parse_messages(request: &Value) -> Result<Vec<InternalMessage>, FormatConversionError> {
    let source = required_array(request, "messages", "$.messages")?;
    let mut messages = Vec::with_capacity(source.len());
    for (index, value) in source.iter().enumerate() {
        messages.push(parse_message(value, index)?);
    }
    Ok(messages)
}

fn parse_message(value: &Value, index: usize) -> Result<InternalMessage, FormatConversionError> {
    let object = required_object(Some(value), &format!("$.messages[{index}]"))?;
    let role = object.get("role").and_then(Value::as_str).unwrap_or("user");
    Ok(InternalMessage {
        role: claude_role(role),
        text: content_text(object.get("content"), &format!("$.messages[{index}].content"))?,
    })
}

fn messages_from_internal(messages: &[InternalMessage]) -> Vec<Value> {
    messages
        .iter()
        .filter(|message| message.role != InternalRole::System)
        .map(|message| {
            json!({
                "role": message_role(&message.role),
                "content": message.text,
            })
        })
        .collect()
}

fn joined_system(messages: &[InternalMessage]) -> Option<String> {
    let system: Vec<&str> = messages
        .iter()
        .filter(|message| message.role == InternalRole::System && !message.text.is_empty())
        .map(|message| message.text.as_str())
        .collect();
    (!system.is_empty()).then(|| system.join("\n\n"))
}

fn claude_role(value: &str) -> InternalRole {
    if value == "assistant" { InternalRole::Assistant } else { InternalRole::User }
}

fn message_role(role: &InternalRole) -> &'static str {
    match role {
        InternalRole::Assistant => "assistant",
        _ => "user",
    }
}

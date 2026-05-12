use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalMessage, InternalRequest, InternalRole};

use super::common::{
    ensure_tools_disabled, generation_config, insert_optional_integer, insert_optional_number, optional_bool, optional_f64_from_config, optional_string,
    optional_u32_from_config, parts_text, required_array, required_object, system_instruction_text,
};

pub fn to_internal(request: &Value) -> Result<InternalRequest, FormatConversionError> {
    ensure_tools_disabled(request)?;
    let config = generation_config(request);
    Ok(InternalRequest {
        model: optional_string(request, "model").unwrap_or_default(),
        messages: parse_messages(request)?,
        temperature: optional_f64_from_config(config, "temperature", "temperature"),
        max_tokens: optional_u32_from_config(config, "maxOutputTokens", "max_output_tokens"),
        stream: optional_bool(request, "stream").unwrap_or(false),
    })
}

pub fn from_internal(internal: &InternalRequest) -> Result<Value, FormatConversionError> {
    let mut output = Map::new();
    output.insert("model".into(), Value::String(internal.model.clone()));
    output.insert("contents".into(), Value::Array(contents_from_internal(&internal.messages)));
    if let Some(system) = joined_system(&internal.messages) {
        output.insert("system_instruction".into(), json!({ "parts": [{ "text": system }] }));
    }
    let config = generation_config_from_internal(internal);
    if !config.is_empty() {
        output.insert("generation_config".into(), Value::Object(config));
    }
    Ok(Value::Object(output))
}

fn parse_messages(request: &Value) -> Result<Vec<InternalMessage>, FormatConversionError> {
    let mut messages = Vec::new();
    if let Some(system) = system_instruction_text(request)? {
        messages.push(InternalMessage {
            role: InternalRole::System,
            text: system,
        });
    }
    for (index, content) in required_array(request, "contents", "$.contents")?.iter().enumerate() {
        messages.push(content_to_message(content, index)?);
    }
    Ok(messages)
}

fn content_to_message(content: &Value, index: usize) -> Result<InternalMessage, FormatConversionError> {
    let object = required_object(Some(content), &format!("$.contents[{index}]"))?;
    let role = object.get("role").and_then(Value::as_str).unwrap_or("user");
    Ok(InternalMessage {
        role: gemini_role(role),
        text: parts_text(object.get("parts"), &format!("$.contents[{index}].parts"))?,
    })
}

fn contents_from_internal(messages: &[InternalMessage]) -> Vec<Value> {
    messages
        .iter()
        .filter(|message| message.role != InternalRole::System)
        .map(|message| {
            json!({
                "role": content_role(&message.role),
                "parts": [{ "text": message.text }],
            })
        })
        .collect()
}

fn joined_system(messages: &[InternalMessage]) -> Option<String> {
    let text: Vec<&str> = messages
        .iter()
        .filter(|message| message.role == InternalRole::System && !message.text.is_empty())
        .map(|message| message.text.as_str())
        .collect();
    (!text.is_empty()).then(|| text.join("\n\n"))
}

fn generation_config_from_internal(internal: &InternalRequest) -> Map<String, Value> {
    let mut config = Map::new();
    insert_optional_integer(&mut config, "max_output_tokens", internal.max_tokens);
    insert_optional_number(&mut config, "temperature", internal.temperature);
    config
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

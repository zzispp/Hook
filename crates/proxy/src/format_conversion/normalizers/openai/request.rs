use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalMessage, InternalRequest, InternalRole};

use super::common::{
    FORMAT, ensure_tools_disabled, insert_optional_integer, insert_optional_number, optional_bool, optional_f64, optional_u32, parse_content, required_array,
    required_object, required_string,
};

pub fn to_internal(request: &Value) -> Result<InternalRequest, FormatConversionError> {
    ensure_tools_disabled(request)?;
    Ok(InternalRequest {
        model: required_string(request, "model", "$.model")?,
        messages: parse_request_messages(request)?,
        temperature: optional_f64(request, "temperature"),
        max_tokens: optional_u32(request, "max_completion_tokens").or(optional_u32(request, "max_tokens")),
        stream: optional_bool(request, "stream").unwrap_or(false),
    })
}

pub fn from_internal(internal: &InternalRequest) -> Result<Value, FormatConversionError> {
    let mut output = Map::new();
    output.insert("model".into(), Value::String(internal.model.clone()));
    output.insert("messages".into(), Value::Array(request_messages_from_internal(&internal.messages)));
    insert_optional_number(&mut output, "temperature", internal.temperature);
    insert_optional_integer(&mut output, "max_tokens", internal.max_tokens);
    if internal.stream {
        output.insert("stream".into(), Value::Bool(true));
        output.insert("stream_options".into(), json!({ "include_usage": true }));
    }
    Ok(Value::Object(output))
}

fn parse_request_messages(request: &Value) -> Result<Vec<InternalMessage>, FormatConversionError> {
    let source = required_array(request, "messages", "$.messages")?;
    let mut messages = Vec::with_capacity(source.len());
    for (index, value) in source.iter().enumerate() {
        messages.push(parse_request_message(value, index)?);
    }
    Ok(messages)
}

fn parse_request_message(value: &Value, index: usize) -> Result<InternalMessage, FormatConversionError> {
    let object = required_object(Some(value), "$.messages[]")?;
    if object.get("tool_calls").is_some() {
        return Err(FormatConversionError::unsupported_feature(FORMAT, "tool_calls"));
    }
    let role = required_string(value, "role", &format!("$.messages[{index}].role"))?;
    Ok(InternalMessage {
        role: map_openai_role(&role)?,
        text: parse_content(object.get("content"), &format!("$.messages[{index}].content"))?,
    })
}

fn request_messages_from_internal(messages: &[InternalMessage]) -> Vec<Value> {
    messages
        .iter()
        .map(|message| {
            json!({
                "role": openai_role(&message.role),
                "content": message.text,
            })
        })
        .collect()
}

fn map_openai_role(value: &str) -> Result<InternalRole, FormatConversionError> {
    match value {
        "system" | "developer" => Ok(InternalRole::System),
        "user" => Ok(InternalRole::User),
        "assistant" => Ok(InternalRole::Assistant),
        "tool" => Err(FormatConversionError::unsupported_feature(FORMAT, "tool role")),
        _ => Err(FormatConversionError::invalid_payload(FORMAT, format!("unknown role: {value}"))),
    }
}

fn openai_role(role: &InternalRole) -> &'static str {
    match role {
        InternalRole::System => "system",
        InternalRole::User => "user",
        InternalRole::Assistant => "assistant",
    }
}

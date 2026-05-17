use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalRequest};

use super::{
    common::{insert_optional_integer, insert_optional_number, optional_bool, optional_f64, optional_string, optional_u32},
    request_codec::{
        joined_system, messages_from_internal, parse_messages, parse_tool_choice, parse_tools, system_messages, tool_choice_from_internal, tools_from_internal,
    },
};

pub fn to_internal(request: &Value) -> Result<InternalRequest, FormatConversionError> {
    let mut messages = system_messages(request)?;
    messages.extend(parse_messages(request)?);
    let mut internal = InternalRequest::new(
        optional_string(request, "model").unwrap_or_default(),
        messages,
        optional_bool(request, "stream").unwrap_or(false),
    );
    internal.temperature = optional_f64(request, "temperature");
    internal.max_tokens = optional_u32(request, "max_tokens");
    internal.tools = parse_tools(request.get("tools"))?;
    internal.tool_choice = parse_tool_choice(request.get("tool_choice"))?;
    internal.top_p = optional_f64(request, "top_p");
    internal.stop_sequences = parse_stop_sequences(request.get("stop_sequences"))?;
    internal.thinking_budget_tokens = thinking_budget(request.get("thinking"));
    Ok(internal)
}

pub fn from_internal(internal: &InternalRequest) -> Result<Value, FormatConversionError> {
    let mut output = Map::new();
    output.insert("model".into(), Value::String(internal.model.clone()));
    output.insert("messages".into(), Value::Array(messages_from_internal(&internal.messages)?));
    if let Some(system) = joined_system(&internal.messages)? {
        output.insert("system".into(), Value::String(system));
    }
    insert_optional_integer(&mut output, "max_tokens", internal.max_tokens);
    insert_optional_number(&mut output, "temperature", internal.temperature);
    insert_optional_number(&mut output, "top_p", internal.top_p);
    insert_stop_sequences(&mut output, &internal.stop_sequences);
    insert_tool_fields(&mut output, internal);
    if let Some(budget) = internal.thinking_budget_tokens {
        output.insert("thinking".into(), json!({ "type": "enabled", "budget_tokens": budget }));
    }
    if internal.stream {
        output.insert("stream".into(), Value::Bool(true));
    }
    Ok(Value::Object(output))
}

fn parse_stop_sequences(value: Option<&Value>) -> Result<Vec<String>, FormatConversionError> {
    match value {
        None => Ok(Vec::new()),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| FormatConversionError::invalid_payload(super::common::FORMAT, "$.stop_sequences[]"))
            })
            .collect(),
        Some(_) => Err(FormatConversionError::invalid_payload(super::common::FORMAT, "$.stop_sequences")),
    }
}

fn insert_stop_sequences(output: &mut Map<String, Value>, stop_sequences: &[String]) {
    if !stop_sequences.is_empty() {
        output.insert(
            "stop_sequences".into(),
            Value::Array(stop_sequences.iter().cloned().map(Value::String).collect()),
        );
    }
}

fn insert_tool_fields(output: &mut Map<String, Value>, internal: &InternalRequest) {
    if !internal.tools.is_empty() {
        output.insert("tools".into(), Value::Array(tools_from_internal(&internal.tools)));
    }
    if let Some(choice) = &internal.tool_choice {
        output.insert("tool_choice".into(), tool_choice_from_internal(choice));
    }
}

fn thinking_budget(value: Option<&Value>) -> Option<u32> {
    value?.get("budget_tokens")?.as_u64().and_then(|value| u32::try_from(value).ok())
}

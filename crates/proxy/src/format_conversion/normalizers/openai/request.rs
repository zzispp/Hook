use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalRequest};

use super::{
    common::{insert_optional_integer, insert_optional_number, optional_bool, optional_f64, optional_string_value, optional_u32, required_string},
    request_codec::{parse_request_messages, parse_tool_choice, parse_tools, request_messages_from_internal, tool_choice_from_internal, tools_from_internal},
};

pub fn to_internal(request: &Value) -> Result<InternalRequest, FormatConversionError> {
    let mut internal = InternalRequest::new(
        required_string(request, "model", "$.model")?,
        parse_request_messages(request)?,
        optional_bool(request, "stream").unwrap_or(false),
    );
    internal.temperature = optional_f64(request, "temperature");
    internal.max_tokens = optional_u32(request, "max_completion_tokens").or(optional_u32(request, "max_tokens"));
    internal.tools = parse_tools(request.get("tools"))?;
    internal.tool_choice = parse_tool_choice(request.get("tool_choice"))?;
    internal.parallel_tool_calls = optional_bool(request, "parallel_tool_calls");
    internal.top_p = optional_f64(request, "top_p");
    internal.stop_sequences = parse_stop(request.get("stop"))?;
    internal.response_format = request.get("response_format").cloned();
    internal.reasoning_effort = optional_string_value(request.get("reasoning_effort"));
    Ok(internal)
}

pub fn from_internal(internal: &InternalRequest) -> Result<Value, FormatConversionError> {
    let mut output = Map::new();
    output.insert("model".into(), Value::String(internal.model.clone()));
    output.insert("messages".into(), Value::Array(request_messages_from_internal(&internal.messages)?));
    insert_optional_number(&mut output, "temperature", internal.temperature);
    insert_optional_integer(&mut output, "max_tokens", internal.max_tokens);
    insert_optional_number(&mut output, "top_p", internal.top_p);
    insert_stop(&mut output, &internal.stop_sequences);
    insert_optional_value(&mut output, "response_format", internal.response_format.as_ref());
    insert_tool_fields(&mut output, internal);
    if let Some(reasoning_effort) = &internal.reasoning_effort {
        output.insert("reasoning_effort".into(), Value::String(reasoning_effort.clone()));
    }
    if let Some(parallel) = internal.parallel_tool_calls {
        output.insert("parallel_tool_calls".into(), Value::Bool(parallel));
    }
    if internal.stream {
        output.insert("stream".into(), Value::Bool(true));
        output.insert("stream_options".into(), json!({ "include_usage": true }));
    }
    Ok(Value::Object(output))
}

fn parse_stop(value: Option<&Value>) -> Result<Vec<String>, FormatConversionError> {
    match value {
        None => Ok(Vec::new()),
        Some(Value::String(text)) => Ok(vec![text.clone()]),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| FormatConversionError::invalid_payload(super::common::FORMAT, "$.stop[]"))
            })
            .collect(),
        Some(_) => Err(FormatConversionError::invalid_payload(super::common::FORMAT, "$.stop")),
    }
}

fn insert_stop(output: &mut Map<String, Value>, stop_sequences: &[String]) {
    if stop_sequences.is_empty() {
        return;
    }
    output.insert("stop".into(), Value::Array(stop_sequences.iter().cloned().map(Value::String).collect()));
}

fn insert_tool_fields(output: &mut Map<String, Value>, internal: &InternalRequest) {
    if !internal.tools.is_empty() {
        output.insert("tools".into(), Value::Array(tools_from_internal(&internal.tools)));
    }
    if let Some(choice) = &internal.tool_choice {
        output.insert("tool_choice".into(), tool_choice_from_internal(choice));
    }
}

fn insert_optional_value(output: &mut Map<String, Value>, key: &str, value: Option<&Value>) {
    if let Some(value) = value {
        output.insert(key.into(), value.clone());
    }
}

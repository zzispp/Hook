use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalRequest};

use super::{
    common::{insert_optional_integer, insert_optional_number, optional_bool, optional_f64, optional_string, optional_u32},
    request_codec::{
        messages_from_internal, parse_messages, parse_tool_choice, parse_tools, system_from_internal, system_messages, tool_choice_from_internal,
        tools_from_internal, web_search_tool_from_options,
    },
};

const REASONING_TO_CLAUDE_EFFORT: &[(&str, &str)] = &[("low", "low"), ("medium", "medium"), ("high", "high"), ("xhigh", "max")];
const CLAUDE_TO_REASONING_EFFORT: &[(&str, &str)] = &[("low", "low"), ("medium", "medium"), ("high", "high"), ("max", "xhigh")];

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
    internal.parallel_tool_calls = parallel_tool_calls(request.get("tool_choice"));
    internal.top_p = optional_f64(request, "top_p");
    internal.top_k = optional_u32(request, "top_k");
    internal.stop_sequences = parse_stop_sequences(request.get("stop_sequences"))?;
    internal.thinking_budget_tokens = thinking_budget(request.get("thinking"));
    internal.reasoning_effort = output_config_effort(request);
    Ok(internal)
}

pub fn from_internal(internal: &InternalRequest) -> Result<Value, FormatConversionError> {
    let mut output = Map::new();
    output.insert("model".into(), Value::String(internal.model.clone()));
    output.insert("messages".into(), Value::Array(messages_from_internal(&internal.messages)?));
    if let Some(system) = system_from_internal(&internal.messages)? {
        output.insert("system".into(), system);
    }
    output.insert("max_tokens".into(), Value::Number(internal.max_tokens.unwrap_or(8192).into()));
    insert_optional_number(&mut output, "temperature", internal.temperature);
    insert_optional_number(&mut output, "top_p", internal.top_p);
    insert_optional_integer(&mut output, "top_k", internal.top_k);
    insert_stop_sequences(&mut output, &internal.stop_sequences);
    insert_tool_fields(&mut output, internal);
    if let Some(budget) = internal.thinking_budget_tokens {
        output.insert("thinking".into(), json!({ "type": "enabled", "budget_tokens": budget }));
    }
    insert_output_config(&mut output, internal.reasoning_effort.as_deref());
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
    let mut tools = tools_from_internal(&internal.tools);
    if let Some(options) = internal.extra.get("web_search_options") {
        tools.push(web_search_tool_from_options(options));
    }
    if !tools.is_empty() {
        output.insert("tools".into(), Value::Array(tools));
    }
    if let Some(choice) = &internal.tool_choice {
        let mut tool_choice = tool_choice_from_internal(choice);
        if internal.parallel_tool_calls == Some(false) && choice != &crate::format_conversion::InternalToolChoice::None {
            tool_choice["disable_parallel_tool_use"] = Value::Bool(true);
        }
        output.insert("tool_choice".into(), tool_choice);
    }
}

fn thinking_budget(value: Option<&Value>) -> Option<u32> {
    value?.get("budget_tokens")?.as_u64().and_then(|value| u32::try_from(value).ok())
}

fn parallel_tool_calls(value: Option<&Value>) -> Option<bool> {
    let disabled = value?.get("disable_parallel_tool_use").and_then(Value::as_bool)?;
    Some(!disabled)
}

fn output_config_effort(request: &Value) -> Option<String> {
    let effort = request.get("output_config")?.get("effort")?.as_str()?;
    CLAUDE_TO_REASONING_EFFORT
        .iter()
        .find_map(|(key, value)| (*key == effort).then(|| (*value).to_owned()))
}

fn insert_output_config(output: &mut Map<String, Value>, effort: Option<&str>) {
    let Some(effort) = effort.and_then(claude_effort) else {
        return;
    };
    output.insert("output_config".into(), json!({ "effort": effort }));
}

fn claude_effort(effort: &str) -> Option<&'static str> {
    REASONING_TO_CLAUDE_EFFORT.iter().find_map(|(key, value)| (*key == effort).then_some(*value))
}

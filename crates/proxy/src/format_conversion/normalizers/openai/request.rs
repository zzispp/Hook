use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalRequest};

use super::{
    common::{insert_optional_integer, insert_optional_number, optional_bool, optional_f64, optional_string_value, optional_u32, required_string},
    request_codec::{parse_request_messages, parse_tool_choice, parse_tools, request_messages_from_internal, tool_choice_from_internal, tools_from_internal},
};

const REASONING_BUDGETS: &[(&str, u32)] = &[("low", 1280), ("medium", 2048), ("high", 4096), ("xhigh", 8192)];
const BUDGET_THRESHOLDS: &[(u32, &str)] = &[(1664, "low"), (3072, "medium"), (6144, "high"), (u32::MAX, "xhigh")];

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
    internal.n = optional_u32(request, "n");
    internal.presence_penalty = optional_f64(request, "presence_penalty");
    internal.frequency_penalty = optional_f64(request, "frequency_penalty");
    internal.seed = optional_u32(request, "seed");
    internal.logprobs = optional_bool(request, "logprobs");
    internal.top_logprobs = optional_u32(request, "top_logprobs");
    internal.stop_sequences = parse_stop(request.get("stop"))?;
    internal.response_format = request.get("response_format").cloned();
    internal.reasoning_effort = optional_string_value(request.get("reasoning_effort"));
    internal.thinking_budget_tokens = internal.reasoning_effort.as_deref().and_then(reasoning_budget);
    if let Some(options) = request.get("web_search_options") {
        internal.extra.insert("web_search_options".into(), options.clone());
    }
    if let Some(google) = request
        .get("extra_body")
        .and_then(|value| value.get("google"))
        .filter(|value| value.is_object())
    {
        internal.extra.insert("google".into(), google.clone());
    }
    if let Some(verbosity) = request.get("verbosity").and_then(Value::as_str).filter(|value| !value.is_empty()) {
        internal.extra.insert("verbosity".into(), Value::String(verbosity.to_owned()));
    }
    Ok(internal)
}

pub fn from_internal(internal: &InternalRequest) -> Result<Value, FormatConversionError> {
    let mut output = Map::new();
    output.insert("model".into(), Value::String(internal.model.clone()));
    output.insert("messages".into(), Value::Array(request_messages_from_internal(&internal.messages)?));
    insert_optional_number(&mut output, "temperature", internal.temperature);
    insert_optional_integer(&mut output, "max_tokens", internal.max_tokens);
    insert_optional_number(&mut output, "top_p", internal.top_p);
    insert_optional_integer(&mut output, "n", internal.n.filter(|value| *value > 1));
    insert_optional_number(&mut output, "presence_penalty", internal.presence_penalty);
    insert_optional_number(&mut output, "frequency_penalty", internal.frequency_penalty);
    insert_optional_integer(&mut output, "seed", internal.seed);
    if let Some(logprobs) = internal.logprobs {
        output.insert("logprobs".into(), Value::Bool(logprobs));
    }
    insert_optional_integer(&mut output, "top_logprobs", internal.top_logprobs);
    insert_stop(&mut output, &internal.stop_sequences);
    insert_optional_value(&mut output, "response_format", internal.response_format.as_ref());
    insert_tool_fields(&mut output, internal);
    if let Some(reasoning_effort) = reasoning_effort_from_internal(internal) {
        output.insert("reasoning_effort".into(), Value::String(reasoning_effort));
    }
    if let Some(parallel) = internal.parallel_tool_calls {
        output.insert("parallel_tool_calls".into(), Value::Bool(parallel));
    }
    if internal.stream {
        output.insert("stream".into(), Value::Bool(true));
        output.insert("stream_options".into(), json!({ "include_usage": true }));
    }
    if let Some(verbosity) = internal.extra.get("verbosity").and_then(Value::as_str).filter(|value| !value.is_empty()) {
        output.insert("verbosity".into(), Value::String(verbosity.to_owned()));
    }
    if let Some(options) = internal.extra.get("web_search_options") {
        output.insert("web_search_options".into(), options.clone());
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
    let tools = internal
        .tools
        .iter()
        .filter(|tool| !tool.extra.contains_key("gemini_builtin_tool"))
        .filter(|tool| !is_non_chat_responses_tool(tool))
        .cloned()
        .collect::<Vec<_>>();
    if !tools.is_empty() {
        output.insert("tools".into(), Value::Array(tools_from_internal(&tools)));
    }
    if let Some(choice) = &internal.tool_choice {
        output.insert("tool_choice".into(), tool_choice_from_internal(choice));
    }
}

fn is_non_chat_responses_tool(tool: &crate::format_conversion::InternalTool) -> bool {
    let Some(raw) = tool.extra.get("openai_responses_raw_tool").and_then(Value::as_object) else {
        return false;
    };
    !matches!(raw.get("type").and_then(Value::as_str), Some("function" | "custom"))
}

fn insert_optional_value(output: &mut Map<String, Value>, key: &str, value: Option<&Value>) {
    if let Some(value) = value {
        output.insert(key.into(), value.clone());
    }
}

fn reasoning_budget(effort: &str) -> Option<u32> {
    REASONING_BUDGETS.iter().find_map(|(key, value)| (*key == effort).then_some(*value))
}

fn reasoning_effort_from_internal(internal: &InternalRequest) -> Option<String> {
    if let Some(effort) = &internal.reasoning_effort {
        return Some(openai_reasoning_effort(effort).to_owned());
    }
    let budget = internal.thinking_budget_tokens?;
    BUDGET_THRESHOLDS
        .iter()
        .find_map(|(threshold, effort)| (budget <= *threshold).then(|| openai_reasoning_effort(effort).to_owned()))
}

fn openai_reasoning_effort(effort: &str) -> &str {
    if effort == "xhigh" { "high" } else { effort }
}

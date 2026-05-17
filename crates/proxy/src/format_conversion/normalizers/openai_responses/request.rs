use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalRequest};

use super::{
    request_codec::{input_messages, messages_from_internal, parse_tool_choice, parse_tools, tool_choice_from_internal, tools_from_internal},
    request_fields::{bool_field, insert_integer, insert_number, number_field, string_field, u32_field},
};

pub fn to_internal(request: &Value) -> Result<InternalRequest, FormatConversionError> {
    let mut internal = InternalRequest::new(
        string_field(request, "model", "$.model")?,
        input_messages(request.get("input"))?,
        bool_field(request, "stream").unwrap_or(false),
    );
    internal.temperature = number_field(request, "temperature");
    internal.max_tokens = u32_field(request, "max_output_tokens").or(u32_field(request, "max_tokens"));
    internal.tools = parse_tools(request.get("tools"))?;
    internal.tool_choice = parse_tool_choice(request.get("tool_choice"))?;
    internal.top_p = number_field(request, "top_p");
    internal.response_format = request.get("text").cloned();
    if let Some(reasoning) = request.get("reasoning").and_then(Value::as_object) {
        internal.reasoning_effort = reasoning.get("effort").and_then(Value::as_str).map(str::to_owned);
    }
    Ok(internal)
}

pub fn from_internal(internal: &InternalRequest) -> Result<Value, FormatConversionError> {
    let mut output = Map::new();
    output.insert("model".into(), Value::String(internal.model.clone()));
    output.insert("input".into(), Value::Array(messages_from_internal(&internal.messages)?));
    insert_number(&mut output, "temperature", internal.temperature);
    insert_number(&mut output, "top_p", internal.top_p);
    insert_integer(&mut output, "max_output_tokens", internal.max_tokens);
    insert_optional(&mut output, "text", internal.response_format.as_ref());
    insert_reasoning(&mut output, internal.reasoning_effort.as_deref());
    insert_tool_fields(&mut output, internal);
    if internal.stream {
        output.insert("stream".into(), Value::Bool(true));
    }
    Ok(Value::Object(output))
}

fn insert_optional(output: &mut Map<String, Value>, key: &str, value: Option<&Value>) {
    if let Some(value) = value {
        output.insert(key.into(), value.clone());
    }
}

fn insert_reasoning(output: &mut Map<String, Value>, effort: Option<&str>) {
    if let Some(effort) = effort {
        output.insert("reasoning".into(), json!({ "effort": effort }));
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

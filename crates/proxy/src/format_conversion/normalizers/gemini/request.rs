use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalRequest};

use super::{
    common::{
        generation_config, generation_config_value, insert_optional_integer, insert_optional_number, optional_bool, optional_f64_from_config, optional_string,
        optional_u32_from_config,
    },
    request_codec::{contents_from_internal, joined_system, parse_messages},
    request_tools::{parse_stop_sequences, parse_tool_choice, parse_tools, tool_choice_from_internal, tools_from_internal},
};

pub fn to_internal(request: &Value) -> Result<InternalRequest, FormatConversionError> {
    let config = generation_config(request);
    let mut internal = InternalRequest::new(
        optional_string(request, "model").unwrap_or_default(),
        parse_messages(request)?,
        optional_bool(request, "stream").unwrap_or(false),
    );
    internal.temperature = optional_f64_from_config(config, "temperature", "temperature");
    internal.max_tokens = optional_u32_from_config(config, "maxOutputTokens", "max_output_tokens");
    internal.tools = parse_tools(request.get("tools"))?;
    internal.tool_choice = parse_tool_choice(request.get("toolConfig").or_else(|| request.get("tool_config")))?;
    internal.top_p = optional_f64_from_config(config, "topP", "top_p");
    internal.stop_sequences = parse_stop_sequences(config)?;
    internal.thinking_budget_tokens = config
        .and_then(|value| generation_config_value(value, "thinkingConfig", "thinking_config"))
        .and_then(thinking_budget);
    Ok(internal)
}

pub fn from_internal(internal: &InternalRequest) -> Result<Value, FormatConversionError> {
    let mut output = Map::new();
    output.insert("model".into(), Value::String(internal.model.clone()));
    output.insert("contents".into(), Value::Array(contents_from_internal(&internal.messages)?));
    if let Some(system) = joined_system(&internal.messages)? {
        output.insert("systemInstruction".into(), json!({ "parts": [{ "text": system }] }));
    }
    let config = generation_config_from_internal(internal);
    if !config.is_empty() {
        output.insert("generationConfig".into(), Value::Object(config));
    }
    if !internal.tools.is_empty() {
        output.insert("tools".into(), Value::Array(tools_from_internal(&internal.tools)));
    }
    if let Some(choice) = &internal.tool_choice {
        output.insert("toolConfig".into(), tool_choice_from_internal(choice));
    }
    Ok(Value::Object(output))
}

fn generation_config_from_internal(internal: &InternalRequest) -> Map<String, Value> {
    let mut config = Map::new();
    insert_optional_integer(&mut config, "maxOutputTokens", internal.max_tokens);
    insert_optional_number(&mut config, "temperature", internal.temperature);
    insert_optional_number(&mut config, "topP", internal.top_p);
    if !internal.stop_sequences.is_empty() {
        config.insert(
            "stopSequences".into(),
            Value::Array(internal.stop_sequences.iter().cloned().map(Value::String).collect()),
        );
    }
    if let Some(budget) = internal.thinking_budget_tokens {
        config.insert("thinkingConfig".into(), json!({ "thinkingBudget": budget }));
    }
    config
}

fn thinking_budget(value: &Value) -> Option<u32> {
    value
        .get("thinkingBudget")
        .or_else(|| value.get("thinking_budget"))
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
}

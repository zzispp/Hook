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
    parse_native_gemini_extras(request, &mut internal);
    internal.temperature = optional_f64_from_config(config, "temperature", "temperature");
    internal.max_tokens = optional_u32_from_config(config, "maxOutputTokens", "max_output_tokens");
    internal.tools = parse_tools(request.get("tools"))?;
    internal.tool_choice = parse_tool_choice(request.get("toolConfig").or_else(|| request.get("tool_config")))?;
    internal.top_p = optional_f64_from_config(config, "topP", "top_p");
    internal.top_k = optional_u32_from_config(config, "topK", "top_k");
    internal.stop_sequences = parse_stop_sequences(config)?;
    if let Some(config) = config {
        parse_generation_config(config, &mut internal);
    }
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
    let tools = tools_for_internal(internal);
    if !tools.is_empty() {
        output.insert("tools".into(), Value::Array(tools));
    }
    if let Some(choice) = &internal.tool_choice {
        output.insert("toolConfig".into(), tool_choice_from_internal(choice));
    }
    if let Some(safety) = internal.extra.get("gemini_safety_settings") {
        output.insert("safetySettings".into(), safety.clone());
    }
    if let Some(cached) = internal.extra.get("gemini_cached_content") {
        output.insert("cachedContent".into(), cached.clone());
    }
    Ok(Value::Object(output))
}

fn generation_config_from_internal(internal: &InternalRequest) -> Map<String, Value> {
    let mut config = Map::new();
    insert_optional_integer(&mut config, "maxOutputTokens", internal.max_tokens);
    insert_optional_number(&mut config, "temperature", internal.temperature);
    insert_optional_number(&mut config, "topP", internal.top_p);
    insert_optional_integer(&mut config, "topK", internal.top_k);
    if !internal.stop_sequences.is_empty() {
        config.insert(
            "stopSequences".into(),
            Value::Array(internal.stop_sequences.iter().cloned().map(Value::String).collect()),
        );
    }
    if let Some(budget) = internal.thinking_budget_tokens {
        config.insert("thinkingConfig".into(), json!({ "includeThoughts": true, "thinkingBudget": budget }));
    }
    insert_response_format(&mut config, internal.response_format.as_ref());
    if let Some(options) = internal.extra.get("gemini_thinking_config") {
        config.entry("thinkingConfig").or_insert_with(|| options.clone());
    }
    if let Some(google) = internal.extra.get("google").and_then(Value::as_object) {
        insert_google_extra(&mut config, google);
    }
    if let Some(modalities) = internal.extra.get("gemini_response_modalities") {
        config.insert("responseModalities".into(), modalities.clone());
    }
    if let Some(extra_config) = internal.extra.get("gemini_generation_config_extra").and_then(Value::as_object) {
        for (key, value) in extra_config {
            config.entry(key.clone()).or_insert_with(|| value.clone());
        }
    }
    if let Some(n) = internal.n.filter(|value| *value > 1) {
        config.insert("candidateCount".into(), Value::Number(n.into()));
    }
    config
}

fn parse_native_gemini_extras(request: &Value, internal: &mut InternalRequest) {
    if let Some(value) = request.get("safetySettings").or_else(|| request.get("safety_settings")) {
        internal.extra.insert("gemini_safety_settings".into(), value.clone());
    }
    if let Some(value) = request.get("cachedContent").or_else(|| request.get("cached_content")) {
        internal.extra.insert("gemini_cached_content".into(), value.clone());
    }
}

fn tools_for_internal(internal: &InternalRequest) -> Vec<Value> {
    let mut tools = tools_from_internal(&internal.tools);
    if internal.extra.contains_key("web_search_options") && !has_google_search(&tools) {
        tools.push(json!({ "googleSearch": {} }));
    }
    tools
}

fn has_google_search(tools: &[Value]) -> bool {
    tools
        .iter()
        .any(|tool| tool.get("googleSearch").is_some() || tool.get("google_search").is_some())
}

fn thinking_budget(value: &Value) -> Option<u32> {
    value
        .get("thinkingBudget")
        .or_else(|| value.get("thinking_budget"))
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
}

fn parse_generation_config(config: &Map<String, Value>, internal: &mut InternalRequest) {
    if let Some(thinking_config) = generation_config_value(config, "thinkingConfig", "thinking_config") {
        internal.thinking_budget_tokens = thinking_budget(thinking_config);
        internal.extra.insert("gemini_thinking_config".into(), thinking_config.clone());
    }
    if let Some(response_modalities) = generation_config_value(config, "responseModalities", "response_modalities") {
        internal.extra.insert("gemini_response_modalities".into(), response_modalities.clone());
    }
    internal.n = generation_config_value(config, "candidateCount", "candidate_count")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok());
    let extra = generation_config_extra(config);
    if !extra.is_empty() {
        internal.extra.insert("gemini_generation_config_extra".into(), Value::Object(extra));
    }
    internal.response_format = response_format(config);
}

fn response_format(config: &Map<String, Value>) -> Option<Value> {
    let mime = generation_config_value(config, "responseMimeType", "response_mime_type");
    let schema = generation_config_value(config, "responseSchema", "response_schema");
    if let Some(schema) = schema {
        return Some(json!({ "type": "json_schema", "json_schema": schema }));
    }
    let mime_text = mime.and_then(Value::as_str)?;
    if mime_text.contains("json") {
        return Some(json!({ "type": "json_object" }));
    }
    Some(json!({ "type": "text", "response_mime_type": mime_text }))
}

fn insert_response_format(config: &mut Map<String, Value>, value: Option<&Value>) {
    let Some(object) = value.and_then(Value::as_object) else {
        return;
    };
    match object.get("type").and_then(Value::as_str) {
        Some("json_schema") => {
            config.insert("responseMimeType".into(), Value::String("application/json".into()));
            if let Some(schema) = object.get("json_schema").map(unwrap_openai_schema) {
                config.insert("responseSchema".into(), crate::format_conversion::schema_utils::clean_gemini_schema(&schema));
            }
        }
        Some("json_object") => {
            config.insert("responseMimeType".into(), Value::String("application/json".into()));
        }
        Some("text") => {
            if let Some(mime) = object.get("response_mime_type") {
                config.insert("responseMimeType".into(), mime.clone());
            }
        }
        _ => {}
    }
}

fn unwrap_openai_schema(value: &Value) -> Value {
    value
        .get("schema")
        .filter(|schema| schema.is_object())
        .cloned()
        .unwrap_or_else(|| value.clone())
}

fn insert_google_extra(config: &mut Map<String, Value>, google: &Map<String, Value>) {
    if let Some(thinking) = google.get("thinking_config").and_then(Value::as_object) {
        let mut output = Map::new();
        if let Some(value) = thinking.get("thinking_budget") {
            output.insert("thinkingBudget".into(), value.clone());
        }
        if let Some(value) = thinking.get("include_thoughts") {
            output.insert("includeThoughts".into(), value.clone());
        }
        for (key, value) in thinking {
            if key != "thinking_budget" && key != "include_thoughts" {
                output.insert(key.clone(), value.clone());
            }
        }
        if !output.is_empty() {
            config.entry("thinkingConfig").or_insert(Value::Object(output));
        }
    }
    if let Some(value) = google.get("response_modalities") {
        config.entry("responseModalities").or_insert_with(|| value.clone());
    }
}

fn generation_config_extra(config: &Map<String, Value>) -> Map<String, Value> {
    config
        .iter()
        .filter(|(key, _)| !known_generation_config_key(key))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn known_generation_config_key(key: &str) -> bool {
    matches!(
        key,
        "maxOutputTokens"
            | "max_output_tokens"
            | "temperature"
            | "topP"
            | "top_p"
            | "topK"
            | "top_k"
            | "stopSequences"
            | "stop_sequences"
            | "thinkingConfig"
            | "thinking_config"
            | "responseModalities"
            | "response_modalities"
            | "candidateCount"
            | "candidate_count"
            | "responseMimeType"
            | "response_mime_type"
            | "responseSchema"
            | "response_schema"
    )
}

use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalTool, InternalToolChoice};

use super::request_fields::FORMAT;

pub(super) fn parse_tools(value: Option<&Value>) -> Result<Vec<InternalTool>, FormatConversionError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    value
        .as_array()
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, "$.tools"))?
        .iter()
        .enumerate()
        .map(parse_tool)
        .collect()
}

pub(super) fn parse_tool_choice(value: Option<&Value>) -> Result<Option<InternalToolChoice>, FormatConversionError> {
    match value {
        None => Ok(None),
        Some(Value::String(text)) if text == "auto" => Ok(Some(InternalToolChoice::Auto)),
        Some(Value::String(text)) if text == "none" => Ok(Some(InternalToolChoice::None)),
        Some(Value::String(text)) if text == "required" => Ok(Some(InternalToolChoice::Required)),
        Some(Value::String(_)) => Ok(Some(InternalToolChoice::Auto)),
        Some(Value::Object(object)) => Ok(responses_tool_choice_name(object)
            .map(InternalToolChoice::Tool)
            .or(Some(InternalToolChoice::Auto))),
        Some(_) => Ok(Some(InternalToolChoice::Auto)),
    }
}

pub(super) fn tools_from_internal(tools: &[InternalTool]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            if let Some(raw) = tool.extra.get("openai_responses_raw_tool").and_then(Value::as_object) {
                return Value::Object(raw.clone());
            }
            if let Some(raw) = tool.extra.get("openai_chat_raw_tool").and_then(Value::as_object) {
                if let Some(tool) = chat_tool_to_responses_tool(raw) {
                    return tool;
                }
            }
            json!({
                "type": "function",
                "name": tool.name,
                "description": tool.description,
                "parameters": tool.parameters.as_ref().map(crate::format_conversion::schema_utils::openai_schema_with_object_fixes),
            })
        })
        .collect()
}

pub(super) fn tool_choice_from_internal(choice: &InternalToolChoice) -> Value {
    match choice {
        InternalToolChoice::Auto => Value::String("auto".into()),
        InternalToolChoice::None => Value::String("none".into()),
        InternalToolChoice::Required => Value::String("required".into()),
        InternalToolChoice::Tool(name) => json!({ "type": "function", "name": name }),
    }
}

fn parse_tool(value: (usize, &Value)) -> Result<InternalTool, FormatConversionError> {
    let (index, value) = value;
    let object = value
        .as_object()
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("$.tools[{index}]")))?;
    let tool_type = object.get("type").and_then(Value::as_str).unwrap_or("function");
    if tool_type != "function" {
        return parse_hosted_tool(object, tool_type);
    }
    Ok(InternalTool {
        name: required_text(object, &format!("$.tools[{index}]"), "name")?.to_owned(),
        description: object.get("description").and_then(Value::as_str).map(str::to_owned),
        parameters: object.get("parameters").cloned(),
        extra: Map::new(),
    })
}

fn responses_tool_choice_name(object: &Map<String, Value>) -> Option<String> {
    if object.get("type").and_then(Value::as_str) == Some("function") {
        return object
            .get("name")
            .or_else(|| object.get("function").and_then(|value| value.get("name")))
            .and_then(Value::as_str)
            .map(str::to_owned);
    }
    if object.get("type").and_then(Value::as_str) == Some("custom") {
        return object
            .get("name")
            .or_else(|| object.get("custom").and_then(|value| value.get("name")))
            .and_then(Value::as_str)
            .map(str::to_owned);
    }
    None
}

fn chat_tool_to_responses_tool(tool: &Map<String, Value>) -> Option<Value> {
    match tool.get("type").and_then(Value::as_str)? {
        "function" => chat_function_tool_to_responses(tool),
        "custom" => chat_custom_tool_to_responses(tool),
        _ => None,
    }
}

fn chat_function_tool_to_responses(tool: &Map<String, Value>) -> Option<Value> {
    let function = tool.get("function")?.as_object()?;
    let name = function.get("name").and_then(Value::as_str).filter(|value| !value.is_empty())?;
    let mut output = Map::new();
    output.insert("type".into(), Value::String("function".into()));
    output.insert("name".into(), Value::String(name.to_owned()));
    insert_optional_clone(&mut output, function, "description");
    insert_optional_clone(&mut output, function, "parameters");
    insert_optional_clone(&mut output, function, "strict");
    Some(Value::Object(output))
}

fn chat_custom_tool_to_responses(tool: &Map<String, Value>) -> Option<Value> {
    let custom = tool.get("custom")?.as_object()?;
    let name = custom.get("name").and_then(Value::as_str).filter(|value| !value.is_empty())?;
    let mut output = Map::new();
    output.insert("type".into(), Value::String("custom".into()));
    output.insert("name".into(), Value::String(name.to_owned()));
    insert_optional_clone(&mut output, custom, "description");
    insert_optional_clone(&mut output, custom, "format");
    Some(Value::Object(output))
}

fn insert_optional_clone(output: &mut Map<String, Value>, source: &Map<String, Value>, key: &str) {
    if let Some(value) = source.get(key) {
        output.insert(key.to_owned(), value.clone());
    }
}

fn parse_hosted_tool(object: &Map<String, Value>, tool_type: &str) -> Result<InternalTool, FormatConversionError> {
    let name = hosted_tool_name(object, tool_type);
    let mut extra = Map::new();
    extra.insert("openai_responses_raw_tool".into(), Value::Object(object.clone()));
    if tool_type.starts_with("web_search") {
        extra.insert("web_search_options".into(), web_search_options(object));
    }
    Ok(InternalTool {
        name,
        description: object.get("description").and_then(Value::as_str).map(str::to_owned),
        parameters: object.get("parameters").cloned(),
        extra,
    })
}

fn hosted_tool_name(object: &Map<String, Value>, tool_type: &str) -> String {
    object
        .get("name")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(tool_type)
        .to_owned()
}

fn web_search_options(object: &Map<String, Value>) -> Value {
    let mut options = Map::new();
    if let Some(value) = object.get("search_context_size") {
        options.insert("search_context_size".into(), value.clone());
    }
    if let Some(value) = object.get("user_location") {
        options.insert("user_location".into(), value.clone());
    }
    Value::Object(options)
}

fn required_text<'a>(object: &'a Map<String, Value>, path: &str, key: &str) -> Result<&'a str, FormatConversionError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("{path}.{key}")))
}

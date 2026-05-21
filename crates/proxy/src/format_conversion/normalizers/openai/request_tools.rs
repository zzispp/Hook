use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalTool, InternalToolChoice};

use super::common::{FORMAT, required_object};

pub(super) fn parse_tools(value: Option<&Value>) -> Result<Vec<InternalTool>, FormatConversionError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let tools = value.as_array().ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, "$.tools"))?;
    tools.iter().enumerate().map(parse_tool).collect()
}

pub(super) fn parse_tool_choice(value: Option<&Value>) -> Result<Option<InternalToolChoice>, FormatConversionError> {
    match value {
        None => Ok(None),
        Some(Value::String(text)) if text == "auto" => Ok(Some(InternalToolChoice::Auto)),
        Some(Value::String(text)) if text == "none" => Ok(Some(InternalToolChoice::None)),
        Some(Value::String(text)) if text == "required" => Ok(Some(InternalToolChoice::Required)),
        Some(Value::String(_)) => Ok(Some(InternalToolChoice::Auto)),
        Some(Value::Object(object)) => object_tool_choice(object),
        Some(_) => Ok(Some(InternalToolChoice::Auto)),
    }
}

pub(super) fn tools_from_internal(tools: &[InternalTool]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            if let Some(raw) = tool.extra.get("openai_chat_raw_tool").and_then(Value::as_object) {
                return Value::Object(raw.clone());
            }
            if let Some(raw) = tool.extra.get("openai_responses_raw_tool").and_then(Value::as_object) {
                if let Some(tool) = responses_tool_to_chat_tool(raw) {
                    return tool;
                }
            }
            function_tool_from_internal(tool)
        })
        .collect()
}

pub(super) fn tool_choice_from_internal(choice: &InternalToolChoice) -> Value {
    match choice {
        InternalToolChoice::Auto => Value::String("auto".into()),
        InternalToolChoice::None => Value::String("none".into()),
        InternalToolChoice::Required => Value::String("required".into()),
        InternalToolChoice::Tool(name) => json!({ "type": "function", "function": { "name": name } }),
    }
}

fn object_tool_choice(object: &Map<String, Value>) -> Result<Option<InternalToolChoice>, FormatConversionError> {
    if object.get("type").and_then(Value::as_str) == Some("function") {
        let function = required_object(object.get("function"), "$.tool_choice.function")?;
        return Ok(function
            .get("name")
            .and_then(Value::as_str)
            .map(|name| InternalToolChoice::Tool(name.to_owned())));
    }
    if object.get("type").and_then(Value::as_str) == Some("custom") {
        return Ok(custom_tool_name(object).map(InternalToolChoice::Tool).or(Some(InternalToolChoice::Auto)));
    }
    Ok(Some(InternalToolChoice::Auto))
}

fn function_tool_from_internal(tool: &InternalTool) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": tool.name,
            "description": tool.description,
            "parameters": tool.parameters.as_ref().map(crate::format_conversion::schema_utils::openai_schema_with_object_fixes),
        },
    })
}

fn parse_tool(value: (usize, &Value)) -> Result<InternalTool, FormatConversionError> {
    let (index, value) = value;
    let object = required_object(Some(value), &format!("$.tools[{index}]"))?;
    if object.get("type").and_then(Value::as_str) == Some("custom") {
        return parse_custom_tool(object, index);
    }
    if object.get("type").and_then(Value::as_str) != Some("function") {
        return Err(FormatConversionError::invalid_payload(FORMAT, format!("$.tools[{index}].function")));
    }
    let function = required_object(object.get("function"), &format!("$.tools[{index}].function"))?;
    Ok(InternalTool {
        name: function.get("name").and_then(Value::as_str).unwrap_or_default().to_owned(),
        description: function.get("description").and_then(Value::as_str).map(str::to_owned),
        parameters: function.get("parameters").cloned(),
        extra: Map::new(),
    })
}

fn parse_custom_tool(object: &Map<String, Value>, index: usize) -> Result<InternalTool, FormatConversionError> {
    let custom = required_object(object.get("custom"), &format!("$.tools[{index}].custom"))?;
    let name = custom
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("$.tools[{index}].custom.name")))?
        .to_owned();
    let mut extra = Map::new();
    extra.insert("openai_chat_raw_tool".into(), Value::Object(object.clone()));
    Ok(InternalTool {
        name,
        description: custom.get("description").and_then(Value::as_str).map(str::to_owned),
        parameters: None,
        extra,
    })
}

fn custom_tool_name(object: &Map<String, Value>) -> Option<String> {
    object
        .get("custom")
        .and_then(Value::as_object)
        .and_then(|custom| custom.get("name"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn responses_tool_to_chat_tool(tool: &Map<String, Value>) -> Option<Value> {
    match tool.get("type").and_then(Value::as_str)? {
        "function" => responses_function_tool_to_chat(tool),
        "custom" => responses_custom_tool_to_chat(tool),
        _ => None,
    }
}

fn responses_function_tool_to_chat(tool: &Map<String, Value>) -> Option<Value> {
    let name = tool.get("name").and_then(Value::as_str).filter(|value| !value.is_empty())?;
    let mut function = Map::new();
    function.insert("name".into(), Value::String(name.to_owned()));
    insert_optional_clone(&mut function, tool, "description");
    insert_optional_clone(&mut function, tool, "parameters");
    insert_optional_clone(&mut function, tool, "strict");
    Some(json!({ "type": "function", "function": function }))
}

fn responses_custom_tool_to_chat(tool: &Map<String, Value>) -> Option<Value> {
    let name = tool.get("name").and_then(Value::as_str).filter(|value| !value.is_empty())?;
    let mut custom = Map::new();
    custom.insert("name".into(), Value::String(name.to_owned()));
    insert_optional_clone(&mut custom, tool, "description");
    insert_optional_clone(&mut custom, tool, "format");
    Some(json!({ "type": "custom", "custom": custom }))
}

fn insert_optional_clone(output: &mut Map<String, Value>, source: &Map<String, Value>, key: &str) {
    if let Some(value) = source.get(key) {
        output.insert(key.to_owned(), value.clone());
    }
}

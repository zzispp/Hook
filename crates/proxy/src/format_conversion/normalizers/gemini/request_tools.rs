use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalTool, InternalToolChoice};

use super::common::{FORMAT, generation_config_value, required_object};

const GEMINI_BUILTIN_TOOLS: &[&str] = &["googleSearch", "google_search", "codeExecution", "code_execution"];

pub(super) fn parse_tools(value: Option<&Value>) -> Result<Vec<InternalTool>, FormatConversionError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let mut tools = Vec::new();
    for (group_index, group) in value
        .as_array()
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, "$.tools"))?
        .iter()
        .enumerate()
    {
        if let Some(tool) = parse_builtin_tool(group)? {
            tools.push(tool);
            continue;
        }
        let declarations = group
            .get("functionDeclarations")
            .or_else(|| group.get("function_declarations"))
            .and_then(Value::as_array)
            .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("$.tools[{group_index}].functionDeclarations")))?;
        for (index, declaration) in declarations.iter().enumerate() {
            tools.push(parse_tool_declaration(declaration, group_index, index)?);
        }
    }
    Ok(tools)
}

pub(super) fn parse_tool_choice(value: Option<&Value>) -> Result<Option<InternalToolChoice>, FormatConversionError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let Some(config) = value.get("functionCallingConfig").or_else(|| value.get("function_calling_config")) else {
        return Ok(Some(InternalToolChoice::Auto));
    };
    let object = required_object(Some(config), "$.toolConfig.functionCallingConfig")?;
    match object.get("mode").and_then(Value::as_str).unwrap_or("AUTO").to_ascii_uppercase().as_str() {
        "AUTO" => Ok(Some(InternalToolChoice::Auto)),
        "NONE" => Ok(Some(InternalToolChoice::None)),
        "ANY" | "REQUIRED" => Ok(object
            .get("allowedFunctionNames")
            .or_else(|| object.get("allowed_function_names"))
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(Value::as_str)
            .map(|name| InternalToolChoice::Tool(name.to_owned()))
            .or(Some(InternalToolChoice::Required))),
        _ => Ok(Some(InternalToolChoice::Auto)),
    }
}

pub(super) fn tools_from_internal(tools: &[InternalTool]) -> Vec<Value> {
    let mut builtins = Vec::new();
    let declarations = tools
        .iter()
        .filter_map(|tool| {
            if let Some(builtin) = builtin_from_internal(tool) {
                builtins.push(builtin);
                return None;
            }
            Some(tool_from_internal(tool))
        })
        .collect::<Vec<_>>();
    if !declarations.is_empty() {
        builtins.insert(0, json!({ "functionDeclarations": declarations }));
    }
    builtins
}

pub(super) fn tool_choice_from_internal(choice: &InternalToolChoice) -> Value {
    match choice {
        InternalToolChoice::Auto => json!({ "functionCallingConfig": { "mode": "AUTO" } }),
        InternalToolChoice::None => json!({ "functionCallingConfig": { "mode": "NONE" } }),
        InternalToolChoice::Required => json!({ "functionCallingConfig": { "mode": "ANY" } }),
        InternalToolChoice::Tool(name) => json!({ "functionCallingConfig": { "mode": "ANY", "allowedFunctionNames": [name] } }),
    }
}

pub(super) fn parse_stop_sequences(config: Option<&Map<String, Value>>) -> Result<Vec<String>, FormatConversionError> {
    let Some(value) = config.and_then(|config| generation_config_value(config, "stopSequences", "stop_sequences")) else {
        return Ok(Vec::new());
    };
    value
        .as_array()
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, "$.generationConfig.stopSequences"))?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, "$.generationConfig.stopSequences[]"))
        })
        .collect()
}

fn parse_tool_declaration(value: &Value, group_index: usize, index: usize) -> Result<InternalTool, FormatConversionError> {
    let object = required_object(Some(value), &format!("$.tools[{group_index}].functionDeclarations[{index}]"))?;
    Ok(InternalTool {
        name: required_text(object, "functionDeclaration", "name")?.to_owned(),
        description: object.get("description").and_then(Value::as_str).map(str::to_owned),
        parameters: object.get("parameters").cloned(),
        extra: Map::new(),
    })
}

fn parse_builtin_tool(value: &Value) -> Result<Option<InternalTool>, FormatConversionError> {
    let object = required_object(Some(value), "$.tools[]")?;
    let Some(key) = GEMINI_BUILTIN_TOOLS.iter().find(|key| object.contains_key(**key)) else {
        return Ok(None);
    };
    let canonical = if key.contains("google") { "googleSearch" } else { "codeExecution" };
    let mut extra = Map::new();
    extra.insert("gemini_builtin_tool".into(), Value::String(canonical.into()));
    if canonical == "googleSearch" {
        extra.insert("web_search_options".into(), json!({}));
    }
    Ok(Some(InternalTool {
        name: canonical.to_owned(),
        description: None,
        parameters: None,
        extra,
    }))
}

fn builtin_from_internal(tool: &InternalTool) -> Option<Value> {
    if let Some(Value::String(name)) = tool.extra.get("gemini_builtin_tool") {
        return Some(json!({ name: {} }));
    }
    if tool.extra.contains_key("web_search_options") || tool.name == "googleSearch" {
        return Some(json!({ "googleSearch": {} }));
    }
    None
}

fn tool_from_internal(tool: &InternalTool) -> Value {
    json!({
        "name": tool.name,
        "description": tool.description,
        "parameters": tool.parameters.as_ref().map(crate::format_conversion::schema_utils::clean_gemini_schema),
    })
}

fn required_text<'a>(object: &'a Map<String, Value>, path: &str, key: &str) -> Result<&'a str, FormatConversionError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("{path}.{key}")))
}

use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalTool, InternalToolChoice};

use super::common::{FORMAT, generation_config_value, required_object};

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
    let config = value
        .get("functionCallingConfig")
        .or_else(|| value.get("function_calling_config"))
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, "$.toolConfig.functionCallingConfig"))?;
    let object = required_object(Some(config), "$.toolConfig.functionCallingConfig")?;
    match object.get("mode").and_then(Value::as_str).unwrap_or("AUTO") {
        "AUTO" => Ok(Some(InternalToolChoice::Auto)),
        "NONE" => Ok(Some(InternalToolChoice::None)),
        "ANY" => Ok(object
            .get("allowedFunctionNames")
            .or_else(|| object.get("allowed_function_names"))
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(Value::as_str)
            .map(|name| InternalToolChoice::Tool(name.to_owned()))
            .or(Some(InternalToolChoice::Required))),
        _ => Err(FormatConversionError::invalid_payload(FORMAT, "$.toolConfig.functionCallingConfig.mode")),
    }
}

pub(super) fn tools_from_internal(tools: &[InternalTool]) -> Vec<Value> {
    vec![json!({
        "functionDeclarations": tools.iter().map(tool_from_internal).collect::<Vec<_>>(),
    })]
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
    })
}

fn tool_from_internal(tool: &InternalTool) -> Value {
    json!({
        "name": tool.name,
        "description": tool.description,
        "parameters": tool.parameters,
    })
}

fn required_text<'a>(object: &'a Map<String, Value>, path: &str, key: &str) -> Result<&'a str, FormatConversionError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, format!("{path}.{key}")))
}

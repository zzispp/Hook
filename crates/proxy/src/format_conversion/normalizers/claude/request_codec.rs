use serde_json::{Map, Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalMessage, InternalRole, InternalTool, InternalToolChoice};

use super::{
    common::{FORMAT, required_array, required_object},
    request_content::{claude_text_from_internal, content_blocks_from_claude, required_text},
};

pub(super) use super::request_content::content_from_internal;

pub(super) fn system_messages(request: &Value) -> Result<Vec<InternalMessage>, FormatConversionError> {
    match request.get("system") {
        Some(Value::String(text)) if !text.is_empty() => Ok(vec![InternalMessage::text(InternalRole::System, text)]),
        Some(Value::Array(blocks)) => Ok(vec![InternalMessage {
            role: InternalRole::System,
            content: content_blocks_from_claude(Some(&Value::Array(blocks.clone())), "$.system")?,
        }]),
        Some(_) => Err(FormatConversionError::invalid_payload(FORMAT, "$.system")),
        None => Ok(Vec::new()),
    }
}

pub(super) fn parse_messages(request: &Value) -> Result<Vec<InternalMessage>, FormatConversionError> {
    required_array(request, "messages", "$.messages")?
        .iter()
        .enumerate()
        .map(|(index, value)| parse_message(value, index))
        .collect()
}

pub(super) fn messages_from_internal(messages: &[InternalMessage]) -> Result<Vec<Value>, FormatConversionError> {
    super::message_grouping::messages_from_internal(messages)
}

pub(super) fn system_from_internal(messages: &[InternalMessage]) -> Result<Option<Value>, FormatConversionError> {
    let system_messages = messages
        .iter()
        .filter(|message| matches!(message.role, InternalRole::System | InternalRole::Developer))
        .collect::<Vec<_>>();
    if system_messages.is_empty() {
        return Ok(None);
    }
    if system_messages.iter().any(|message| message.content.iter().any(text_has_cache_control)) {
        let blocks = system_messages
            .iter()
            .flat_map(|message| message.content.iter())
            .filter_map(system_text_block)
            .collect::<Vec<_>>();
        return Ok((!blocks.is_empty()).then(|| Value::Array(blocks)));
    }
    let system = system_messages
        .iter()
        .map(|message| message.text_content())
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>();
    Ok((!system.is_empty()).then(|| Value::String(system.join("\n\n"))))
}

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
        Some(Value::Object(object)) => match object.get("type").and_then(Value::as_str).unwrap_or_default() {
            "auto" => Ok(Some(InternalToolChoice::Auto)),
            "none" => Ok(Some(InternalToolChoice::None)),
            "any" | "required" => Ok(Some(InternalToolChoice::Required)),
            "tool" | "tool_use" => Ok(object.get("name").and_then(Value::as_str).map(|name| InternalToolChoice::Tool(name.to_owned()))),
            _ => Ok(Some(InternalToolChoice::Auto)),
        },
        Some(_) => Ok(Some(InternalToolChoice::Auto)),
    }
}

pub(super) fn tools_from_internal(tools: &[InternalTool]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            if let Some(tool) = web_search_tool_from_internal(tool) {
                return tool;
            }
            json!({
                "name": tool.name,
                "description": tool.description,
                "input_schema": tool.parameters.clone().unwrap_or_else(empty_object_schema),
            })
        })
        .collect()
}

pub(super) fn web_search_tool_from_options(options: &Value) -> Value {
    let context_size = options.get("search_context_size").and_then(Value::as_str).unwrap_or("medium");
    let max_uses = web_search_max_uses(context_size);
    let mut output = Map::new();
    output.insert("type".into(), Value::String("web_search_20250305".into()));
    output.insert("name".into(), Value::String("web_search".into()));
    output.insert("max_uses".into(), Value::Number(max_uses.into()));
    if let Some(user_location) = options.get("user_location") {
        output.insert("user_location".into(), user_location.clone());
    }
    Value::Object(output)
}

pub(super) fn tool_choice_from_internal(choice: &InternalToolChoice) -> Value {
    match choice {
        InternalToolChoice::Auto => json!({ "type": "auto" }),
        InternalToolChoice::None => json!({ "type": "none" }),
        InternalToolChoice::Required => json!({ "type": "any" }),
        InternalToolChoice::Tool(name) => json!({ "type": "tool", "name": name }),
    }
}

pub(super) fn message_role(role: &InternalRole) -> &'static str {
    match role {
        InternalRole::Assistant => "assistant",
        _ => "user",
    }
}

fn text_has_cache_control(block: &InternalContentBlock) -> bool {
    matches!(block, InternalContentBlock::Text { cache_control: Some(_), .. })
}

fn system_text_block(block: &InternalContentBlock) -> Option<Value> {
    let InternalContentBlock::Text { text, cache_control } = block else {
        return None;
    };
    if text.is_empty() {
        return None;
    }
    Some(claude_text_from_internal(text, cache_control.as_ref()))
}

fn empty_object_schema() -> Value {
    json!({ "type": "object", "properties": {} })
}

fn web_search_tool_from_internal(tool: &InternalTool) -> Option<Value> {
    Some(web_search_tool_from_options(tool.extra.get("web_search_options")?))
}

fn web_search_max_uses(context_size: &str) -> u64 {
    match context_size {
        "low" => 1,
        "high" => 10,
        _ => 5,
    }
}

fn parse_message(value: &Value, index: usize) -> Result<InternalMessage, FormatConversionError> {
    let object = required_object(Some(value), &format!("$.messages[{index}]"))?;
    let role = claude_role(object.get("role").and_then(Value::as_str).unwrap_or("user"));
    Ok(InternalMessage {
        role,
        content: content_blocks_from_claude(object.get("content"), &format!("$.messages[{index}].content"))?,
    })
}

fn parse_tool(value: (usize, &Value)) -> Result<InternalTool, FormatConversionError> {
    let (index, value) = value;
    let object = required_object(Some(value), &format!("$.tools[{index}]"))?;
    Ok(InternalTool {
        name: required_text(object, &format!("$.tools[{index}]"), "name")?.to_owned(),
        description: object.get("description").and_then(Value::as_str).map(str::to_owned),
        parameters: object.get("input_schema").cloned(),
        extra: Map::new(),
    })
}

fn claude_role(value: &str) -> InternalRole {
    if value == "assistant" { InternalRole::Assistant } else { InternalRole::User }
}

use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalResponse, InternalUsage};

use super::common::{
    first_choice, map_openai_stop_reason, openai_finish_reason, optional_string, optional_string_value, parse_content, required_object, usage_from_openai,
};

pub fn to_internal(response: &Value) -> Result<InternalResponse, FormatConversionError> {
    let choice = first_choice(response, "$.choices")?;
    let message = required_object(choice.get("message"), "$.choices[0].message")?;
    let finish_reason = optional_string_value(choice.get("finish_reason")).map(|value| map_openai_stop_reason(&value));
    let mut content = response_content(message)?;
    content.extend(response_tool_calls(message)?);
    InternalResponse::new(
        optional_string(response, "id"),
        optional_string(response, "model").unwrap_or_else(|| "openai-unknown".to_owned()),
        content,
        finish_reason,
        usage_from_openai(response.get("usage")),
    )
}

pub fn from_internal(internal: &InternalResponse) -> Result<Value, FormatConversionError> {
    let (reasoning, content, tool_calls) = response_message_fields(&internal.content)?;
    let mut payload = json!({
        "id": openai_chat_completion_id(internal.id.as_deref()),
        "object": "chat.completion",
        "created": 0,
        "model": internal.model,
        "system_fingerprint": null,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": content,
            },
            "finish_reason": internal.finish_reason.as_ref().map(openai_finish_reason).unwrap_or("stop"),
        }]
    });
    if let Some(reasoning) = reasoning {
        payload["choices"][0]["message"]["reasoning_content"] = Value::String(reasoning);
    }
    if !tool_calls.is_empty() {
        payload["choices"][0]["message"]["tool_calls"] = Value::Array(tool_calls);
    }
    if let Some(usage) = internal.usage.clone().map(InternalUsage::with_total) {
        payload["usage"] = usage_json(&usage);
    }
    Ok(payload)
}

fn openai_chat_completion_id(id: Option<&str>) -> String {
    let Some(id) = id.filter(|value| !value.is_empty()) else {
        return "chatcmpl-unknown".to_owned();
    };
    if id.starts_with("chatcmpl-") {
        return id.to_owned();
    }
    if let Some(rest) = id.strip_prefix("resp_") {
        return format!("chatcmpl-{rest}");
    }
    id.to_owned()
}

fn response_message_fields(blocks: &[InternalContentBlock]) -> Result<(Option<String>, Value, Vec<Value>), FormatConversionError> {
    let reasoning = thinking_text(blocks);
    let content_blocks = blocks
        .iter()
        .filter(|block| !matches!(block, InternalContentBlock::Thinking { .. } | InternalContentBlock::ToolUse { .. }));
    let content = content_text(content_blocks)?;
    let tool_calls = tool_calls_from_blocks(blocks)?;
    let content_value = if content.is_empty() && !tool_calls.is_empty() {
        Value::Null
    } else {
        Value::String(content)
    };
    Ok(((!reasoning.is_empty()).then_some(reasoning), content_value, tool_calls))
}

fn content_text<'a>(blocks: impl Iterator<Item = &'a InternalContentBlock>) -> Result<String, FormatConversionError> {
    let mut output = String::new();
    for block in blocks {
        match block {
            InternalContentBlock::Text { text, .. } => output.push_str(text),
            InternalContentBlock::ToolResult { .. } => {}
            _ => {
                return Err(FormatConversionError::unsupported_content(
                    super::common::FORMAT,
                    "content block cannot be represented in OpenAI Chat response",
                ));
            }
        }
    }
    Ok(output)
}

fn thinking_text(blocks: &[InternalContentBlock]) -> String {
    blocks
        .iter()
        .filter_map(|block| match block {
            InternalContentBlock::Thinking { text, .. } if !text.is_empty() => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
}

fn tool_calls_from_blocks(blocks: &[InternalContentBlock]) -> Result<Vec<Value>, FormatConversionError> {
    blocks
        .iter()
        .filter_map(|block| match block {
            InternalContentBlock::ToolUse { id, name, input } => Some(tool_call_json(id, name, input)),
            _ => None,
        })
        .collect()
}

fn tool_call_json(id: &str, name: &str, input: &Value) -> Result<Value, FormatConversionError> {
    Ok(json!({
        "id": id,
        "type": "function",
        "function": {
            "name": name,
            "arguments": serde_json::to_string(input).map_err(|error| FormatConversionError::invalid_payload(super::common::FORMAT, error.to_string()))?,
        },
    }))
}

fn usage_json(usage: &InternalUsage) -> Value {
    let mut payload = json!({
        "prompt_tokens": usage.prompt_tokens,
        "completion_tokens": usage.completion_tokens,
        "total_tokens": usage.total_tokens,
    });
    if usage.cache_read_tokens.is_some() || usage.cache_creation_tokens.is_some() {
        payload["prompt_tokens_details"] = json!({
            "cached_tokens": usage.cache_read_tokens.unwrap_or_default(),
            "cache_creation_tokens": usage.cache_creation_tokens.unwrap_or_default(),
        });
    }
    if let Some(reasoning_tokens) = usage.reasoning_tokens {
        payload["completion_tokens_details"] = json!({ "reasoning_tokens": reasoning_tokens });
    }
    payload
}

fn response_content(message: &serde_json::Map<String, Value>) -> Result<Vec<InternalContentBlock>, FormatConversionError> {
    let mut blocks = Vec::new();
    if let Some(reasoning) = message.get("reasoning_content").and_then(Value::as_str) {
        blocks.push(InternalContentBlock::Thinking {
            text: reasoning.to_owned(),
            signature: None,
        });
    }
    let text = parse_content(message.get("content"), "$.choices[0].message.content")?;
    if !text.is_empty() {
        blocks.push(InternalContentBlock::text(text));
    }
    Ok(blocks)
}

fn response_tool_calls(message: &serde_json::Map<String, Value>) -> Result<Vec<InternalContentBlock>, FormatConversionError> {
    let Some(calls) = message.get("tool_calls").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    calls.iter().map(tool_call_block).collect()
}

fn tool_call_block(value: &Value) -> Result<InternalContentBlock, FormatConversionError> {
    let object = required_object(Some(value), "$.choices[0].message.tool_calls[]")?;
    let function = required_object(object.get("function"), "$.choices[0].message.tool_calls[].function")?;
    let input = function_arguments(function.get("arguments"))?;
    Ok(InternalContentBlock::ToolUse {
        id: object.get("id").and_then(Value::as_str).unwrap_or_default().to_owned(),
        name: function.get("name").and_then(Value::as_str).unwrap_or_default().to_owned(),
        input,
    })
}

fn function_arguments(value: Option<&Value>) -> Result<Value, FormatConversionError> {
    value
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(|text| {
            serde_json::from_str(text)
                .map(|parsed| match parsed {
                    Value::Object(_) => parsed,
                    other => json!({ "raw": other }),
                })
                .or_else(|_| Ok(json!({ "raw": text })))
        })
        .transpose()
        .map(|value| value.unwrap_or_else(|| json!({})))
}

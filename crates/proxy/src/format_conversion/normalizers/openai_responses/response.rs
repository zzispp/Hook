use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalResponse, InternalUsage};

const FORMAT: &str = "openai_responses";

pub fn to_internal(response: &Value) -> Result<InternalResponse, FormatConversionError> {
    InternalResponse::new(
        optional_string(response, "id"),
        optional_string(response, "model").unwrap_or_else(|| "openai-responses-unknown".to_owned()),
        response_content(response)?,
        None,
        usage_from_response(response.get("usage")),
    )
}

pub fn from_internal(internal: &InternalResponse) -> Result<Value, FormatConversionError> {
    let output_text = internal.text.clone();
    let mut payload = json!({
        "id": internal.id.clone().unwrap_or_else(|| "resp_unknown".to_owned()),
        "object": "response",
        "created_at": 0,
        "status": "completed",
        "model": internal.model,
        "output": [{
            "type": "message",
            "role": "assistant",
            "content": [{
                "type": "output_text",
                "text": output_text,
            }],
        }],
        "output_text": internal.text,
    });
    if let Some(usage) = internal.usage.clone().map(InternalUsage::with_total) {
        payload["usage"] = usage_json(&usage);
    }
    Ok(payload)
}

fn response_content(response: &Value) -> Result<Vec<InternalContentBlock>, FormatConversionError> {
    if let Some(text) = response.get("output_text").and_then(Value::as_str) {
        return Ok(vec![InternalContentBlock::text(text.to_owned())]);
    }
    output_blocks(response.get("output"))
}

fn output_blocks(value: Option<&Value>) -> Result<Vec<InternalContentBlock>, FormatConversionError> {
    let items = value
        .and_then(Value::as_array)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, "$.output"))?;
    let mut output = Vec::new();
    for item in items {
        append_output_item_blocks(item, &mut output)?;
    }
    Ok(output)
}

fn append_output_item_blocks(item: &Value, output: &mut Vec<InternalContentBlock>) -> Result<(), FormatConversionError> {
    match item.get("type").and_then(Value::as_str).unwrap_or_default() {
        "message" => append_message_content(item, output),
        "function_call" => {
            output.push(function_call_block(item)?);
            Ok(())
        }
        "reasoning" => {
            append_reasoning_content(item, output);
            Ok(())
        }
        _ => Ok(()),
    }
}

fn append_message_content(item: &Value, output: &mut Vec<InternalContentBlock>) -> Result<(), FormatConversionError> {
    let Some(content) = item.get("content").and_then(Value::as_array) else {
        return Ok(());
    };
    for block in content {
        output.push(output_content_block(block)?);
    }
    Ok(())
}

fn output_content_block(block: &Value) -> Result<InternalContentBlock, FormatConversionError> {
    let block_type = block.get("type").and_then(Value::as_str).unwrap_or_default();
    if matches!(block_type, "output_text" | "text") {
        let text = block
            .get("text")
            .and_then(Value::as_str)
            .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, "$.output.content.text"))?;
        return Ok(InternalContentBlock::text(text.to_owned()));
    }
    Err(FormatConversionError::unsupported_content(
        FORMAT,
        format!("unsupported output content block type {block_type}"),
    ))
}

fn function_call_block(item: &Value) -> Result<InternalContentBlock, FormatConversionError> {
    let input = arguments_json(item.get("arguments"))?;
    Ok(InternalContentBlock::ToolUse {
        id: item.get("call_id").and_then(Value::as_str).unwrap_or_default().to_owned(),
        name: item.get("name").and_then(Value::as_str).unwrap_or_default().to_owned(),
        input,
    })
}

fn arguments_json(value: Option<&Value>) -> Result<Value, FormatConversionError> {
    value
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(|text| {
            serde_json::from_str(text)
                .map(|parsed| match parsed {
                    Value::Object(_) => parsed,
                    other => json!({ "_raw": other }),
                })
                .or_else(|_| Ok(json!({ "_raw": text })))
        })
        .transpose()
        .map(|value| value.unwrap_or_else(|| json!({})))
}

fn append_reasoning_content(item: &Value, output: &mut Vec<InternalContentBlock>) {
    let text = item
        .get("summary")
        .and_then(Value::as_array)
        .map(|items| reasoning_summary_text(items))
        .unwrap_or_default();
    if !text.is_empty() {
        output.push(InternalContentBlock::Thinking { text, signature: None });
    }
}

fn reasoning_summary_text(items: &[Value]) -> String {
    items
        .iter()
        .filter_map(|item| item.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("")
}

pub(super) fn usage_from_response(value: Option<&Value>) -> Option<InternalUsage> {
    let object = value?.as_object()?;
    Some(
        InternalUsage {
            prompt_tokens: object.get("input_tokens").and_then(as_u32),
            completion_tokens: object.get("output_tokens").and_then(as_u32),
            total_tokens: object.get("total_tokens").and_then(as_u32),
            cache_read_tokens: object.get("input_tokens_details").and_then(|value| value.get("cached_tokens")).and_then(as_u32),
            cache_creation_tokens: object
                .get("input_tokens_details")
                .and_then(|value| value.get("cache_creation_tokens"))
                .and_then(as_u32),
            reasoning_tokens: object
                .get("output_tokens_details")
                .and_then(|value| value.get("reasoning_tokens"))
                .and_then(as_u32),
        }
        .with_total(),
    )
}

fn optional_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn as_u32(value: &Value) -> Option<u32> {
    value.as_u64().and_then(|item| u32::try_from(item).ok())
}

fn usage_json(usage: &InternalUsage) -> Value {
    let prompt_tokens = usage.prompt_tokens.unwrap_or_default();
    let completion_tokens = usage.completion_tokens.unwrap_or_default();
    let mut output = json!({
        "input_tokens": prompt_tokens,
        "output_tokens": completion_tokens,
        "total_tokens": usage.total_tokens.unwrap_or(prompt_tokens.saturating_add(completion_tokens)),
    });
    insert_usage_details(&mut output, usage);
    output
}

fn insert_usage_details(output: &mut Value, usage: &InternalUsage) {
    if usage.cache_read_tokens.is_some() || usage.cache_creation_tokens.is_some() {
        output["input_tokens_details"] = json!({
            "cached_tokens": usage.cache_read_tokens.unwrap_or_default(),
            "cache_creation_tokens": usage.cache_creation_tokens.unwrap_or_default(),
        });
    }
    if let Some(reasoning_tokens) = usage.reasoning_tokens {
        output["output_tokens_details"] = json!({ "reasoning_tokens": reasoning_tokens });
    }
}

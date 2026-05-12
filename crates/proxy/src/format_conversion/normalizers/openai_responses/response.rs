use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalResponse, InternalUsage};

const FORMAT: &str = "openai_responses";

pub fn to_internal(response: &Value) -> Result<InternalResponse, FormatConversionError> {
    Ok(InternalResponse {
        id: optional_string(response, "id"),
        model: optional_string(response, "model").unwrap_or_else(|| "openai-responses-unknown".to_owned()),
        text: response_text(response)?,
        finish_reason: None,
        usage: usage_from_response(response.get("usage")),
    })
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
        payload["usage"] = json!({
            "input_tokens": usage.prompt_tokens,
            "output_tokens": usage.completion_tokens,
            "total_tokens": usage.total_tokens,
        });
    }
    Ok(payload)
}

fn response_text(response: &Value) -> Result<String, FormatConversionError> {
    if let Some(text) = response.get("output_text").and_then(Value::as_str) {
        return Ok(text.to_owned());
    }
    output_text(response.get("output"))
}

fn output_text(value: Option<&Value>) -> Result<String, FormatConversionError> {
    let items = value
        .and_then(Value::as_array)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, "$.output"))?;
    let mut text = String::new();
    for item in items {
        append_output_item_text(item, &mut text)?;
    }
    Ok(text)
}

fn append_output_item_text(item: &Value, output: &mut String) -> Result<(), FormatConversionError> {
    let Some(content) = item.get("content").and_then(Value::as_array) else {
        return Ok(());
    };
    for block in content {
        let block_type = block.get("type").and_then(Value::as_str).unwrap_or_default();
        if matches!(block_type, "output_text" | "text") {
            output.push_str(
                block
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, "$.output.content.text"))?,
            );
        }
    }
    Ok(())
}

fn usage_from_response(value: Option<&Value>) -> Option<InternalUsage> {
    let object = value?.as_object()?;
    Some(
        InternalUsage {
            prompt_tokens: object.get("input_tokens").and_then(as_u32),
            completion_tokens: object.get("output_tokens").and_then(as_u32),
            total_tokens: object.get("total_tokens").and_then(as_u32),
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

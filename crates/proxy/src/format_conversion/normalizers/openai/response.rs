use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalResponse, InternalUsage};

use super::common::{
    first_choice, map_openai_stop_reason, openai_finish_reason, optional_string, optional_string_value, parse_content, required_object, usage_from_openai,
};

pub fn to_internal(response: &Value) -> Result<InternalResponse, FormatConversionError> {
    let choice = first_choice(response, "$.choices")?;
    let message = required_object(choice.get("message"), "$.choices[0].message")?;
    let finish_reason = optional_string_value(choice.get("finish_reason")).map(|value| map_openai_stop_reason(&value));
    Ok(InternalResponse {
        id: optional_string(response, "id"),
        model: optional_string(response, "model").unwrap_or_else(|| "openai-unknown".to_owned()),
        text: parse_content(message.get("content"), "$.choices[0].message.content")?,
        finish_reason,
        usage: usage_from_openai(response.get("usage")),
    })
}

pub fn from_internal(internal: &InternalResponse) -> Result<Value, FormatConversionError> {
    let mut payload = json!({
        "id": internal.id.clone().unwrap_or_else(|| "chatcmpl_unknown".to_owned()),
        "object": "chat.completion",
        "created": 0,
        "model": internal.model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": internal.text,
            },
            "finish_reason": internal.finish_reason.as_ref().map(openai_finish_reason).unwrap_or("stop"),
        }]
    });
    if let Some(usage) = internal.usage.clone().map(InternalUsage::with_total) {
        payload["usage"] = json!({
            "prompt_tokens": usage.prompt_tokens,
            "completion_tokens": usage.completion_tokens,
            "total_tokens": usage.total_tokens,
        });
    }
    Ok(payload)
}

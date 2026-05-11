use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalResponse};

use super::common::{claude_stop_reason, claude_usage, content_text, map_claude_stop_reason, optional_string, response_content, usage_from_claude};

pub fn to_internal(response: &Value) -> Result<InternalResponse, FormatConversionError> {
    let finish_reason = response.get("stop_reason").and_then(Value::as_str).map(map_claude_stop_reason);
    Ok(InternalResponse {
        id: optional_string(response, "id"),
        model: optional_string(response, "model").unwrap_or_else(|| "claude-unknown".to_owned()),
        text: content_text(response.get("content"), "$.content")?,
        finish_reason,
        usage: usage_from_claude(response.get("usage")),
    })
}

pub fn from_internal(internal: &InternalResponse) -> Result<Value, FormatConversionError> {
    let mut payload = json!({
        "id": internal.id.clone().unwrap_or_else(|| "msg_unknown".to_owned()),
        "type": "message",
        "role": "assistant",
        "model": internal.model,
        "content": response_content(&internal.text),
        "stop_reason": internal.finish_reason.as_ref().map(claude_stop_reason),
        "stop_sequence": null,
    });
    if let Some(usage) = &internal.usage {
        payload["usage"] = claude_usage(usage);
    }
    Ok(payload)
}

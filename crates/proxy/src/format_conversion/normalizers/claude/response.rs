use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalResponse};

use super::{
    common::{claude_stop_reason, claude_usage, empty_claude_usage, map_claude_stop_reason, optional_string, usage_from_claude},
    request_codec::{content_blocks_from_claude, content_from_internal},
};

pub fn to_internal(response: &Value) -> Result<InternalResponse, FormatConversionError> {
    let finish_reason = response.get("stop_reason").and_then(Value::as_str).map(map_claude_stop_reason);
    InternalResponse::new(
        optional_string(response, "id"),
        optional_string(response, "model").unwrap_or_else(|| "claude-unknown".to_owned()),
        content_blocks_from_claude(response.get("content"), "$.content")?,
        finish_reason,
        usage_from_claude(response.get("usage")),
    )
}

pub fn from_internal(internal: &InternalResponse) -> Result<Value, FormatConversionError> {
    let mut payload = json!({
        "id": claude_message_id(internal.id.as_deref()),
        "type": "message",
        "role": "assistant",
        "model": internal.model,
        "content": content_from_internal(&internal.content)?,
        "stop_reason": internal.finish_reason.as_ref().map(claude_stop_reason),
        "stop_sequence": null,
    });
    payload["usage"] = internal.usage.as_ref().map(claude_usage).unwrap_or_else(empty_claude_usage);
    Ok(payload)
}

fn claude_message_id(id: Option<&str>) -> String {
    let id = id.filter(|value| !value.is_empty()).unwrap_or("unknown");
    if id.starts_with("msg_") {
        return id.to_owned();
    }
    format!("msg_{id}")
}

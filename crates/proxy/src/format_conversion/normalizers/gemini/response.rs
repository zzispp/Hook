use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalResponse};

use super::{
    common::{map_gemini_stop_reason, optional_string, required_array, required_object, terminal_chunk, usage_from_gemini},
    request_codec::{parts_from_gemini, parts_from_internal},
};

pub fn to_internal(response: &Value) -> Result<InternalResponse, FormatConversionError> {
    let candidate = first_candidate(response)?;
    let content = required_object(candidate.get("content"), "$.candidates[0].content")?;
    let finish_reason = candidate.get("finishReason").and_then(Value::as_str).map(map_gemini_stop_reason);
    InternalResponse::new(
        optional_string(response, "id"),
        optional_string(response, "modelVersion").unwrap_or_else(|| "gemini-unknown".to_owned()),
        parts_from_gemini(content.get("parts"), "$.candidates[0].content.parts")?,
        finish_reason,
        usage_from_gemini(response.get("usageMetadata")),
    )
}

pub fn from_internal(internal: &InternalResponse) -> Result<Value, FormatConversionError> {
    let mut payload = terminal_chunk(&internal.model, internal.finish_reason.as_ref(), internal.usage.as_ref());
    payload["candidates"][0]["content"] = json!({
        "role": "model",
        "parts": parts_from_internal(&internal.content)?,
    });
    if let Some(id) = &internal.id {
        payload["id"] = json!(id);
    }
    Ok(payload)
}

fn first_candidate(value: &Value) -> Result<&serde_json::Map<String, Value>, FormatConversionError> {
    let candidates = required_array(value, "candidates", "$.candidates")?;
    let first = candidates
        .first()
        .ok_or_else(|| FormatConversionError::invalid_payload("gemini", "$.candidates[0]"))?;
    required_object(Some(first), "$.candidates[0]")
}

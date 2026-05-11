use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalResponse};

use super::common::{content_chunk, map_gemini_stop_reason, optional_string, parts_text, required_array, required_object, usage_from_gemini};

pub fn to_internal(response: &Value) -> Result<InternalResponse, FormatConversionError> {
    let candidate = first_candidate(response)?;
    let content = required_object(candidate.get("content"), "$.candidates[0].content")?;
    let finish_reason = candidate.get("finishReason").and_then(Value::as_str).map(map_gemini_stop_reason);
    Ok(InternalResponse {
        id: optional_string(response, "id"),
        model: optional_string(response, "modelVersion").unwrap_or_else(|| "gemini-unknown".to_owned()),
        text: parts_text(content.get("parts"), "$.candidates[0].content.parts")?,
        finish_reason,
        usage: usage_from_gemini(response.get("usageMetadata")),
    })
}

pub fn from_internal(internal: &InternalResponse) -> Result<Value, FormatConversionError> {
    let mut payload = content_chunk(&internal.text, &internal.model, internal.finish_reason.as_ref(), internal.usage.as_ref());
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

use serde_json::{Value, json};

use crate::format_conversion::FormatConversionError;

use super::request_fields::FORMAT;

pub(super) fn arguments_json(value: Option<&Value>) -> Result<Value, FormatConversionError> {
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

pub(super) fn custom_tool_input(value: Option<&Value>) -> Value {
    let input = value.and_then(Value::as_str).unwrap_or_default();
    json!({ "_raw": input })
}

pub(super) fn custom_tool_input_text(input: &Value) -> Result<String, FormatConversionError> {
    if let Some(text) = input.get("_raw").and_then(Value::as_str) {
        return Ok(text.to_owned());
    }
    serde_json::to_string(input).map_err(|error| FormatConversionError::invalid_payload(FORMAT, error.to_string()))
}

use serde_json::{Map, Value};

use crate::format_conversion::FormatConversionError;

pub(super) const FORMAT: &str = "openai_responses";

pub(super) fn string_field(value: &Value, key: &str, path: &str) -> Result<String, FormatConversionError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| FormatConversionError::invalid_payload(FORMAT, path))
}

pub(super) fn number_field(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

pub(super) fn u32_field(value: &Value, key: &str) -> Option<u32> {
    value.get(key).and_then(Value::as_u64).and_then(|item| u32::try_from(item).ok())
}

pub(super) fn bool_field(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

pub(super) fn insert_number(map: &mut Map<String, Value>, key: &str, value: Option<f64>) {
    if let Some(number) = value.and_then(serde_json::Number::from_f64) {
        map.insert(key.to_owned(), Value::Number(number));
    }
}

pub(super) fn insert_integer(map: &mut Map<String, Value>, key: &str, value: Option<u32>) {
    if let Some(number) = value {
        map.insert(key.to_owned(), Value::Number(serde_json::Number::from(number)));
    }
}

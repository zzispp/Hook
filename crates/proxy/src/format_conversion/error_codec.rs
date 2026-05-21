use serde_json::{Value, json};

use super::InternalError;

pub(crate) fn to_internal(value: &Value, status: Option<u16>) -> InternalError {
    InternalError {
        message: value_message(value).unwrap_or_else(|| "upstream error".to_owned()),
        error_type: value_string(value, &["type", "error_type"]),
        code: value_string(value, &["code", "error_code"]),
        param: value_string(value, &["param", "parameter", "field"]),
        status,
    }
}

pub(crate) fn openai_error(error: &InternalError) -> Value {
    json!({ "error": error_payload(error) })
}

pub(crate) fn claude_error(error: &InternalError) -> Value {
    json!({ "type": "error", "error": error_payload(error) })
}

pub(crate) fn gemini_error(error: &InternalError) -> Value {
    json!({ "error": {
        "message": error.message,
        "status": error.code,
        "code": error.status,
    } })
}

fn error_payload(error: &InternalError) -> Value {
    json!({
        "message": error.message,
        "type": error.error_type,
        "code": error.code,
        "param": error.param,
    })
}

fn value_message(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Array(items) => items.iter().find_map(value_message),
        Value::Object(map) => direct_string(map, &["message", "error_description", "detail", "title"])
            .or_else(|| nested_message(map, "error"))
            .or_else(|| nested_message(map, "detail"))
            .or_else(|| nested_message(map, "details"))
            .or_else(|| nested_message(map, "errors")),
        _ => None,
    }
}

fn value_string(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Array(items) => items.iter().find_map(|item| value_string(item, keys)),
        Value::Object(map) => nested_string(map, "error", keys)
            .or_else(|| nested_string(map, "detail", keys))
            .or_else(|| nested_string(map, "details", keys))
            .or_else(|| nested_string(map, "errors", keys))
            .or_else(|| direct_string(map, keys)),
        _ => None,
    }
}

fn direct_string(map: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| map.get(*key).and_then(value_message))
}

fn nested_message(map: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    map.get(key).and_then(value_message)
}

fn nested_string(map: &serde_json::Map<String, Value>, key: &str, keys: &[&str]) -> Option<String> {
    map.get(key).and_then(|value| value_string(value, keys))
}

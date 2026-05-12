use serde_json::Value;

pub(super) fn body_value(bytes: &[u8]) -> Value {
    match serde_json::from_slice(bytes) {
        Ok(value) => value,
        Err(_) => text_or_bytes(bytes),
    }
}

fn text_or_bytes(bytes: &[u8]) -> Value {
    match String::from_utf8(bytes.to_vec()) {
        Ok(text) => Value::String(text),
        Err(_) => Value::Array(bytes.iter().copied().map(Value::from).collect()),
    }
}

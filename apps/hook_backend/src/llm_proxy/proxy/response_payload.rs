use serde_json::Value;

pub(super) struct UpstreamStatusErrorDetails {
    pub(super) message: String,
    pub(super) code: Option<String>,
    pub(super) param: Option<String>,
}

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

pub(super) fn upstream_status_error_details(status_code: u16, bytes: &[u8]) -> UpstreamStatusErrorDetails {
    let prefix = format!("upstream returned status {status_code}");
    let body = body_value(bytes);
    let message = match error_message_fragment(&body) {
        Some(message) => format!("{prefix}: {message}"),
        None => prefix,
    };
    UpstreamStatusErrorDetails {
        message,
        code: error_code_fragment(&body),
        param: error_param_fragment(&body),
    }
}

fn error_message_fragment(value: &Value) -> Option<String> {
    let message = value_message(value)?;
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(truncate_message(trimmed))
}

fn error_code_fragment(value: &Value) -> Option<String> {
    string_fragment(value, &["code", "error_code"])
}

fn error_param_fragment(value: &Value) -> Option<String> {
    string_fragment(value, &["param", "parameter", "field"])
}

fn string_fragment(value: &Value, keys: &[&str]) -> Option<String> {
    let text = value_string(value, keys)?;
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| truncate_message(trimmed))
}

fn value_message(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Array(items) => items.iter().find_map(value_message),
        Value::Object(map) => direct_message(map)
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
        Value::Object(map) => direct_string(map, keys)
            .or_else(|| nested_string(map, "error", keys))
            .or_else(|| nested_string(map, "detail", keys))
            .or_else(|| nested_string(map, "details", keys))
            .or_else(|| nested_string(map, "errors", keys)),
        _ => None,
    }
}

fn direct_message(map: &serde_json::Map<String, Value>) -> Option<String> {
    ["message", "error_description", "detail", "title"]
        .into_iter()
        .find_map(|key| map.get(key).and_then(value_message))
}

fn nested_message(map: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    map.get(key).and_then(value_message)
}

fn direct_string(map: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| map.get(*key).and_then(value_message))
}

fn nested_string(map: &serde_json::Map<String, Value>, key: &str, keys: &[&str]) -> Option<String> {
    map.get(key).and_then(|value| value_string(value, keys))
}

fn truncate_message(value: &str) -> String {
    const MAX_ERROR_MESSAGE_CHARS: usize = 240;
    let mut end = MAX_ERROR_MESSAGE_CHARS.min(value.len());
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    if end == value.len() {
        return value.to_owned();
    }
    format!("{}...", &value[..end])
}

#[cfg(test)]
mod tests {
    use super::upstream_status_error_details;

    #[test]
    fn upstream_status_error_message_reads_nested_json_error() {
        let bytes = br#"{"error":{"message":"quota exceeded","type":"insufficient_quota"}}"#;
        assert_eq!(
            upstream_status_error_details(429, bytes).message,
            "upstream returned status 429: quota exceeded"
        );
    }

    #[test]
    fn upstream_status_error_details_extracts_code_and_param() {
        let bytes = br#"{"error":{"message":"quota exceeded","code":"insufficient_quota","param":"model"}}"#;
        let details = upstream_status_error_details(429, bytes);
        assert_eq!(details.message, "upstream returned status 429: quota exceeded");
        assert_eq!(details.code.as_deref(), Some("insufficient_quota"));
        assert_eq!(details.param.as_deref(), Some("model"));
    }

    #[test]
    fn upstream_status_error_message_reads_plain_text_body() {
        let bytes = b"provider overloaded";
        assert_eq!(
            upstream_status_error_details(503, bytes).message,
            "upstream returned status 503: provider overloaded"
        );
    }
}

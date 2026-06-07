use serde_json::Value;

const MAX_ERROR_MESSAGE_CHARS: usize = 240;
const MAX_ERROR_CODE_CHARS: usize = 120;
const MAX_ERROR_PARAM_CHARS: usize = 160;
const TRUNCATION_MARKER: &str = "...";

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
    string_fragment(value, &["code", "error_code"], MAX_ERROR_CODE_CHARS)
}

fn error_param_fragment(value: &Value) -> Option<String> {
    string_fragment(value, &["param", "parameter", "field"], MAX_ERROR_PARAM_CHARS)
}

fn string_fragment(value: &Value, keys: &[&str], max_chars: usize) -> Option<String> {
    let text = value_string(value, keys)?;
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| truncate_fragment(trimmed, max_chars))
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
    keys.iter().find_map(|key| map.get(*key).and_then(Value::as_str).map(str::to_owned))
}

fn nested_string(map: &serde_json::Map<String, Value>, key: &str, keys: &[&str]) -> Option<String> {
    map.get(key).and_then(|value| value_string(value, keys))
}

fn truncate_message(value: &str) -> String {
    truncate_fragment(value, MAX_ERROR_MESSAGE_CHARS)
}

fn truncate_fragment(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_owned();
    }
    let marker_chars = TRUNCATION_MARKER.chars().count();
    let prefix_chars = max_chars.saturating_sub(marker_chars);
    let end = byte_index_after_chars(value, prefix_chars);
    format!("{}{}", &value[..end], TRUNCATION_MARKER)
}

fn byte_index_after_chars(value: &str, char_count: usize) -> usize {
    value.char_indices().nth(char_count).map(|(index, _)| index).unwrap_or(value.len())
}

#[cfg(test)]
mod tests {
    use super::{MAX_ERROR_CODE_CHARS, MAX_ERROR_PARAM_CHARS, TRUNCATION_MARKER, upstream_status_error_details};

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
        let details = upstream_status_error_details(503, bytes);
        assert_eq!(details.message, "upstream returned status 503: provider overloaded");
        assert_eq!(details.code, None);
        assert_eq!(details.param, None);
    }

    #[test]
    fn upstream_status_error_details_truncates_code_and_param_to_column_limits() {
        let body = serde_json::json!({
            "error": {
                "message": "quota exceeded",
                "code": "x".repeat(MAX_ERROR_CODE_CHARS + 20),
                "param": "y".repeat(MAX_ERROR_PARAM_CHARS + 20)
            }
        })
        .to_string();
        let details = upstream_status_error_details(429, body.as_bytes());
        let code = details.code.expect("structured code should be captured");
        let param = details.param.expect("structured param should be captured");
        assert_eq!(code.chars().count(), MAX_ERROR_CODE_CHARS);
        assert_eq!(param.chars().count(), MAX_ERROR_PARAM_CHARS);
        assert!(code.ends_with(TRUNCATION_MARKER));
        assert!(param.ends_with(TRUNCATION_MARKER));
    }
}

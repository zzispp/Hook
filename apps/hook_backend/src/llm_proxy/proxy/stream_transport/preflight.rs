use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StreamPreflightError {
    pub(super) error_type: &'static str,
    pub(super) message: String,
}

pub(super) fn inspect_provider_error(bytes: &[u8]) -> Option<StreamPreflightError> {
    inspect_json(bytes).or_else(|| inspect_sse_lines(bytes))
}

fn inspect_json(bytes: &[u8]) -> Option<StreamPreflightError> {
    let stripped = strip_ws(bytes);
    if !matches!(stripped.first(), Some(b'{') | Some(b'[')) {
        return None;
    }
    let value = serde_json::from_slice::<Value>(stripped).ok()?;
    error_from_value(&value)
}

fn inspect_sse_lines(bytes: &[u8]) -> Option<StreamPreflightError> {
    let text = std::str::from_utf8(bytes).ok()?;
    for line in text.lines() {
        let payload = line.trim().strip_prefix("data:").map(str::trim);
        let Some(payload) = payload.filter(|value| !value.is_empty() && *value != "[DONE]") else {
            continue;
        };
        let value = serde_json::from_str::<Value>(payload).ok()?;
        if let Some(error) = error_from_value(&value) {
            return Some(error);
        }
    }
    None
}

fn error_from_value(value: &Value) -> Option<StreamPreflightError> {
    let error = nested_error(value)?;
    Some(StreamPreflightError {
        error_type: "upstream_stream_error",
        message: error_message(error).unwrap_or_else(|| "upstream stream returned an error payload".into()),
    })
}

fn nested_error(value: &Value) -> Option<&Value> {
    match value {
        Value::Object(map) => map.get("error").or_else(|| map.get("errors")).or_else(|| map.get("detail")),
        Value::Array(items) => items.iter().find_map(nested_error),
        _ => None,
    }
}

fn error_message(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => non_empty(text),
        Value::Object(map) => ["message", "detail", "reason", "type", "code"]
            .into_iter()
            .find_map(|key| map.get(key).and_then(error_message)),
        Value::Array(items) => items.iter().find_map(error_message),
        _ => None,
    }
}

fn non_empty(text: &str) -> Option<String> {
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

fn strip_ws(bytes: &[u8]) -> &[u8] {
    let start = bytes.iter().position(|byte| !byte.is_ascii_whitespace()).unwrap_or(bytes.len());
    &bytes[start..]
}

#[cfg(test)]
mod tests {
    use super::inspect_provider_error;

    #[test]
    fn detects_json_error_body() {
        let error = inspect_provider_error(br#"{"error":{"message":"bad upstream"}}"#).expect("error");

        assert_eq!(error.error_type, "upstream_stream_error");
        assert_eq!(error.message, "bad upstream");
    }

    #[test]
    fn detects_sse_error_frame() {
        let error = inspect_provider_error(b"data: {\"error\":{\"message\":\"bad stream\"}}\n\n").expect("error");

        assert_eq!(error.message, "bad stream");
    }

    #[test]
    fn ignores_normal_sse_frame() {
        assert!(inspect_provider_error(b"data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\n").is_none());
    }
}

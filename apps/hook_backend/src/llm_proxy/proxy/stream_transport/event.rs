use std::io;

use axum::body::Bytes;
use proxy::format_conversion::ApiFormat;
use serde_json::{Value, json};

pub(super) fn parse_stream_values(bytes: &[u8]) -> Vec<Value> {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| line.strip_prefix("data:"))
        .map(str::trim)
        .filter(|payload| !payload.is_empty() && *payload != "[DONE]")
        .filter_map(|payload| serde_json::from_str(payload).ok())
        .collect()
}

pub(super) fn render_stream_event(event: &Value, source_format: ApiFormat) -> Bytes {
    let mut output = String::new();
    if matches!(source_format, ApiFormat::ClaudeChat) {
        if let Some(event_type) = event.get("type").and_then(Value::as_str) {
            output.push_str("event: ");
            output.push_str(event_type);
            output.push('\n');
        }
    }
    output.push_str("data: ");
    output.push_str(&event.to_string());
    output.push_str("\n\n");
    Bytes::from(output)
}

pub(super) fn render_stream_error(source_format: ApiFormat) -> Bytes {
    let payload = if matches!(source_format, ApiFormat::OpenAiChat | ApiFormat::OpenAiResponses) {
        json!({"error": {"message": "response format conversion failed", "type": "format_conversion_error"}})
    } else {
        json!({"type": "error", "error": {"type": "format_conversion_error", "message": "response format conversion failed"}})
    };
    render_stream_event(&payload, source_format)
}

pub(super) fn stream_error(message: String) -> io::Error {
    io::Error::other(message)
}

use std::io;

use axum::body::Bytes;
use proxy::format_conversion::ApiFormat;
use serde_json::{Value, json};

const SSE_KEEPALIVE: &[u8] = b": PING\n\n";

pub(super) fn render_keepalive() -> Bytes {
    Bytes::from_static(SSE_KEEPALIVE)
}

pub(super) fn render_stream_event(event: &Value, source_format: ApiFormat) -> Bytes {
    let mut output = String::new();
    if matches!(source_format, ApiFormat::ClaudeChat)
        && let Some(event_type) = event.get("type").and_then(Value::as_str)
    {
        output.push_str("event: ");
        output.push_str(event_type);
        output.push('\n');
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

#[cfg(test)]
mod tests {
    use super::render_keepalive;

    #[test]
    fn keepalive_is_sse_comment_frame() {
        assert_eq!(render_keepalive().as_ref(), b": PING\n\n");
    }
}

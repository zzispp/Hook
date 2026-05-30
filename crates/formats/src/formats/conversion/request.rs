//! Pairwise request conversion helpers.
//!
//! These helpers keep the call sites readable while delegating wire-format
//! parsing and emitting to `formats::<format>::request` through the registry's
//! canonical IR path.

use serde_json::Value;

use crate::formats::{context::FormatContext, registry};

pub fn convert_openai_chat_request_to_claude_request(body_json: &Value, mapped_model: &str, upstream_is_stream: bool) -> Option<Value> {
    registry::convert_request("openai:chat", "claude:messages", body_json, &request_context(mapped_model, upstream_is_stream)).ok()
}

pub fn convert_openai_chat_request_to_gemini_request(body_json: &Value, mapped_model: &str, upstream_is_stream: bool) -> Option<Value> {
    registry::convert_request(
        "openai:chat",
        "gemini:generate_content",
        body_json,
        &request_context(mapped_model, upstream_is_stream),
    )
    .ok()
}

pub fn convert_openai_chat_request_to_openai_responses_request(
    body_json: &Value,
    mapped_model: &str,
    upstream_is_stream: bool,
    compact: bool,
) -> Option<Value> {
    let target_format = if compact { "openai:responses:compact" } else { "openai:responses" };
    registry::convert_request("openai:chat", target_format, body_json, &request_context(mapped_model, upstream_is_stream)).ok()
}

pub fn normalize_openai_responses_request_to_openai_chat_request(body_json: &Value) -> Option<Value> {
    registry::convert_request("openai:responses", "openai:chat", body_json, &FormatContext::default()).ok()
}

pub fn normalize_claude_request_to_openai_chat_request(body_json: &Value) -> Option<Value> {
    registry::convert_request("claude:messages", "openai:chat", body_json, &FormatContext::default()).ok()
}

pub fn normalize_gemini_request_to_openai_chat_request(body_json: &Value, request_path: &str) -> Option<Value> {
    registry::convert_request(
        "gemini:generate_content",
        "openai:chat",
        body_json,
        &FormatContext::default().with_request_path(request_path),
    )
    .ok()
}

pub fn extract_openai_text_content(content: Option<&Value>) -> Option<String> {
    match content {
        None | Some(Value::Null) => Some(String::new()),
        Some(Value::String(text)) => Some(text.clone()),
        Some(Value::Array(parts)) => {
            let mut collected = Vec::new();
            for part in parts {
                let part_object = part.as_object()?;
                let part_type = part_object.get("type").and_then(Value::as_str).unwrap_or_default();
                if matches!(part_type, "text" | "input_text")
                    && let Some(text) = part_object.get("text").and_then(Value::as_str)
                    && !text.trim().is_empty()
                {
                    collected.push(text.to_string());
                }
            }
            Some(collected.join("\n"))
        }
        _ => None,
    }
}

pub fn parse_openai_tool_result_content(content: Option<&Value>) -> Value {
    match content {
        Some(Value::String(raw)) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                Value::String(String::new())
            } else {
                serde_json::from_str::<Value>(trimmed).unwrap_or_else(|_| Value::String(raw.clone()))
            }
        }
        Some(Value::Array(parts)) => {
            let texts = parts
                .iter()
                .filter_map(|part| {
                    part.as_object()
                        .and_then(|object| object.get("text"))
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                })
                .collect::<Vec<_>>();
            if texts.is_empty() {
                Value::Array(parts.clone())
            } else {
                Value::String(texts.join("\n"))
            }
        }
        Some(value) => value.clone(),
        None => Value::String(String::new()),
    }
}

fn request_context(mapped_model: &str, upstream_is_stream: bool) -> FormatContext {
    FormatContext::default()
        .with_mapped_model(mapped_model)
        .with_upstream_stream(upstream_is_stream)
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{
        convert_openai_chat_request_to_claude_request, convert_openai_chat_request_to_openai_responses_request, normalize_claude_request_to_openai_chat_request,
    };

    #[test]
    fn pairwise_request_helper_routes_through_registry() {
        let body = json!({
            "model": "gpt-source",
            "messages": [{"role": "user", "content": "hello"}],
        });

        let converted = convert_openai_chat_request_to_openai_responses_request(&body, "gpt-target", true, false).expect("responses request");

        assert_eq!(converted["model"], "gpt-target");
        assert_eq!(converted["stream"], true);
        assert_eq!(converted["input"][0]["type"], "message");
    }

    #[test]
    fn pairwise_request_helper_keeps_claude_shape() {
        let body = json!({
            "model": "gpt-source",
            "messages": [{"role": "user", "content": "hello"}],
        });

        let converted = convert_openai_chat_request_to_claude_request(&body, "claude-target", false).expect("claude request");

        assert_eq!(converted["model"], "claude-target");
        assert_eq!(converted["messages"][0]["role"], "user");
    }

    #[test]
    fn request_normalizer_uses_format_adapter() {
        let body = json!({
            "model": "claude-sonnet",
            "messages": [{"role": "user", "content": [{"type": "text", "text": "hello"}]}],
            "max_tokens": 128,
        });

        let converted = normalize_claude_request_to_openai_chat_request(&body).expect("openai chat request");

        assert_eq!(converted["model"], "claude-sonnet");
        assert_eq!(converted["messages"][0]["role"], "user");
        assert_eq!(converted["messages"][0]["content"], "hello");
    }

    #[test]
    fn request_normalizer_preserves_multiple_claude_tool_results() {
        let body = json!({
            "model": "claude-sonnet",
            "messages": [
                {
                    "role": "assistant",
                    "content": [
                        {
                            "type": "tool_use",
                            "id": "toolu_1",
                            "name": "lookup",
                            "input": {"query": "alpha"}
                        },
                        {
                            "type": "tool_use",
                            "id": "toolu_2",
                            "name": "lookup",
                            "input": {"query": "beta"}
                        }
                    ]
                },
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "tool_result",
                            "tool_use_id": "toolu_1",
                            "content": "alpha result"
                        },
                        {
                            "type": "tool_result",
                            "tool_use_id": "toolu_2",
                            "content": [{"type": "text", "text": "beta result"}]
                        }
                    ]
                }
            ],
            "max_tokens": 128,
        });

        let converted = normalize_claude_request_to_openai_chat_request(&body).expect("openai chat request");
        let messages = converted["messages"].as_array().expect("messages");

        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0]["role"], "assistant");
        assert_eq!(messages[0]["tool_calls"].as_array().unwrap().len(), 2);
        assert_eq!(messages[0]["tool_calls"][0]["id"], "toolu_1");
        assert_eq!(messages[0]["tool_calls"][1]["id"], "toolu_2");
        assert_eq!(messages[1]["role"], "tool");
        assert_eq!(messages[1]["tool_call_id"], "toolu_1");
        assert_eq!(messages[1]["content"], "alpha result");
        assert_eq!(messages[2]["role"], "tool");
        assert_eq!(messages[2]["tool_call_id"], "toolu_2");
        assert_eq!(messages[2]["content"], "beta result");
    }

    #[test]
    fn request_normalizer_preserves_claude_tool_result_order_around_text() {
        let body = json!({
            "model": "claude-sonnet",
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "text", "text": "before"},
                    {
                        "type": "tool_result",
                        "tool_use_id": "toolu_1",
                        "content": "first"
                    },
                    {"type": "text", "text": "between"},
                    {
                        "type": "tool_result",
                        "tool_use_id": "toolu_2",
                        "content": "second"
                    }
                ]
            }],
            "max_tokens": 128,
        });

        let converted = normalize_claude_request_to_openai_chat_request(&body).expect("openai chat request");
        let messages = converted["messages"].as_array().expect("messages");

        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"], "before");
        assert_eq!(messages[1]["role"], "tool");
        assert_eq!(messages[1]["tool_call_id"], "toolu_1");
        assert_eq!(messages[1]["content"], "first");
        assert_eq!(messages[2]["role"], "user");
        assert_eq!(messages[2]["content"], "between");
        assert_eq!(messages[3]["role"], "tool");
        assert_eq!(messages[3]["tool_call_id"], "toolu_2");
        assert_eq!(messages[3]["content"], "second");
    }

    #[test]
    fn request_normalizer_marks_claude_error_tool_result_string_and_object_content() {
        let object_result = json!({"code": "ENOENT", "message": "missing"});
        let body = json!({
            "model": "claude-sonnet",
            "messages": [{
                "role": "user",
                "content": [
                    {
                        "type": "tool_result",
                        "tool_use_id": "toolu_error_string",
                        "content": "lookup failed",
                        "is_error": true
                    },
                    {
                        "type": "tool_result",
                        "tool_use_id": "toolu_error_empty",
                        "content": "",
                        "is_error": true
                    },
                    {
                        "type": "tool_result",
                        "tool_use_id": "toolu_error_object",
                        "content": object_result,
                        "is_error": true
                    },
                    {
                        "type": "tool_result",
                        "tool_use_id": "toolu_ok",
                        "content": "still ok"
                    }
                ]
            }],
            "max_tokens": 128,
        });

        let converted = normalize_claude_request_to_openai_chat_request(&body).expect("openai chat request");
        let messages = converted["messages"].as_array().expect("messages");

        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0]["role"], "tool");
        assert_eq!(messages[0]["tool_call_id"], "toolu_error_string");
        assert_eq!(messages[0]["content"], "[tool error]\nlookup failed");

        assert_eq!(messages[1]["role"], "tool");
        assert_eq!(messages[1]["tool_call_id"], "toolu_error_empty");
        assert_eq!(messages[1]["content"], "[tool error]");

        assert_eq!(messages[2]["role"], "tool");
        assert_eq!(messages[2]["tool_call_id"], "toolu_error_object");
        let object_content = messages[2]["content"].as_str().expect("object content");
        let serialized_object = object_content.strip_prefix("[tool error]\n").expect("error prefix");
        assert_eq!(serde_json::from_str::<Value>(serialized_object).expect("serialized object"), object_result);

        assert_eq!(messages[3]["role"], "tool");
        assert_eq!(messages[3]["tool_call_id"], "toolu_ok");
        assert_eq!(messages[3]["content"], "still ok");
    }

    #[test]
    fn request_normalizer_marks_claude_error_tool_result_multipart_image_content() {
        let body = json!({
            "model": "claude-sonnet",
            "messages": [{
                "role": "user",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": "toolu_error_image",
                    "content": [
                        {"type": "text", "text": "preview"},
                        {
                            "type": "image",
                            "source": {
                                "type": "base64",
                                "media_type": "image/png",
                                "data": "aW1hZ2U="
                            }
                        }
                    ],
                    "is_error": true
                }]
            }],
            "max_tokens": 128,
        });

        let converted = normalize_claude_request_to_openai_chat_request(&body).expect("openai chat request");
        let messages = converted["messages"].as_array().expect("messages");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "tool");
        assert_eq!(messages[0]["tool_call_id"], "toolu_error_image");
        let content = messages[0]["content"].as_array().expect("multipart error content");
        assert_eq!(
            content.as_slice(),
            &[
                json!({"type": "text", "text": "[tool error]"}),
                json!({"type": "text", "text": "preview"}),
                json!({
                    "type": "image_url",
                    "image_url": {"url": "data:image/png;base64,aW1hZ2U="}
                }),
            ]
        );
    }

    #[test]
    fn request_normalizer_preserves_legal_openai_tool_content_for_claude_variants() {
        let anthropic_blocks = json!([
            {"type": "text", "text": "preview"},
            {
                "type": "image",
                "source": {
                    "type": "base64",
                    "media_type": "image/jpeg",
                    "data": "aGVsbG8="
                }
            },
            {
                "type": "image",
                "source": {
                    "type": "url",
                    "url": "https://example.com/image.jpg"
                }
            },
            {
                "type": "document",
                "source": {
                    "type": "base64",
                    "media_type": "application/pdf",
                    "data": "JVBERi0x"
                }
            },
            {
                "type": "document",
                "source": {
                    "type": "url",
                    "url": "https://example.com/report.pdf"
                }
            },
            {
                "type": "document",
                "source": {
                    "type": "text",
                    "media_type": "text/plain",
                    "data": "document body"
                }
            }
        ]);
        let object_result = json!({"answer": 42, "ok": true});
        let body = json!({
            "model": "claude-sonnet",
            "messages": [{
                "role": "user",
                "content": [
                    {
                        "type": "tool_result",
                        "tool_use_id": "toolu_object",
                        "content": object_result
                    },
                    {
                        "type": "tool_result",
                        "tool_use_id": "toolu_text_blocks",
                        "content": [
                            {"type": "text", "text": "line one"},
                            {"type": "text", "text": "line two"}
                        ]
                    },
                    {
                        "type": "tool_result",
                        "tool_use_id": "toolu_anthropic_blocks",
                        "content": anthropic_blocks
                    }
                ]
            }],
            "max_tokens": 128,
        });

        let converted = normalize_claude_request_to_openai_chat_request(&body).expect("openai chat request");
        let messages = converted["messages"].as_array().expect("messages");

        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0]["role"], "tool");
        assert_eq!(messages[0]["tool_call_id"], "toolu_object");
        let object_content = messages[0]["content"].as_str().expect("object content");
        assert_eq!(serde_json::from_str::<Value>(object_content).expect("serialized object"), object_result);

        assert_eq!(messages[1]["role"], "tool");
        assert_eq!(messages[1]["tool_call_id"], "toolu_text_blocks");
        assert_eq!(messages[1]["content"], "line one\n\nline two");

        assert_eq!(messages[2]["role"], "tool");
        assert_eq!(messages[2]["tool_call_id"], "toolu_anthropic_blocks");
        let block_content = messages[2]["content"].as_array().expect("multipart anthropic block content");
        assert_eq!(
            block_content.as_slice(),
            &[
                json!({"type": "text", "text": "preview"}),
                json!({
                    "type": "image_url",
                    "image_url": {"url": "data:image/jpeg;base64,aGVsbG8="}
                }),
                json!({
                    "type": "image_url",
                    "image_url": {"url": "https://example.com/image.jpg"}
                }),
                json!({
                    "type": "file",
                    "file": {"file_data": "data:application/pdf;base64,JVBERi0x"}
                }),
                json!({"type": "text", "text": "[File: https://example.com/report.pdf]"}),
                json!({
                    "type": "text",
                    "text": "[Claude tool_result document content omitted: text/plain]"
                }),
            ]
        );
        let block_content_json = Value::Array(block_content.clone()).to_string();
        assert!(!block_content_json.contains("\"source\""));
        assert!(!block_content_json.contains("document body"));
    }
}

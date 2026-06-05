use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::json;

#[test]
fn stream_conversion_maps_thinking_signature_tool_delta_and_usage() {
    let registry = FormatConversionRegistry;
    let claude = vec![
        json!({ "type": "message_start", "message": { "id": "msg_1", "model": "claude-sonnet" } }),
        json!({ "type": "content_block_delta", "index": 0, "delta": { "type": "thinking_delta", "thinking": "why" } }),
        json!({ "type": "content_block_delta", "index": 0, "delta": { "type": "signature_delta", "signature": "sig" } }),
        json!({ "type": "content_block_delta", "index": 1, "delta": { "type": "input_json_delta", "partial_json": "{\"q\"" } }),
        json!({ "type": "message_delta", "delta": { "stop_reason": "tool_use" }, "usage": { "input_tokens": 4, "output_tokens": 3, "cache_read_input_tokens": 2 } }),
    ];

    let openai = registry.convert_stream(&claude, ApiFormat::ClaudeChat, ApiFormat::OpenAiChat).unwrap();
    assert_eq!(openai[1]["choices"][0]["delta"]["reasoning_content"], "why");
    let tool_delta = openai
        .iter()
        .find(|event| event["choices"][0]["delta"]["tool_calls"][0]["function"]["arguments"] == "{\"q\"")
        .expect("tool argument delta should exist");
    assert_eq!(tool_delta["choices"][0]["delta"]["tool_calls"][0]["function"]["arguments"], "{\"q\"");
    let usage = openai.iter().find(|event| event.get("usage").is_some()).expect("usage chunk should exist");
    assert_eq!(usage["usage"]["prompt_tokens_details"]["cached_tokens"], 2);
}

#[test]
fn gemini_stream_output_omits_null_signature_and_usage_fields() {
    let registry = FormatConversionRegistry;
    let claude = vec![
        json!({ "type": "message_start", "message": { "id": "msg_1", "model": "claude-sonnet" } }),
        json!({ "type": "content_block_start", "index": 0, "content_block": { "type": "thinking", "thinking": "" } }),
        json!({ "type": "content_block_delta", "index": 0, "delta": { "type": "thinking_delta", "thinking": "plan" } }),
        json!({ "type": "content_block_stop", "index": 0 }),
        json!({ "type": "message_delta", "delta": { "stop_reason": "end_turn" }, "usage": { "input_tokens": 4, "output_tokens": 3 } }),
    ];

    let gemini = registry.convert_stream(&claude, ApiFormat::ClaudeChat, ApiFormat::GeminiChat).unwrap();

    assert!(gemini[0]["candidates"][0]["content"]["parts"][0].get("thoughtSignature").is_none());
    assert_eq!(gemini[1]["usageMetadata"]["promptTokenCount"], 4);
    assert_eq!(gemini[1]["usageMetadata"]["candidatesTokenCount"], 3);
    assert!(gemini[1]["usageMetadata"].get("cachedContentTokenCount").is_none());
    assert!(gemini[1]["usageMetadata"].get("thoughtsTokenCount").is_none());
}

#[test]
fn responses_stream_reasoning_emits_summary_without_signature_roundtrip() {
    let registry = FormatConversionRegistry;
    let claude = vec![
        json!({ "type": "message_start", "message": { "id": "msg_1", "model": "claude-sonnet" } }),
        json!({ "type": "content_block_start", "index": 0, "content_block": { "type": "thinking", "thinking": "" } }),
        json!({ "type": "content_block_delta", "index": 0, "delta": { "type": "thinking_delta", "thinking": "plan" } }),
        json!({ "type": "content_block_delta", "index": 0, "delta": { "type": "signature_delta", "signature": "sig_1" } }),
        json!({ "type": "content_block_stop", "index": 0 }),
        json!({ "type": "message_delta", "delta": { "stop_reason": "end_turn" }, "usage": { "input_tokens": 4, "output_tokens": 3 } }),
    ];

    let responses = registry.convert_stream(&claude, ApiFormat::ClaudeChat, ApiFormat::OpenAiResponses).unwrap();
    let reasoning_done = responses
        .iter()
        .find(|event| event["type"] == "response.output_item.done" && event["item"]["type"] == "reasoning")
        .expect("reasoning item should be completed");
    assert_eq!(reasoning_done["item"]["summary"][0]["text"], "plan");
    assert!(reasoning_done["item"].get("encrypted_content").is_none());

    let next_request = json!({
        "model": "gpt-5.5",
        "input": [
            {
                "type": "reasoning",
                "summary": [{ "type": "summary_text", "text": "plan" }],
                "encrypted_content": "sig_1"
            },
            { "type": "message", "role": "user", "content": [{ "type": "input_text", "text": "continue" }] }
        ]
    });
    let claude_request = registry
        .convert_request(&next_request, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat)
        .unwrap();

    assert_eq!(claude_request["messages"][0]["role"], "user");
    assert_eq!(claude_request["messages"][0]["content"], "");
    assert_eq!(claude_request["messages"][1]["role"], "assistant");
    assert_eq!(claude_request["messages"][1]["content"][0]["type"], "redacted_thinking");
    assert_eq!(claude_request["messages"][1]["content"][0]["data"], "sig_1");
    assert_eq!(claude_request["messages"][2]["role"], "user");
    assert_eq!(claude_request["messages"][2]["content"], "continue");
}

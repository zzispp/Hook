use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::json;

#[test]
fn openai_tool_call_stream_to_claude_emits_tool_block() {
    let registry = FormatConversionRegistry;
    let openai = vec![
        json!({
            "id": "chatcmpl_1",
            "model": "gpt-4o-mini",
            "object": "chat.completion.chunk",
            "choices": [{ "index": 0, "delta": { "role": "assistant" }, "finish_reason": null }]
        }),
        json!({
            "id": "chatcmpl_1",
            "model": "gpt-4o-mini",
            "object": "chat.completion.chunk",
            "choices": [{ "index": 0, "delta": { "tool_calls": [{
                "index": 0,
                "id": "call_1",
                "type": "function",
                "function": { "name": "lookup", "arguments": "{\"q\"" }
            }] }, "finish_reason": null }]
        }),
        json!({
            "id": "chatcmpl_1",
            "model": "gpt-4o-mini",
            "object": "chat.completion.chunk",
            "choices": [{ "index": 0, "delta": { "tool_calls": [{
                "index": 0,
                "function": { "arguments": ":\"eth\"}" }
            }] }, "finish_reason": "tool_calls" }]
        }),
    ];

    let claude = registry.convert_stream(&openai, ApiFormat::OpenAiChat, ApiFormat::ClaudeChat).unwrap();
    let tool_start = claude
        .iter()
        .find(|event| event["type"] == "content_block_start" && event["content_block"]["type"] == "tool_use")
        .expect("tool_use block should start");
    let deltas = claude
        .iter()
        .filter(|event| event["type"] == "content_block_delta" && event["delta"]["type"] == "input_json_delta")
        .collect::<Vec<_>>();

    assert_eq!(tool_start["index"], 0);
    assert_eq!(tool_start["content_block"]["id"], "call_1");
    assert_eq!(tool_start["content_block"]["name"], "lookup");
    assert_eq!(deltas[0]["delta"]["partial_json"], "{\"q\"");
    assert_eq!(deltas[1]["delta"]["partial_json"], ":\"eth\"}");
    assert!(claude.iter().any(|event| event["type"] == "content_block_stop" && event["index"] == 0));
    assert!(
        claude
            .iter()
            .any(|event| event["type"] == "message_delta" && event["delta"]["stop_reason"] == "tool_use")
    );
}

#[test]
fn gemini_function_call_stream_to_claude_emits_tool_block() {
    let registry = FormatConversionRegistry;
    let gemini = vec![json!({
        "modelVersion": "gemini-1.5-flash",
        "candidates": [{
            "content": {
                "role": "model",
                "parts": [{ "functionCall": { "id": "call_1", "name": "lookup", "args": { "q": "eth" } } }]
            },
            "finishReason": "STOP"
        }]
    })];

    let claude = registry.convert_stream(&gemini, ApiFormat::GeminiChat, ApiFormat::ClaudeChat).unwrap();
    let tool_start = claude
        .iter()
        .find(|event| event["type"] == "content_block_start" && event["content_block"]["type"] == "tool_use")
        .expect("tool_use block should start");
    let delta = claude
        .iter()
        .find(|event| event["type"] == "content_block_delta" && event["delta"]["type"] == "input_json_delta")
        .expect("tool argument delta should exist");

    assert_eq!(tool_start["content_block"]["id"], "call_1");
    assert_eq!(tool_start["content_block"]["name"], "lookup");
    assert_eq!(delta["delta"]["partial_json"], "{\"q\":\"eth\"}");
}

#[test]
fn claude_tool_use_stream_to_gemini_finishes_with_stop() {
    let registry = FormatConversionRegistry;
    let claude = vec![
        json!({ "type": "message_start", "message": { "id": "msg_1", "model": "claude-sonnet" } }),
        json!({
            "type": "content_block_start",
            "index": 1,
            "content_block": { "type": "tool_use", "id": "call_1", "name": "read_file", "input": {} }
        }),
        json!({
            "type": "content_block_delta",
            "index": 1,
            "delta": { "type": "input_json_delta", "partial_json": "{\"path\":\"README.md\"}" }
        }),
        json!({ "type": "content_block_stop", "index": 1 }),
        json!({ "type": "message_delta", "delta": { "stop_reason": "tool_use" }, "usage": { "input_tokens": 4, "output_tokens": 3 } }),
    ];

    let gemini = registry.convert_stream(&claude, ApiFormat::ClaudeChat, ApiFormat::GeminiChat).unwrap();
    let call_chunk = gemini
        .iter()
        .find(|event| event["candidates"][0]["content"]["parts"][0].get("functionCall").is_some())
        .expect("functionCall chunk should exist");
    let terminal = gemini
        .iter()
        .find(|event| event["candidates"][0].get("finishReason").is_some())
        .expect("terminal chunk should exist");

    assert_eq!(call_chunk["candidates"][0]["content"]["parts"][0]["functionCall"]["id"], "call_1");
    assert_eq!(call_chunk["candidates"][0]["content"]["parts"][0]["functionCall"]["args"]["path"], "README.md");
    assert_eq!(terminal["candidates"][0]["finishReason"], "STOP");
}

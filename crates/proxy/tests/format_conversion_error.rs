use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::json;

#[test]
fn error_conversion_maps_provider_error_shape() {
    let registry = FormatConversionRegistry;
    let claude_error = json!({
        "type": "error",
        "error": { "type": "rate_limit_error", "message": "too many requests" }
    });

    let openai = registry
        .convert_error(&claude_error, Some(429), ApiFormat::ClaudeChat, ApiFormat::OpenAiChat)
        .unwrap();
    assert_eq!(openai["error"]["message"], "too many requests");
    assert_eq!(openai["error"]["type"], "rate_limit_error");

    let gemini = registry
        .convert_error(&openai, Some(429), ApiFormat::OpenAiChat, ApiFormat::GeminiChat)
        .unwrap();
    assert_eq!(gemini["error"]["message"], "too many requests");
    assert_eq!(gemini["error"]["code"], 429);
}

#[test]
fn responses_request_reasoning_input_item_converts_to_openai_chat() {
    let registry = FormatConversionRegistry;
    let input = json!({
        "model": "gpt-5.5",
        "input": [
            { "type": "message", "role": "user", "content": [{ "type": "input_text", "text": "find sdk" }] },
            {
                "type": "reasoning",
                "summary": [{ "type": "summary_text", "text": "plan" }],
                "encrypted_content": "sig_1"
            }
        ]
    });

    let chat = registry.convert_request(&input, ApiFormat::OpenAiResponses, ApiFormat::OpenAiChat).unwrap();

    assert_eq!(chat["messages"][1]["role"], "assistant");
    assert_eq!(chat["messages"][1]["reasoning_parts"][0]["type"], "redacted_thinking");
    assert_eq!(chat["messages"][1]["reasoning_parts"][0]["data"], "sig_1");
    assert_eq!(chat["messages"][1]["content"], "");
}

#[test]
fn gemini_request_compacts_same_role_contents_for_claude_history() {
    let registry = FormatConversionRegistry;
    let input = json!({
        "model": "claude-sonnet",
        "messages": [
            { "role": "user", "content": "first" },
            { "role": "user", "content": "second" },
            {
                "role": "assistant",
                "content": [
                    { "type": "thinking", "thinking": "plan", "signature": "sig" },
                    { "type": "text", "text": "answer" }
                ]
            }
        ]
    });

    let gemini = registry.convert_request(&input, ApiFormat::ClaudeChat, ApiFormat::GeminiChat).unwrap();

    assert_eq!(gemini["contents"].as_array().unwrap().len(), 2);
    assert_eq!(gemini["contents"][0]["role"], "user");
    assert_eq!(gemini["contents"][0]["parts"].as_array().unwrap().len(), 2);
    assert_eq!(gemini["contents"][1]["role"], "model");
    assert_eq!(gemini["contents"][1]["parts"][0]["thought"], true);
    assert_eq!(gemini["contents"][1]["parts"][1]["text"], "answer");
}

#[test]
fn claude_tool_stream_outputs_complete_gemini_function_call_on_block_stop() {
    let registry = FormatConversionRegistry;
    let claude = vec![
        json!({ "type": "message_start", "message": { "id": "msg_1", "model": "claude-sonnet" } }),
        json!({ "type": "content_block_start", "index": 0, "content_block": { "type": "tool_use", "id": "tool_1", "name": "search" } }),
        json!({ "type": "content_block_delta", "index": 0, "delta": { "type": "input_json_delta", "partial_json": "{\"q\"" } }),
        json!({ "type": "content_block_delta", "index": 0, "delta": { "type": "input_json_delta", "partial_json": ":\"eth\"}" } }),
        json!({ "type": "content_block_stop", "index": 0 }),
        json!({ "type": "message_delta", "delta": { "stop_reason": "tool_use" }, "usage": { "input_tokens": 4, "output_tokens": 3 } }),
    ];

    let gemini = registry.convert_stream(&claude, ApiFormat::ClaudeChat, ApiFormat::GeminiChat).unwrap();
    let function_call = gemini
        .iter()
        .find_map(|chunk| chunk["candidates"][0]["content"]["parts"][0].get("functionCall"))
        .expect("functionCall chunk should exist");

    assert_eq!(function_call["name"], "search");
    assert_eq!(function_call["id"], "tool_1");
    assert_eq!(function_call["args"]["q"], "eth");
    assert!(gemini.iter().all(|chunk| {
        chunk["candidates"][0]["content"]["parts"][0]
            .get("functionCall")
            .and_then(|call| call.get("argsDelta"))
            .is_none()
    }));
}

#[test]
fn gemini_request_maps_parallel_function_calls_to_claude_tool_uses() {
    let registry = FormatConversionRegistry;
    let input = json!({
        "model": "gemini-pro",
        "contents": [
            { "role": "user", "parts": [{ "text": "read files" }] },
            {
                "role": "model",
                "parts": [
                    { "functionCall": { "id": "call_1", "name": "read_file", "args": { "file_path": "a" } } },
                    { "functionCall": { "id": "call_2", "name": "read_file", "args": { "file_path": "b" } } }
                ]
            },
            {
                "role": "user",
                "parts": [
                    { "functionResponse": { "name": "read_file", "response": { "result": "A" } } },
                    { "functionResponse": { "name": "read_file", "response": { "result": "B" } } }
                ]
            }
        ]
    });

    let claude = registry.convert_request(&input, ApiFormat::GeminiChat, ApiFormat::ClaudeChat).unwrap();

    assert_eq!(claude["messages"][1]["content"][0]["type"], "tool_use");
    assert_eq!(claude["messages"][1]["content"][0]["id"], "call_1");
    assert_eq!(claude["messages"][1]["content"][0]["name"], "read_file");
    assert_eq!(claude["messages"][1]["content"][0]["input"]["file_path"], "a");
    assert_eq!(claude["messages"][1]["content"][1]["type"], "tool_use");
    assert_eq!(claude["messages"][1]["content"][1]["id"], "call_2");
    assert_eq!(claude["messages"][1]["content"][1]["input"]["file_path"], "b");
    assert_eq!(claude["messages"][2]["content"][0]["type"], "tool_result");
    assert_eq!(claude["messages"][2]["content"][0]["tool_use_id"], "toolu_read_file");
    assert_eq!(claude["messages"][2]["content"][0]["content"], "A");
    assert_eq!(claude["messages"][2]["content"][1]["tool_use_id"], "toolu_read_file");
    assert_eq!(claude["messages"][2]["content"][1]["content"], "B");
}

#[test]
fn gemini_request_maps_explicit_thought_part_to_claude_thinking() {
    let registry = FormatConversionRegistry;
    let input = json!({
        "model": "gemini-pro",
        "contents": [
            { "role": "user", "parts": [{ "text": "inspect project" }] },
            {
                "role": "model",
                "parts": [
                    { "text": "Let me inspect the project.", "thought": true, "thoughtSignature": "skip_thought_signature_validator" },
                    { "functionCall": { "id": "call_1", "name": "read_file", "args": { "file_path": "README.md" } } },
                    { "functionCall": { "id": "call_2", "name": "read_file", "args": { "file_path": "package.json" } } }
                ]
            },
            {
                "role": "user",
                "parts": [
                    { "functionResponse": { "id": "call_1", "name": "read_file", "response": { "result": "README" } } },
                    { "functionResponse": { "id": "call_2", "name": "read_file", "response": { "result": "package" } } }
                ]
            }
        ]
    });

    let claude = registry.convert_request(&input, ApiFormat::GeminiChat, ApiFormat::ClaudeChat).unwrap();
    let content = claude["messages"][1]["content"].as_array().unwrap();

    assert_eq!(content[0]["type"], "thinking");
    assert_eq!(content[0]["thinking"], "Let me inspect the project.");
    assert_eq!(content[0]["signature"], "skip_thought_signature_validator");
    assert_eq!(content[1]["type"], "tool_use");
    assert_eq!(content[1]["id"], "call_1");
    assert_eq!(content[2]["id"], "call_2");
}

#[test]
fn gemini_request_preserves_function_call_with_thought_signature_as_tool_use() {
    let registry = FormatConversionRegistry;
    let input = json!({
        "model": "gemini-pro",
        "contents": [
            { "role": "user", "parts": [{ "text": "inspect project" }] },
            {
                "role": "model",
                "parts": [
                    {
                        "thoughtSignature": "skip_thought_signature_validator",
                        "functionCall": { "id": "call_1", "name": "read_file", "args": { "file_path": "README.md" } }
                    },
                    { "functionCall": { "id": "call_2", "name": "read_file", "args": { "file_path": "package.json" } } }
                ]
            },
            {
                "role": "user",
                "parts": [
                    { "functionResponse": { "id": "call_1", "name": "read_file", "response": { "result": "README" } } },
                    { "functionResponse": { "id": "call_2", "name": "read_file", "response": { "result": "package" } } }
                ]
            }
        ]
    });

    let claude = registry.convert_request(&input, ApiFormat::GeminiChat, ApiFormat::ClaudeChat).unwrap();
    let content = claude["messages"][1]["content"].as_array().unwrap();

    assert_eq!(content[0]["type"], "tool_use");
    assert_eq!(content[0]["id"], "call_1");
    assert_eq!(content[0]["name"], "read_file");
    assert_eq!(content[0]["input"]["file_path"], "README.md");
    assert!(content[0].get("thoughtSignature").is_none());
    assert_eq!(content[1]["type"], "tool_use");
    assert_eq!(content[1]["id"], "call_2");
}

#[test]
fn gemini_to_claude_keeps_orphaned_history_tool_use_visible() {
    let registry = FormatConversionRegistry;
    let input = json!({
        "model": "gemini-pro",
        "contents": [
            { "role": "user", "parts": [{ "text": "plan" }] },
            {
                "role": "model",
                "parts": [{
                    "text": "Let me inspect first."
                }, {
                    "functionCall": {
                        "id": "call_00_MRnT5ExfRUvx7WmI7oUj4977",
                        "name": "codebase_investigator",
                        "args": { "task": "inspect" }
                    }
                }]
            },
            { "role": "user", "parts": [{ "text": "next prompt" }] }
        ]
    });

    let claude = registry.convert_request(&input, ApiFormat::GeminiChat, ApiFormat::ClaudeChat).unwrap();
    let messages = claude["messages"].as_array().unwrap();

    assert_eq!(messages[1]["role"], "assistant");
    assert_eq!(messages[1]["content"][0]["text"], "Let me inspect first.");
    assert_eq!(messages[1]["content"][1]["type"], "tool_use");
    assert_eq!(messages[1]["content"][1]["id"], "call_00_MRnT5ExfRUvx7WmI7oUj4977");
    assert_eq!(messages[2]["role"], "user");
    assert_eq!(messages[2]["content"], "next prompt");
}

#[test]
fn gemini_to_claude_preserves_mixed_user_part_order() {
    let registry = FormatConversionRegistry;
    let input = json!({
        "model": "gemini-pro",
        "contents": [
            { "role": "user", "parts": [{ "text": "run" }] },
            {
                "role": "model",
                "parts": [{
                    "functionCall": { "id": "call_1", "name": "lookup", "args": { "q": "eth" } }
                }]
            },
            {
                "role": "user",
                "parts": [
                    { "text": "also answer this" },
                    { "functionResponse": { "id": "call_1", "name": "lookup", "response": { "result": "ok" } } }
                ]
            }
        ]
    });

    let claude = registry.convert_request(&input, ApiFormat::GeminiChat, ApiFormat::ClaudeChat).unwrap();
    let content = claude["messages"][2]["content"].as_array().unwrap();

    assert_eq!(content[0]["type"], "text");
    assert_eq!(content[0]["text"], "also answer this");
    assert_eq!(content[1]["type"], "tool_result");
    assert_eq!(content[1]["tool_use_id"], "call_1");
}

use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::json;

#[test]
fn error_conversion_maps_provider_error_shape() {
    let registry = FormatConversionRegistry::default();
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
fn responses_request_keeps_claude_assistant_thinking_text_and_tools_together() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gpt-5.5",
        "input": [
            { "type": "message", "role": "user", "content": [{ "type": "input_text", "text": "find sdk" }] },
            { "type": "function_call", "call_id": "call_1", "name": "search", "arguments": "{\"q\":\"ethers\"}" },
            { "type": "function_call", "call_id": "call_2", "name": "search", "arguments": "{\"q\":\"wagmi\"}" },
            {
                "type": "reasoning",
                "summary": [{ "type": "summary_text", "text": "plan" }],
                "encrypted_content": "sig_1"
            },
            { "type": "message", "role": "assistant", "content": [{ "type": "output_text", "text": "checking" }] },
            { "type": "function_call_output", "call_id": "call_1", "output": "ethers result" },
            { "type": "function_call_output", "call_id": "call_2", "output": "wagmi result" }
        ]
    });

    let claude = registry.convert_request(&input, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat).unwrap();

    assert_eq!(claude["messages"][1]["role"], "assistant");
    assert_eq!(claude["messages"][1]["content"].as_array().unwrap().len(), 4);
    assert_eq!(claude["messages"][1]["content"][0]["type"], "thinking");
    assert_eq!(claude["messages"][1]["content"][0]["signature"], "sig_1");
    assert_eq!(claude["messages"][1]["content"][1]["type"], "text");
    assert_eq!(claude["messages"][1]["content"][2]["type"], "tool_use");
    assert_eq!(claude["messages"][1]["content"][3]["id"], "call_2");
    assert_eq!(claude["messages"][2]["role"], "user");
    assert_eq!(claude["messages"][2]["content"][0]["type"], "tool_result");
    assert_eq!(claude["messages"][2]["content"][0]["content"], "ethers result");
}

#[test]
fn gemini_request_compacts_same_role_contents_for_claude_history() {
    let registry = FormatConversionRegistry::default();
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
    let registry = FormatConversionRegistry::default();
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
fn gemini_request_preserves_parallel_function_call_ids_for_claude() {
    let registry = FormatConversionRegistry::default();
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

    assert_eq!(claude["messages"][1]["content"][0]["id"], "call_1");
    assert_eq!(claude["messages"][1]["content"][1]["id"], "call_2");
    assert_eq!(claude["messages"][2]["content"][0]["tool_use_id"], "call_1");
    assert_eq!(claude["messages"][2]["content"][1]["tool_use_id"], "call_2");
}

#[test]
fn gemini_request_restores_function_call_thought_signature_for_claude() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gemini-pro",
        "contents": [
            { "role": "user", "parts": [{ "text": "inspect project" }] },
            {
                "role": "model",
                "parts": [
                    { "text": "Let me inspect the project." },
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

    assert_eq!(content[0]["type"], "thinking");
    assert_eq!(content[0]["thinking"], "Let me inspect the project.");
    assert_eq!(content[0]["signature"], "skip_thought_signature_validator");
    assert_eq!(content[1]["type"], "text");
    assert_eq!(content[2]["type"], "tool_use");
    assert_eq!(content[2]["id"], "call_1");
    assert_eq!(content[3]["id"], "call_2");
}

#[test]
fn gemini_request_uses_dummy_thinking_when_signature_has_no_text() {
    let registry = FormatConversionRegistry::default();
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

    assert_eq!(content[0]["type"], "thinking");
    assert_eq!(content[0]["thinking"], "Thinking...");
    assert_eq!(content[0]["signature"], "skip_thought_signature_validator");
    assert_eq!(content[1]["type"], "tool_use");
    assert_eq!(content[1]["id"], "call_1");
    assert_eq!(content[2]["id"], "call_2");
}

#[test]
fn gemini_to_claude_removes_orphaned_history_tool_use() {
    let registry = FormatConversionRegistry::default();
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
    assert_eq!(messages[2]["role"], "user");
    assert_eq!(messages[2]["content"][0]["text"], "next prompt");
    assert!(serde_json::to_string(&claude).unwrap().find("call_00_MRnT5ExfRUvx7WmI7oUj4977").is_none());
}

#[test]
fn gemini_to_claude_places_tool_results_before_user_text() {
    let registry = FormatConversionRegistry::default();
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

    assert_eq!(content[0]["type"], "tool_result");
    assert_eq!(content[0]["tool_use_id"], "call_1");
    assert_eq!(content[1]["type"], "text");
}

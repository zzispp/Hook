use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::json;

#[test]
fn claude_request_to_openai_keeps_thinking_and_tool_results() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "claude-sonnet",
        "messages": [
            { "role": "user", "content": "inspect" },
            {
                "role": "assistant",
                "content": [
                    { "type": "thinking", "thinking": "plan", "signature": "sig" },
                    { "type": "text", "text": "checking" },
                    { "type": "tool_use", "id": "call_1", "name": "read_file", "input": { "path": "README.md" } }
                ]
            },
            {
                "role": "user",
                "content": [
                    { "type": "tool_result", "tool_use_id": "call_1", "content": "file contents" }
                ]
            }
        ]
    });

    let openai = registry.convert_request(&input, ApiFormat::ClaudeChat, ApiFormat::OpenAiChat).unwrap();

    assert_eq!(openai["messages"][1]["role"], "assistant");
    assert_eq!(openai["messages"][1]["reasoning_content"], "plan");
    assert_eq!(openai["messages"][1]["content"][0]["text"], "checking");
    assert_eq!(openai["messages"][1]["tool_calls"][0]["id"], "call_1");
    assert_eq!(openai["messages"][2]["role"], "tool");
    assert_eq!(openai["messages"][2]["tool_call_id"], "call_1");
    assert_eq!(openai["messages"][2]["content"], "file contents");
}

#[test]
fn gemini_request_to_openai_keeps_thinking_and_parallel_tool_results() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gemini-pro",
        "contents": [
            { "role": "user", "parts": [{ "text": "inspect" }] },
            {
                "role": "model",
                "parts": [
                    { "text": "plan", "thought": true, "thoughtSignature": "sig" },
                    { "text": "checking" },
                    { "functionCall": { "id": "call_1", "name": "read_file", "args": { "path": "README.md" } } },
                    { "functionCall": { "id": "call_2", "name": "read_file", "args": { "path": "Cargo.toml" } } }
                ]
            },
            {
                "role": "user",
                "parts": [
                    { "functionResponse": { "name": "read_file", "response": { "result": "README" } } },
                    { "functionResponse": { "name": "read_file", "response": { "result": "Cargo" } } }
                ]
            }
        ]
    });

    let openai = registry.convert_request(&input, ApiFormat::GeminiChat, ApiFormat::OpenAiChat).unwrap();

    assert_eq!(openai["messages"][1]["reasoning_content"], "plan");
    assert_eq!(openai["messages"][1]["content"][0]["text"], "checking");
    assert_eq!(openai["messages"][1]["tool_calls"][0]["id"], "call_1");
    assert_eq!(openai["messages"][1]["tool_calls"][1]["id"], "call_2");
    assert_eq!(openai["messages"][2]["role"], "tool");
    assert_eq!(openai["messages"][2]["tool_call_id"], "call_1");
    assert_eq!(openai["messages"][3]["tool_call_id"], "call_2");
}

#[test]
fn tool_result_messages_keep_adjacent_user_text_for_openai_targets_like_aether() {
    let registry = FormatConversionRegistry::default();
    let claude = json!({
        "model": "claude-sonnet",
        "messages": [{
            "role": "user",
            "content": [
                { "type": "text", "text": "before" },
                { "type": "tool_result", "tool_use_id": "call_1", "content": "tool output" },
                { "type": "text", "text": "after" }
            ]
        }]
    });
    let gemini = json!({
        "model": "gemini-pro",
        "contents": [{
            "role": "user",
            "parts": [
                { "text": "before" },
                { "functionResponse": { "id": "call_1", "name": "lookup", "response": { "result": "tool output" } } },
                { "text": "after" }
            ]
        }]
    });

    let openai = registry.convert_request(&claude, ApiFormat::ClaudeChat, ApiFormat::OpenAiChat).unwrap();
    let responses = registry.convert_request(&gemini, ApiFormat::GeminiChat, ApiFormat::OpenAiResponses).unwrap();

    assert_eq!(openai["messages"][0]["role"], "user");
    assert_eq!(openai["messages"][0]["content"], "before");
    assert_eq!(openai["messages"][1]["role"], "tool");
    assert_eq!(openai["messages"][1]["tool_call_id"], "call_1");
    assert_eq!(openai["messages"][2]["content"], "after");
    assert_eq!(responses["input"][0]["type"], "function_call_output");
    assert_eq!(responses["input"][0]["call_id"], "call_1");
    assert_eq!(responses["input"][1]["type"], "message");
    assert_eq!(responses["input"][1]["content"][0]["text"], "before");
    assert_eq!(responses["input"][1]["content"][1]["text"], "after");
}

#[test]
fn registry_repairs_missing_tool_ids_like_aether() {
    let registry = FormatConversionRegistry::default();
    let openai = json!({
        "model": "gpt-5.5",
        "messages": [
            { "role": "assistant", "tool_calls": [{
                "type": "function",
                "function": { "name": "lookup", "arguments": "{\"q\":\"eth\"}" }
            }] },
            { "role": "tool", "content": "ok" }
        ]
    });
    let responses = json!({
        "model": "gpt-5.5",
        "input": [
            { "type": "function_call", "name": "lookup", "arguments": "{\"q\":\"eth\"}" },
            { "type": "function_call_output", "output": "ok" }
        ]
    });
    let gemini = json!({
        "model": "gemini-pro",
        "contents": [
            { "role": "model", "parts": [{ "functionCall": { "name": "lookup", "args": { "q": "eth" } } }] },
            { "role": "user", "parts": [{ "functionResponse": { "name": "lookup", "response": { "result": "ok" } } }] }
        ]
    });

    let claude_from_openai = registry.convert_request(&openai, ApiFormat::OpenAiChat, ApiFormat::ClaudeChat).unwrap();
    let claude_from_responses = registry.convert_request(&responses, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat).unwrap();
    let openai_from_gemini = registry.convert_request(&gemini, ApiFormat::GeminiChat, ApiFormat::OpenAiChat).unwrap();

    assert_eq!(claude_from_openai["messages"][1]["content"][0]["id"], "call_auto_1");
    assert_eq!(claude_from_openai["messages"][2]["content"][0]["tool_use_id"], "call_auto_1");
    assert_eq!(claude_from_responses["messages"][1]["content"][0]["id"], "call_auto_1");
    assert_eq!(claude_from_responses["messages"][2]["content"][0]["tool_use_id"], "call_auto_1");
    assert_eq!(openai_from_gemini["messages"][0]["tool_calls"][0]["id"], "toolu_lookup");
    assert_eq!(openai_from_gemini["messages"][1]["tool_call_id"], "toolu_lookup");
}

#[test]
fn openai_legacy_function_call_maps_to_tool_use_like_aether() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gpt-5.5",
        "messages": [
            {
                "role": "assistant",
                "content": null,
                "function_call": { "name": "lookup", "arguments": "\"raw query\"" }
            },
            { "role": "tool", "tool_call_id": "call_0", "content": "ok" }
        ]
    });

    let claude = registry.convert_request(&input, ApiFormat::OpenAiChat, ApiFormat::ClaudeChat).unwrap();
    let gemini = registry.convert_request(&input, ApiFormat::OpenAiChat, ApiFormat::GeminiChat).unwrap();

    assert_eq!(claude["messages"][1]["content"][0]["type"], "tool_use");
    assert_eq!(claude["messages"][1]["content"][0]["id"], "call_0");
    assert_eq!(claude["messages"][1]["content"][0]["name"], "lookup");
    assert_eq!(claude["messages"][1]["content"][0]["input"]["raw"], "raw query");
    assert_eq!(gemini["contents"][0]["parts"][0]["functionCall"]["id"], "call_0");
    assert_eq!(gemini["contents"][0]["parts"][0]["functionCall"]["args"]["raw"], "raw query");
}

#[test]
fn openai_malformed_tool_arguments_are_kept_as_raw_like_aether() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gpt-5.5",
        "messages": [{
            "role": "assistant",
            "content": null,
            "tool_calls": [{
                "id": "call_1",
                "type": "function",
                "function": { "name": "lookup", "arguments": "{not-json" }
            }]
        }, {
            "role": "tool",
            "tool_call_id": "call_1",
            "content": "ok"
        }]
    });

    let claude = registry.convert_request(&input, ApiFormat::OpenAiChat, ApiFormat::ClaudeChat).unwrap();
    let gemini = registry.convert_request(&input, ApiFormat::OpenAiChat, ApiFormat::GeminiChat).unwrap();

    assert_eq!(claude["messages"][1]["content"][0]["input"]["raw"], "{not-json");
    assert_eq!(gemini["contents"][0]["parts"][0]["functionCall"]["args"]["raw"], "{not-json");
}

#[test]
fn openai_responses_malformed_tool_arguments_are_kept_as_raw_like_aether() {
    let registry = FormatConversionRegistry::default();
    let request = json!({
        "model": "gpt-5.5",
        "input": [{
            "type": "function_call",
            "call_id": "call_1",
            "name": "lookup",
            "arguments": "{not-json"
        }, {
            "type": "function_call_output",
            "call_id": "call_1",
            "output": "ok"
        }]
    });
    let response = json!({
        "id": "resp_1",
        "model": "gpt-5.5",
        "output": [{
            "type": "function_call",
            "call_id": "call_1",
            "name": "lookup",
            "arguments": "{not-json"
        }]
    });

    let claude_request = registry.convert_request(&request, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat).unwrap();
    let claude_response = registry.convert_response(&response, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat).unwrap();

    assert_eq!(claude_request["messages"][1]["content"][0]["input"]["_raw"], "{not-json");
    assert_eq!(claude_response["content"][0]["input"]["_raw"], "{not-json");
}

#[test]
fn openai_responses_function_call_output_is_user_tool_result_like_aether() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gpt-5.5",
        "input": [
            { "type": "function_call", "call_id": "call_1", "name": "lookup", "arguments": "{\"q\":\"eth\"}" },
            { "type": "function_call_output", "call_id": "call_1", "output": "ok" }
        ]
    });

    let chat = registry.convert_request(&input, ApiFormat::OpenAiResponses, ApiFormat::OpenAiChat).unwrap();
    let gemini = registry.convert_request(&input, ApiFormat::OpenAiResponses, ApiFormat::GeminiChat).unwrap();

    assert_eq!(chat["messages"][1]["role"], "tool");
    assert_eq!(chat["messages"][1]["tool_call_id"], "call_1");
    assert_eq!(gemini["contents"][1]["role"], "user");
    assert_eq!(gemini["contents"][1]["parts"][0]["functionResponse"]["id"], "call_1");
}

#[test]
fn openai_response_to_gemini_keeps_reasoning_and_tool_calls() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "id": "chatcmpl_1",
        "model": "gpt-5.5",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "reasoning_content": "plan",
                "content": "checking",
                "tool_calls": [{
                    "id": "call_1",
                    "type": "function",
                    "function": { "name": "read_file", "arguments": "{\"path\":\"README.md\"}" }
                }]
            },
            "finish_reason": "tool_calls"
        }]
    });

    let gemini = registry.convert_response(&input, ApiFormat::OpenAiChat, ApiFormat::GeminiChat).unwrap();
    let parts = gemini["candidates"][0]["content"]["parts"].as_array().unwrap();

    assert_eq!(parts[0]["thought"], true);
    assert_eq!(parts[0]["text"], "plan");
    assert_eq!(parts[1]["text"], "checking");
    assert_eq!(parts[2]["functionCall"]["id"], "call_1");
    assert_eq!(parts[2]["functionCall"]["args"]["path"], "README.md");
}

#[test]
fn openai_request_to_claude_maps_reasoning_parallel_and_web_search() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gpt-5.5",
        "messages": [{ "role": "user", "content": "search" }],
        "reasoning_effort": "xhigh",
        "parallel_tool_calls": false,
        "tool_choice": "required",
        "web_search_options": {
            "search_context_size": "low",
            "user_location": { "type": "approximate", "country": "US" }
        }
    });

    let claude = registry.convert_request(&input, ApiFormat::OpenAiChat, ApiFormat::ClaudeChat).unwrap();

    assert_eq!(claude["thinking"]["budget_tokens"], 8192);
    assert_eq!(claude["output_config"]["effort"], "max");
    assert_eq!(claude["max_tokens"], 8192);
    assert_eq!(claude["tool_choice"]["disable_parallel_tool_use"], true);
    assert_eq!(claude["tools"][0]["type"], "web_search_20250305");
    assert_eq!(claude["tools"][0]["max_uses"], 1);
}

#[test]
fn claude_request_to_gemini_keeps_top_k_and_parallel_setting() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "claude-sonnet",
        "top_k": 77,
        "tool_choice": { "type": "any", "disable_parallel_tool_use": true },
        "messages": [{ "role": "user", "content": "pick" }]
    });

    let gemini = registry.convert_request(&input, ApiFormat::ClaudeChat, ApiFormat::GeminiChat).unwrap();

    assert_eq!(gemini["generationConfig"]["topK"], 77);
    assert_eq!(gemini["toolConfig"]["functionCallingConfig"]["mode"], "ANY");
}

#[test]
fn openai_request_to_gemini_maps_reasoning_response_format_and_web_search() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gpt-5.5",
        "messages": [{ "role": "user", "content": "json search" }],
        "reasoning_effort": "medium",
        "web_search_options": { "search_context_size": "low" },
        "extra_body": {
            "google": {
                "thinking_config": { "include_thoughts": true, "thinking_budget": 3333 },
                "response_modalities": ["TEXT"]
            }
        },
        "response_format": {
            "type": "json_schema",
            "json_schema": {
                "name": "answer",
                "schema": {
                    "type": "object",
                    "properties": { "answer": { "type": "string" } },
                    "required": ["answer"]
                }
            }
        }
    });

    let gemini = registry.convert_request(&input, ApiFormat::OpenAiChat, ApiFormat::GeminiChat).unwrap();

    assert_eq!(gemini["generationConfig"]["thinkingConfig"]["includeThoughts"], true);
    assert_eq!(gemini["generationConfig"]["thinkingConfig"]["thinkingBudget"], 2048);
    assert_eq!(gemini["generationConfig"]["responseModalities"][0], "TEXT");
    assert_eq!(gemini["generationConfig"]["responseMimeType"], "application/json");
    assert_eq!(gemini["generationConfig"]["responseSchema"]["properties"]["answer"]["type"], "string");
    assert_eq!(gemini["tools"][0]["googleSearch"], json!({}));
}

#[test]
fn gemini_request_to_openai_keeps_response_format_and_builtin_tools() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gemini-pro",
        "contents": [{ "role": "user", "parts": [{ "text": "json search" }] }],
        "generationConfig": {
            "candidateCount": 2,
            "responseMimeType": "application/json",
            "responseSchema": {
                "type": "object",
                "properties": { "answer": { "type": "string" } }
            },
            "thinkingConfig": { "includeThoughts": true, "thinkingBudget": 4096 }
        },
        "tools": [{ "googleSearch": {} }]
    });

    let openai = registry.convert_request(&input, ApiFormat::GeminiChat, ApiFormat::OpenAiChat).unwrap();

    assert_eq!(openai["reasoning_effort"], "high");
    assert_eq!(openai["n"], 2);
    assert_eq!(openai["response_format"]["type"], "json_schema");
    assert_eq!(openai["response_format"]["json_schema"]["properties"]["answer"]["type"], "string");
    assert!(openai.get("tools").is_none());
}

#[test]
fn gemini_native_request_extras_round_trip() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gemini-pro",
        "contents": [{ "role": "user", "parts": [{ "text": "native" }] }],
        "generationConfig": {
            "seed": 9,
            "presencePenalty": 0.4,
            "responseModalities": ["TEXT"]
        },
        "safetySettings": [{ "category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "BLOCK_NONE" }],
        "cachedContent": "cachedContents/abc"
    });

    let gemini = registry.convert_request(&input, ApiFormat::GeminiChat, ApiFormat::GeminiChat).unwrap();

    assert_eq!(gemini["generationConfig"]["seed"], 9);
    assert_eq!(gemini["generationConfig"]["presencePenalty"], 0.4);
    assert_eq!(gemini["generationConfig"]["responseModalities"][0], "TEXT");
    assert_eq!(gemini["safetySettings"][0]["threshold"], "BLOCK_NONE");
    assert_eq!(gemini["cachedContent"], "cachedContents/abc");
}

#[test]
fn claude_tool_result_to_gemini_keeps_tool_id_and_result_shape() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "claude-sonnet",
        "messages": [{
            "role": "user",
            "content": [{
                "type": "tool_result",
                "tool_use_id": "call_1",
                "content": "{\"answer\":\"ok\"}"
            }]
        }]
    });

    let gemini = registry.convert_request(&input, ApiFormat::ClaudeChat, ApiFormat::GeminiChat).unwrap();
    let response = &gemini["contents"][0]["parts"][0]["functionResponse"];

    assert_eq!(response["id"], "call_1");
    assert_eq!(response["name"], "call_1");
    assert_eq!(response["response"]["result"]["answer"], "ok");
}

#[test]
fn claude_non_object_tool_input_is_wrapped_as_raw_like_aether() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "claude-sonnet",
        "messages": [{
            "role": "assistant",
            "content": [{
                "type": "tool_use",
                "id": "call_1",
                "name": "lookup",
                "input": "raw query"
            }]
        }]
    });

    let openai = registry.convert_request(&input, ApiFormat::ClaudeChat, ApiFormat::OpenAiChat).unwrap();
    let gemini = registry.convert_request(&input, ApiFormat::ClaudeChat, ApiFormat::GeminiChat).unwrap();

    assert_eq!(openai["messages"][0]["tool_calls"][0]["function"]["arguments"], "{\"raw\":\"raw query\"}");
    assert_eq!(gemini["contents"][0]["parts"][0]["functionCall"]["args"]["raw"], "raw query");
}

#[test]
fn gemini_function_response_result_unwraps_like_aether() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gemini-pro",
        "contents": [{
            "role": "user",
            "parts": [{
                "functionResponse": {
                    "id": "call_1",
                    "name": "lookup",
                    "response": { "result": "tool output" }
                }
            }]
        }]
    });

    let claude = registry.convert_request(&input, ApiFormat::GeminiChat, ApiFormat::ClaudeChat).unwrap();
    let openai = registry.convert_request(&input, ApiFormat::GeminiChat, ApiFormat::OpenAiChat).unwrap();

    assert_eq!(claude["messages"][0]["content"][0]["content"], "tool output");
    assert_eq!(openai["messages"][0]["content"], "tool output");
}

#[test]
fn gemini_non_object_function_call_args_are_empty_like_aether() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gemini-pro",
        "contents": [
            {
                "role": "model",
                "parts": [{
                    "functionCall": {
                        "id": "call_1",
                        "name": "lookup",
                        "args": "raw query"
                    }
                }]
            },
            {
                "role": "user",
                "parts": [{
                    "functionResponse": {
                        "id": "call_1",
                        "name": "lookup",
                        "response": { "result": "ok" }
                    }
                }]
            }
        ]
    });

    let claude = registry.convert_request(&input, ApiFormat::GeminiChat, ApiFormat::ClaudeChat).unwrap();
    let openai = registry.convert_request(&input, ApiFormat::GeminiChat, ApiFormat::OpenAiChat).unwrap();

    assert_eq!(claude["messages"][1]["content"][0]["input"], json!({}));
    assert_eq!(openai["messages"][0]["tool_calls"][0]["function"]["arguments"], "{}");
}

#[test]
fn claude_target_prepends_empty_user_when_first_message_is_assistant_like_aether() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gpt-5.5",
        "messages": [{
            "role": "assistant",
            "content": "assistant first"
        }]
    });

    let claude = registry.convert_request(&input, ApiFormat::OpenAiChat, ApiFormat::ClaudeChat).unwrap();

    assert_eq!(claude["messages"][0]["role"], "user");
    assert_eq!(claude["messages"][0]["content"].as_array().unwrap().len(), 0);
    assert_eq!(claude["messages"][1]["role"], "assistant");
    assert_eq!(claude["messages"][1]["content"][0]["text"], "assistant first");
}

#[test]
fn claude_response_target_emits_default_usage_like_aether() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "id": "chatcmpl_1",
        "model": "gpt-5.5",
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": "hello" },
            "finish_reason": "stop"
        }]
    });

    let claude = registry.convert_response(&input, ApiFormat::OpenAiChat, ApiFormat::ClaudeChat).unwrap();

    assert_eq!(claude["usage"]["input_tokens"], 0);
    assert_eq!(claude["usage"]["output_tokens"], 0);
}

#[test]
fn openai_zero_usage_is_treated_as_missing_like_aether() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "id": "chatcmpl_1",
        "model": "gpt-5.5",
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": "hello" },
            "finish_reason": "stop"
        }],
        "usage": { "prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0 }
    });

    let gemini = registry.convert_response(&input, ApiFormat::OpenAiChat, ApiFormat::GeminiChat).unwrap();
    let responses = registry.convert_response(&input, ApiFormat::OpenAiChat, ApiFormat::OpenAiResponses).unwrap();

    assert!(gemini.get("usageMetadata").is_none());
    assert!(responses.get("usage").is_none());
}

#[test]
fn gemini_thought_tokens_are_counted_as_output_tokens_like_aether() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "modelVersion": "gemini-pro",
        "candidates": [{
            "content": { "role": "model", "parts": [{ "text": "hello" }] },
            "finishReason": "STOP"
        }],
        "usageMetadata": {
            "promptTokenCount": 5,
            "candidatesTokenCount": 7,
            "thoughtsTokenCount": 3
        }
    });

    let openai = registry.convert_response(&input, ApiFormat::GeminiChat, ApiFormat::OpenAiChat).unwrap();
    let claude = registry.convert_response(&input, ApiFormat::GeminiChat, ApiFormat::ClaudeChat).unwrap();

    assert_eq!(openai["usage"]["completion_tokens"], 10);
    assert_eq!(openai["usage"]["total_tokens"], 15);
    assert_eq!(openai["usage"]["completion_tokens_details"]["reasoning_tokens"], 3);
    assert_eq!(claude["usage"]["output_tokens"], 10);
}

#[test]
fn claude_system_cache_control_round_trips_like_aether() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "claude-sonnet",
        "system": [
            { "type": "text", "text": "cached system", "cache_control": { "type": "ephemeral" } },
            { "type": "text", "text": "plain system" }
        ],
        "messages": [{ "role": "user", "content": "hi" }]
    });

    let claude = registry.convert_request(&input, ApiFormat::ClaudeChat, ApiFormat::ClaudeChat).unwrap();

    assert_eq!(claude["system"][0]["text"], "cached system");
    assert_eq!(claude["system"][0]["cache_control"]["type"], "ephemeral");
    assert_eq!(claude["system"][1]["text"], "plain system");
    assert!(claude["system"][1].get("cache_control").is_none());
}

#[test]
fn response_target_ids_match_aether_envelopes() {
    let registry = FormatConversionRegistry::default();
    let gemini = json!({
        "id": "resp_123",
        "modelVersion": "gemini-pro",
        "candidates": [{
            "content": { "role": "model", "parts": [{ "text": "hello" }] },
            "finishReason": "STOP"
        }]
    });
    let openai = json!({
        "id": "plain_id",
        "model": "gpt-5.5",
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": "hello" },
            "finish_reason": "stop"
        }]
    });

    let openai_from_gemini = registry.convert_response(&gemini, ApiFormat::GeminiChat, ApiFormat::OpenAiChat).unwrap();
    let claude_from_openai = registry.convert_response(&openai, ApiFormat::OpenAiChat, ApiFormat::ClaudeChat).unwrap();

    assert_eq!(openai_from_gemini["id"], "chatcmpl-123");
    assert!(openai_from_gemini.get("system_fingerprint").is_some());
    assert_eq!(claude_from_openai["id"], "msg_plain_id");
}

#[test]
fn unknown_tool_choice_coerces_to_auto_like_aether() {
    let registry = FormatConversionRegistry::default();
    let openai = json!({
        "model": "gpt-5.5",
        "messages": [{ "role": "user", "content": "hi" }],
        "tool_choice": "mystery"
    });
    let gemini = json!({
        "model": "gemini-pro",
        "contents": [{ "role": "user", "parts": [{ "text": "hi" }] }],
        "toolConfig": { "functionCallingConfig": { "mode": "MYSTERY" } }
    });
    let claude = json!({
        "model": "claude-sonnet",
        "messages": [{ "role": "user", "content": "hi" }],
        "tool_choice": "mystery"
    });

    let claude_from_openai = registry.convert_request(&openai, ApiFormat::OpenAiChat, ApiFormat::ClaudeChat).unwrap();
    let openai_from_gemini = registry.convert_request(&gemini, ApiFormat::GeminiChat, ApiFormat::OpenAiChat).unwrap();
    let gemini_from_claude = registry.convert_request(&claude, ApiFormat::ClaudeChat, ApiFormat::GeminiChat).unwrap();

    assert_eq!(claude_from_openai["tool_choice"]["type"], "auto");
    assert_eq!(openai_from_gemini["tool_choice"], "auto");
    assert_eq!(gemini_from_claude["toolConfig"]["functionCallingConfig"]["mode"], "AUTO");
}

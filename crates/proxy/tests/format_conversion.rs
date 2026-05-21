use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::json;

#[test]
fn format_conversion_request_openai_to_gemini_and_claude() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gpt-4o-mini",
        "messages": [
            { "role": "system", "content": "sys" },
            { "role": "user", "content": "hi" }
        ],
        "temperature": 0.2,
        "max_tokens": 12,
        "stream": true
    });

    let gemini = registry.convert_request(&input, ApiFormat::OpenAiChat, ApiFormat::GeminiChat).unwrap();
    assert_eq!(gemini["systemInstruction"]["parts"][0]["text"], "sys");
    assert_eq!(gemini["contents"][0]["role"], "user");
    assert_eq!(gemini["contents"][0]["parts"][0]["text"], "hi");
    assert_eq!(gemini["generationConfig"]["maxOutputTokens"], 12);

    let claude = registry.convert_request(&input, ApiFormat::OpenAiChat, ApiFormat::ClaudeChat).unwrap();
    assert_eq!(claude["system"], "sys");
    assert_eq!(claude["messages"][0]["role"], "user");
    assert_eq!(claude["messages"][0]["content"][0]["text"], "hi");
    assert_eq!(claude["stream"], true);
}

#[test]
fn format_conversion_request_openai_to_responses_and_back() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gpt-5.5",
        "messages": [
            { "role": "user", "content": "hello" }
        ],
        "max_tokens": 16,
        "stream": true
    });

    let responses = registry.convert_request(&input, ApiFormat::OpenAiChat, ApiFormat::OpenAiResponses).unwrap();
    assert_eq!(responses["input"][0]["role"], "user");
    assert_eq!(responses["input"][0]["content"][0]["type"], "input_text");
    assert_eq!(responses["input"][0]["content"][0]["text"], "hello");
    assert_eq!(responses["max_output_tokens"], 16);
    assert_eq!(responses["stream"], true);

    let response_payload = json!({
        "id": "resp_1",
        "model": "gpt-5.5",
        "output_text": "hello",
        "usage": { "input_tokens": 2, "output_tokens": 1, "total_tokens": 3 }
    });
    let openai = registry
        .convert_response(&response_payload, ApiFormat::OpenAiResponses, ApiFormat::OpenAiChat)
        .unwrap();
    assert_eq!(openai["choices"][0]["message"]["content"], "hello");
    assert_eq!(openai["usage"]["total_tokens"], 3);
}

#[test]
fn format_conversion_request_openai_responses_hosted_web_search_to_claude() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gpt-5.5",
        "input": [{ "role": "user", "content": [{ "type": "input_text", "text": "search it" }] }],
        "stream": true,
        "tools": [
            { "type": "function", "name": "run", "description": "Run a command", "parameters": { "type": "object" } },
            { "type": "web_search", "external_web_access": false }
        ]
    });

    let claude = registry.convert_request(&input, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat).unwrap();

    assert_eq!(claude["tools"][0]["name"], "run");
    assert_eq!(claude["tools"][1]["type"], "web_search_20250305");
    assert_eq!(claude["tools"][1]["name"], "web_search");
    assert_eq!(claude["tools"][1]["max_uses"], 5);
}

#[test]
fn format_conversion_request_openai_responses_groups_parallel_tool_turns_for_claude() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gpt-5.5",
        "input": [
            { "role": "user", "content": [{ "type": "input_text", "text": "find sdk" }] },
            { "type": "function_call", "call_id": "call_1", "name": "search", "arguments": "{\"q\":\"ethers\"}" },
            { "type": "function_call", "call_id": "call_2", "name": "search", "arguments": "{\"q\":\"wagmi\"}" },
            { "type": "function_call_output", "call_id": "call_1", "output": "ethers result" },
            { "type": "function_call_output", "call_id": "call_2", "output": "wagmi result" }
        ],
        "tools": [{ "type": "function", "name": "search", "parameters": { "type": "object" } }]
    });

    let claude = registry.convert_request(&input, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat).unwrap();

    assert_eq!(claude["messages"][1]["role"], "assistant");
    assert_eq!(claude["messages"][1]["content"].as_array().unwrap().len(), 2);
    assert_eq!(claude["messages"][1]["content"][0]["type"], "tool_use");
    assert_eq!(claude["messages"][1]["content"][1]["id"], "call_2");
    assert_eq!(claude["messages"][2]["role"], "user");
    assert_eq!(claude["messages"][2]["content"].as_array().unwrap().len(), 2);
    assert_eq!(claude["messages"][2]["content"][0]["type"], "tool_result");
    assert_eq!(claude["messages"][2]["content"][0]["content"], "ethers result");
    assert_eq!(claude["messages"][2]["content"][1]["tool_use_id"], "call_2");
    assert_eq!(claude["messages"][2]["content"][1]["content"], "wagmi result");
}

#[test]
fn format_conversion_request_gemini_and_claude_to_openai() {
    let registry = FormatConversionRegistry::default();
    let gemini = json!({
        "model": "gemini-1.5-flash",
        "systemInstruction": { "parts": [{ "text": "sys" }] },
        "contents": [{ "role": "user", "parts": [{ "text": "hi" }] }],
        "generationConfig": { "temperature": 0.2, "maxOutputTokens": 12 },
        "stream": true
    });

    let openai = registry.convert_request(&gemini, ApiFormat::GeminiChat, ApiFormat::OpenAiChat).unwrap();
    assert_eq!(openai["messages"][0]["role"], "system");
    assert_eq!(openai["messages"][0]["content"], "sys");
    assert_eq!(openai["messages"][1]["content"], "hi");
    assert_eq!(openai["max_tokens"], 12);
    assert_eq!(openai["stream"], true);

    let claude = json!({
        "model": "claude-3-5-sonnet-latest",
        "system": "sys",
        "messages": [{ "role": "user", "content": "hi" }],
        "max_tokens": 12,
        "stream": true
    });
    let openai_from_claude = registry.convert_request(&claude, ApiFormat::ClaudeChat, ApiFormat::OpenAiChat).unwrap();
    assert_eq!(openai_from_claude["messages"][0]["role"], "system");
    assert_eq!(openai_from_claude["messages"][1]["role"], "user");
    assert_eq!(openai_from_claude["stream_options"]["include_usage"], true);
}

#[test]
fn format_conversion_response_maps_text_finish_and_usage() {
    let registry = FormatConversionRegistry::default();
    let openai = json!({
        "id": "chatcmpl_1",
        "model": "gpt-4o-mini",
        "object": "chat.completion",
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": "hello" },
            "finish_reason": "stop"
        }],
        "usage": { "prompt_tokens": 5, "completion_tokens": 7, "total_tokens": 12 }
    });

    let gemini = registry.convert_response(&openai, ApiFormat::OpenAiChat, ApiFormat::GeminiChat).unwrap();
    assert_eq!(gemini["id"], "chatcmpl_1");
    assert_eq!(gemini["candidates"][0]["content"]["parts"][0]["text"], "hello");
    assert_eq!(gemini["candidates"][0]["finishReason"], "STOP");
    assert_eq!(gemini["usageMetadata"]["totalTokenCount"], 12);

    let claude = registry.convert_response(&gemini, ApiFormat::GeminiChat, ApiFormat::ClaudeChat).unwrap();
    assert_eq!(claude["content"][0]["text"], "hello");
    assert_eq!(claude["stop_reason"], "end_turn");
    assert_eq!(claude["usage"]["input_tokens"], 5);
    assert_eq!(claude["usage"]["output_tokens"], 7);
}

#[test]
fn format_conversion_preserves_tools_results_and_multimodal_blocks() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gpt-4o-mini",
        "messages": [
            {
                "role": "user",
                "content": [
                    { "type": "text", "text": "describe" },
                    { "type": "image_url", "image_url": { "url": "data:image/png;base64,aW1n" } }
                ]
            },
            {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_1",
                    "type": "function",
                    "function": { "name": "lookup", "arguments": "{\"city\":\"杭州\"}" }
                }]
            },
            { "role": "tool", "tool_call_id": "call_1", "content": "{\"temp\":21}" }
        ],
        "tools": [{
            "type": "function",
            "function": {
                "name": "lookup",
                "description": "Lookup weather",
                "parameters": {
                    "type": "object",
                    "properties": { "city": { "type": "string" } }
                }
            }
        }],
        "tool_choice": { "type": "function", "function": { "name": "lookup" } }
    });

    let gemini = registry.convert_request(&input, ApiFormat::OpenAiChat, ApiFormat::GeminiChat).unwrap();
    assert_eq!(gemini["tools"][0]["functionDeclarations"][0]["name"], "lookup");
    assert_eq!(gemini["toolConfig"]["functionCallingConfig"]["allowedFunctionNames"][0], "lookup");
    assert_eq!(gemini["contents"][0]["parts"][1]["inlineData"]["mimeType"], "image/png");
    assert_eq!(gemini["contents"][1]["parts"][0]["functionCall"]["args"]["city"], "杭州");
    assert_eq!(gemini["contents"][2]["parts"][0]["functionResponse"]["response"]["result"]["temp"], 21);

    let claude = registry.convert_request(&input, ApiFormat::OpenAiChat, ApiFormat::ClaudeChat).unwrap();
    assert_eq!(claude["tools"][0]["name"], "lookup");
    assert_eq!(claude["tool_choice"]["name"], "lookup");
    assert_eq!(claude["messages"][0]["content"][1]["source"]["media_type"], "image/png");
    assert_eq!(claude["messages"][1]["content"][0]["type"], "tool_use");
    assert_eq!(claude["messages"][2]["content"][0]["type"], "tool_result");
    assert_eq!(claude["messages"][2]["content"][0]["content"]["temp"], 21);
}

#[test]
fn api_format_parse_requires_canonical_chat_cli_ids() {
    assert_eq!(ApiFormat::parse("openai:chat").unwrap(), ApiFormat::OpenAiChat);
    assert_eq!(ApiFormat::parse("openai:cli").unwrap(), ApiFormat::OpenAiResponses);
    assert_eq!(ApiFormat::parse("claude:cli").unwrap(), ApiFormat::ClaudeChat);
    assert!(ApiFormat::parse("openai_chat").is_err());
}

#[test]
fn responses_instructions_and_developer_messages_round_trip_to_chat() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gpt-5.5",
        "instructions": "system rules",
        "input": [
            { "type": "message", "role": "developer", "content": [{ "type": "input_text", "text": "dev rules" }] },
            { "type": "message", "role": "user", "content": [{ "type": "input_text", "text": "hi" }] }
        ],
        "stream": true
    });

    let chat = registry.convert_request(&input, ApiFormat::OpenAiResponses, ApiFormat::OpenAiChat).unwrap();
    assert_eq!(chat["messages"][0]["role"], "system");
    assert_eq!(chat["messages"][0]["content"], "system rules");
    assert_eq!(chat["messages"][1]["role"], "developer");
    assert_eq!(chat["messages"][1]["content"], "dev rules");

    let responses = registry.convert_request(&chat, ApiFormat::OpenAiChat, ApiFormat::OpenAiResponses).unwrap();
    assert_eq!(responses["instructions"], "system rules\n\ndev rules");
    assert_eq!(responses["input"][0]["role"], "user");
}

#[test]
fn response_conversion_preserves_thinking_tool_calls_and_cache_usage() {
    let registry = FormatConversionRegistry::default();
    let openai = json!({
        "id": "chatcmpl_1",
        "model": "gpt-5.5",
        "choices": [{
            "message": {
                "role": "assistant",
                "reasoning_content": "think",
                "content": "answer",
                "tool_calls": [{
                    "id": "call_1",
                    "type": "function",
                    "function": { "name": "lookup", "arguments": "{\"q\":\"x\"}" }
                }]
            },
            "finish_reason": "tool_calls"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "prompt_tokens_details": { "cached_tokens": 7, "cache_creation_tokens": 3 },
            "completion_tokens_details": { "reasoning_tokens": 2 }
        }
    });

    let claude = registry.convert_response(&openai, ApiFormat::OpenAiChat, ApiFormat::ClaudeChat).unwrap();
    assert_eq!(claude["content"][0]["type"], "thinking");
    assert_eq!(claude["content"][1]["text"], "answer");
    assert_eq!(claude["content"][2]["type"], "tool_use");
    assert_eq!(claude["usage"]["cache_read_input_tokens"], 7);
    assert_eq!(claude["usage"]["cache_creation_input_tokens"], 3);

    let responses = registry.convert_response(&claude, ApiFormat::ClaudeChat, ApiFormat::OpenAiResponses).unwrap();
    assert_eq!(responses["output_text"], "answer");
    assert_eq!(responses["usage"]["input_tokens_details"]["cached_tokens"], 7);
}

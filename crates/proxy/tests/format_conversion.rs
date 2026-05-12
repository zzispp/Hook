use proxy::format_conversion::{ApiFormat, FormatConversionError, FormatConversionRegistry, StreamChunkConversion, StreamConversionState};
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
    assert_eq!(gemini["system_instruction"]["parts"][0]["text"], "sys");
    assert_eq!(gemini["contents"][0]["role"], "user");
    assert_eq!(gemini["contents"][0]["parts"][0]["text"], "hi");
    assert_eq!(gemini["generation_config"]["max_output_tokens"], 12);

    let claude = registry.convert_request(&input, ApiFormat::OpenAiChat, ApiFormat::ClaudeChat).unwrap();
    assert_eq!(claude["system"], "sys");
    assert_eq!(claude["messages"][0]["role"], "user");
    assert_eq!(claude["messages"][0]["content"], "hi");
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
    assert_eq!(responses["input"][0]["content"], "hello");
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
fn format_conversion_stream_maps_delta_and_done() {
    let registry = FormatConversionRegistry::default();
    let openai = vec![
        json!({
            "id": "chatcmpl_1",
            "model": "gpt-4o-mini",
            "object": "chat.completion.chunk",
            "choices": [{ "index": 0, "delta": { "role": "assistant", "content": "He" }, "finish_reason": null }]
        }),
        json!({
            "id": "chatcmpl_1",
            "model": "gpt-4o-mini",
            "object": "chat.completion.chunk",
            "choices": [{ "index": 0, "delta": { "content": "llo" }, "finish_reason": "stop" }]
        }),
    ];

    let claude = registry.convert_stream(&openai, ApiFormat::OpenAiChat, ApiFormat::ClaudeChat).unwrap();
    assert_eq!(claude[0]["type"], "message_start");
    assert_eq!(claude[2]["delta"]["text"], "He");
    assert_eq!(claude[3]["delta"]["text"], "llo");
    assert_eq!(claude[5]["delta"]["stop_reason"], "end_turn");

    let gemini = registry.convert_stream(&claude, ApiFormat::ClaudeChat, ApiFormat::GeminiChat).unwrap();
    assert_eq!(gemini[0]["candidates"][0]["content"]["parts"][0]["text"], "He");
    assert_eq!(gemini[1]["candidates"][0]["content"]["parts"][0]["text"], "llo");
    assert_eq!(gemini[2]["candidates"][0]["finishReason"], "STOP");
}

#[test]
fn format_conversion_stream_chunk_matches_batch_for_cumulative_gemini_text() {
    let registry = FormatConversionRegistry::default();
    let gemini = vec![
        json!({
            "modelVersion": "gemini-1.5-flash",
            "candidates": [{ "content": { "parts": [{ "text": "He" }] } }]
        }),
        json!({
            "modelVersion": "gemini-1.5-flash",
            "candidates": [{ "content": { "parts": [{ "text": "Hello" }] } }]
        }),
        json!({
            "modelVersion": "gemini-1.5-flash",
            "candidates": [{ "content": { "parts": [{ "text": "Hello" }] }, "finishReason": "STOP" }],
            "usageMetadata": { "promptTokenCount": 2, "candidatesTokenCount": 1, "totalTokenCount": 3 }
        }),
    ];

    let batch = registry.convert_stream(&gemini, ApiFormat::GeminiChat, ApiFormat::OpenAiChat).unwrap();
    let mut state = StreamConversionState::default();
    let mut incremental = Vec::new();
    for chunk in &gemini {
        incremental.extend(
            registry
                .convert_stream_chunk(StreamChunkConversion {
                    chunk,
                    source: ApiFormat::GeminiChat,
                    target: ApiFormat::OpenAiChat,
                    state: &mut state,
                })
                .unwrap(),
        );
    }

    assert_eq!(incremental, batch);
    assert_eq!(incremental[1]["choices"][0]["delta"]["content"], "He");
    assert_eq!(incremental[2]["choices"][0]["delta"]["content"], "llo");
}

#[test]
fn format_conversion_rejects_unsupported_tools_explicitly() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gpt-4o-mini",
        "messages": [{ "role": "user", "content": "hi" }],
        "tools": [{ "type": "function" }]
    });

    let error = registry.convert_request(&input, ApiFormat::OpenAiChat, ApiFormat::GeminiChat).unwrap_err();
    assert!(matches!(error, FormatConversionError::UnsupportedFeature { .. }));
}

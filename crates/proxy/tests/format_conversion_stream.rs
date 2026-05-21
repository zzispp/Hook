use proxy::format_conversion::{ApiFormat, FormatConversionRegistry, StreamChunkConversion, StreamConversionState};
use serde_json::json;

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
    let stop = claude
        .iter()
        .find(|event| event["type"] == "message_delta")
        .expect("message_delta should exist");
    assert_eq!(claude[0]["type"], "message_start");
    assert_eq!(claude[0]["message"]["usage"]["input_tokens"], 0);
    assert_eq!(claude[0]["message"]["usage"]["output_tokens"], 0);
    assert_eq!(claude[2]["delta"]["text"], "He");
    assert_eq!(claude[3]["delta"]["text"], "llo");
    assert_eq!(stop["delta"]["stop_reason"], "end_turn");

    let gemini = registry.convert_stream(&claude, ApiFormat::ClaudeChat, ApiFormat::GeminiChat).unwrap();
    let text = gemini
        .iter()
        .filter_map(|event| event["candidates"][0]["content"]["parts"][0]["text"].as_str())
        .collect::<Vec<_>>();
    let finish = gemini
        .iter()
        .find(|event| event["candidates"][0].get("finishReason").is_some())
        .expect("finish chunk should exist");
    assert_eq!(text, vec!["He", "llo"]);
    assert_eq!(finish["candidates"][0]["finishReason"], "STOP");
}

#[test]
fn openai_tool_call_stream_to_claude_emits_tool_block() {
    let registry = FormatConversionRegistry::default();
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
    let registry = FormatConversionRegistry::default();
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
fn responses_function_call_stream_to_claude_emits_tool_block() {
    let registry = FormatConversionRegistry::default();
    let responses = vec![
        json!({ "type": "response.created", "response": { "id": "resp_1", "model": "gpt-5.5" } }),
        json!({
            "type": "response.output_item.added",
            "output_index": 0,
            "item": { "type": "function_call", "id": "fc_1", "call_id": "call_1", "name": "lookup", "arguments": "" }
        }),
        json!({
            "type": "response.function_call_arguments.delta",
            "item_id": "fc_1",
            "output_index": 0,
            "delta": "{\"q\""
        }),
        json!({
            "type": "response.function_call_arguments.done",
            "item_id": "fc_1",
            "output_index": 0,
            "arguments": "{\"q\":\"eth\"}"
        }),
        json!({
            "type": "response.output_item.done",
            "output_index": 0,
            "item": {
                "type": "function_call",
                "id": "fc_1",
                "call_id": "call_1",
                "name": "lookup",
                "arguments": "{\"q\":\"eth\"}"
            }
        }),
        json!({ "type": "response.completed", "response": { "id": "resp_1", "model": "gpt-5.5" } }),
    ];

    let claude = registry.convert_stream(&responses, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat).unwrap();
    let tool_start = claude
        .iter()
        .find(|event| event["type"] == "content_block_start" && event["content_block"]["type"] == "tool_use")
        .expect("tool_use block should start");
    let deltas = claude
        .iter()
        .filter(|event| event["type"] == "content_block_delta" && event["delta"]["type"] == "input_json_delta")
        .collect::<Vec<_>>();

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
fn format_conversion_stream_openai_usage_only_chunk_completes_responses_usage() {
    let registry = FormatConversionRegistry::default();
    let openai = vec![
        json!({
            "id": "chatcmpl_1",
            "model": "gpt-5.5",
            "object": "chat.completion.chunk",
            "choices": [{ "index": 0, "delta": { "role": "assistant", "content": "Hi" }, "finish_reason": null }]
        }),
        json!({
            "id": "chatcmpl_1",
            "model": "gpt-5.5",
            "object": "chat.completion.chunk",
            "choices": [{ "index": 0, "delta": {}, "finish_reason": "stop" }]
        }),
        json!({
            "id": "chatcmpl_1",
            "model": "gpt-5.5",
            "object": "chat.completion.chunk",
            "choices": [],
            "usage": { "prompt_tokens": 11, "completion_tokens": 4, "total_tokens": 15 }
        }),
    ];

    let responses = registry.convert_stream(&openai, ApiFormat::OpenAiChat, ApiFormat::OpenAiResponses).unwrap();

    assert_eq!(responses[0]["type"], "response.created");
    assert!(responses.iter().any(|event| event["type"] == "response.output_item.added"));
    assert!(responses.iter().any(|event| event["type"] == "response.content_part.added"));
    assert!(responses.iter().any(|event| event["type"] == "response.output_text.done"));
    assert!(responses.iter().any(|event| event["type"] == "response.content_part.done"));
    assert!(responses.iter().any(|event| event["type"] == "response.output_item.done"));
    let delta = responses.iter().find(|event| event["type"] == "response.output_text.delta").unwrap();
    assert_eq!(delta["delta"], "Hi");
    let completed = responses.iter().find(|event| event["type"] == "response.completed").unwrap();
    assert_eq!(completed["response"]["usage"]["input_tokens"], 11);
    assert_eq!(completed["response"]["usage"]["output_tokens"], 4);
    assert_eq!(completed["response"]["usage"]["total_tokens"], 15);
    assert_eq!(completed["response"]["output"][0]["content"][0]["text"], "Hi");
}

#[test]
fn format_conversion_stream_openai_responses_completed_omits_null_usage_details() {
    let registry = FormatConversionRegistry::default();
    let claude = vec![
        json!({
            "type": "message_start",
            "message": {
                "id": "msg_1",
                "model": "claude-3-5-sonnet-latest",
                "usage": { "input_tokens": 11, "output_tokens": 0 }
            }
        }),
        json!({ "type": "content_block_start", "index": 0, "content_block": { "type": "text", "text": "" } }),
        json!({ "type": "content_block_delta", "index": 0, "delta": { "type": "text_delta", "text": "hi" } }),
        json!({ "type": "content_block_stop", "index": 0 }),
        json!({
            "type": "message_delta",
            "delta": { "stop_reason": "end_turn" },
            "usage": { "input_tokens": 11, "output_tokens": 2 }
        }),
    ];

    let responses = registry.convert_stream(&claude, ApiFormat::ClaudeChat, ApiFormat::OpenAiResponses).unwrap();
    let completed = responses
        .iter()
        .find(|event| event["type"] == "response.completed")
        .expect("completed event should exist");

    assert_eq!(completed["response"]["usage"]["input_tokens"], 11);
    assert_eq!(completed["response"]["usage"]["output_tokens"], 2);
    assert!(completed["response"]["usage"].get("output_tokens_details").is_none());
}

#[test]
fn format_conversion_stream_openai_finish_waits_for_usage_only_incremental_chunk() {
    let registry = FormatConversionRegistry::default();
    let mut state = StreamConversionState::default();
    let finish = json!({
        "id": "chatcmpl_1",
        "model": "gpt-5.5",
        "object": "chat.completion.chunk",
        "choices": [{ "index": 0, "delta": {}, "finish_reason": "stop" }]
    });
    let usage = json!({
        "id": "chatcmpl_1",
        "model": "gpt-5.5",
        "object": "chat.completion.chunk",
        "choices": [],
        "usage": { "prompt_tokens": 9, "completion_tokens": 3, "total_tokens": 12 }
    });

    let first = registry
        .convert_stream_chunk(StreamChunkConversion {
            chunk: &finish,
            source: ApiFormat::OpenAiChat,
            target: ApiFormat::ClaudeChat,
            state: &mut state,
        })
        .unwrap();
    let second = registry
        .convert_stream_chunk(StreamChunkConversion {
            chunk: &usage,
            source: ApiFormat::OpenAiChat,
            target: ApiFormat::ClaudeChat,
            state: &mut state,
        })
        .unwrap();

    assert!(first.is_empty());
    assert_eq!(second[0]["type"], "content_block_stop");
    assert_eq!(second[1]["usage"]["input_tokens"], 9);
    assert_eq!(second[1]["usage"]["output_tokens"], 3);
}

#[test]
fn format_conversion_stream_flushes_openai_done_when_usage_chunk_absent() {
    let registry = FormatConversionRegistry::default();
    let mut state = StreamConversionState::default();
    let finish = json!({
        "id": "chatcmpl_1",
        "model": "gpt-5.5",
        "object": "chat.completion.chunk",
        "choices": [{ "index": 0, "delta": {}, "finish_reason": "stop" }]
    });

    let first = registry
        .convert_stream_chunk(StreamChunkConversion {
            chunk: &finish,
            source: ApiFormat::OpenAiChat,
            target: ApiFormat::OpenAiResponses,
            state: &mut state,
        })
        .unwrap();
    let flushed = registry.flush_stream(ApiFormat::OpenAiChat, ApiFormat::OpenAiResponses, &mut state).unwrap();

    assert!(first.is_empty());
    assert_eq!(flushed[0]["type"], "response.completed");
    assert!(flushed[0]["response"].get("usage").is_none());
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
fn stream_conversion_maps_thinking_signature_tool_delta_and_usage() {
    let registry = FormatConversionRegistry::default();
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
        .find(|event| event["choices"][0]["delta"].get("tool_calls").is_some())
        .expect("tool delta should exist");
    assert_eq!(tool_delta["choices"][0]["delta"]["tool_calls"][0]["function"]["arguments"], "{\"q\"");
    let usage = openai.iter().find(|event| event.get("usage").is_some()).expect("usage chunk should exist");
    assert_eq!(usage["usage"]["prompt_tokens_details"]["cached_tokens"], 2);
}

#[test]
fn claude_tool_use_stream_to_gemini_finishes_with_stop() {
    let registry = FormatConversionRegistry::default();
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

#[test]
fn gemini_stream_output_omits_null_signature_and_usage_fields() {
    let registry = FormatConversionRegistry::default();
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
fn responses_stream_reasoning_preserves_claude_thinking_signature_for_next_request() {
    let registry = FormatConversionRegistry::default();
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
    assert_eq!(reasoning_done["item"]["encrypted_content"], "sig_1");

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
    assert_eq!(claude_request["messages"][1]["content"][0]["type"], "thinking");
    assert_eq!(claude_request["messages"][1]["content"][0]["thinking"], "plan");
    assert_eq!(claude_request["messages"][1]["content"][0]["signature"], "sig_1");
}

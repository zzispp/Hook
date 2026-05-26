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
    let delta = second
        .iter()
        .find(|event| event["type"] == "message_delta")
        .expect("message_delta should carry usage");
    assert_eq!(delta["usage"]["input_tokens"], 9);
    assert_eq!(delta["usage"]["output_tokens"], 3);
    assert!(second.iter().any(|event| event["type"] == "message_stop"));
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
    let completed = flushed
        .iter()
        .find(|event| event["type"] == "response.completed")
        .expect("response.completed should be emitted");
    assert_eq!(completed["response"]["usage"]["input_tokens"], 0);
    assert_eq!(completed["response"]["usage"]["output_tokens"], 0);
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

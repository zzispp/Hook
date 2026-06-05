use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::json;

#[test]
fn responses_function_call_stream_to_claude_emits_tool_block() {
    let registry = FormatConversionRegistry;
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
fn responses_custom_tool_stream_to_claude_errors_visibly() {
    let registry = FormatConversionRegistry;
    let responses = vec![
        json!({ "type": "response.created", "response": { "id": "resp_1", "model": "gpt-5.5" } }),
        json!({
            "type": "response.output_item.added",
            "output_index": 0,
            "item": {
                "type": "custom_tool_call",
                "id": "ct_1",
                "call_id": "call_1",
                "name": "apply_patch",
                "input": ""
            }
        }),
        json!({
            "type": "response.custom_tool_call_input.delta",
            "item_id": "ct_1",
            "call_id": "call_1",
            "delta": "*** Begin"
        }),
        json!({
            "type": "response.custom_tool_call_input.delta",
            "item_id": "ct_1",
            "call_id": "call_1",
            "delta": " Patch"
        }),
        json!({
            "type": "response.output_item.done",
            "output_index": 0,
            "item": {
                "type": "custom_tool_call",
                "id": "ct_1",
                "call_id": "call_1",
                "name": "apply_patch",
                "input": "*** Begin Patch"
            }
        }),
        json!({ "type": "response.completed", "response": { "id": "resp_1", "model": "gpt-5.5" } }),
    ];

    let error = registry
        .convert_stream(&responses, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat)
        .unwrap_err()
        .to_string();

    assert!(error.contains("unsupported output item type custom_tool_call"));
}

#[test]
fn responses_stream_unsupported_official_item_errors() {
    let registry = FormatConversionRegistry;
    let responses = vec![
        json!({ "type": "response.created", "response": { "id": "resp_1", "model": "gpt-5.5" } }),
        json!({
            "type": "response.output_item.added",
            "output_index": 0,
            "item": { "type": "tool_search_call", "call_id": "search_1" }
        }),
    ];

    let error = registry
        .convert_stream(&responses, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat)
        .unwrap_err()
        .to_string();

    assert!(error.contains("unsupported output item type tool_search_call"));
}

#[test]
fn format_conversion_stream_openai_usage_only_chunk_completes_responses_usage() {
    let registry = FormatConversionRegistry;
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
    let registry = FormatConversionRegistry;
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

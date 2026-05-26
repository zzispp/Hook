use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::json;

#[test]
fn format_conversion_openai_responses_tool_output_content_items_to_claude() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gpt-5.5",
        "input": [
            { "type": "message", "role": "user", "content": [{ "type": "input_text", "text": "inspect file" }] },
            { "type": "function_call", "call_id": "call_1", "name": "inspect", "arguments": "{\"path\":\"/tmp/a.png\"}" },
            {
                "type": "function_call_output",
                "call_id": "call_1",
                "output": [
                    { "type": "input_text", "text": "line 1" },
                    { "type": "input_text", "text": "line 2" }
                ]
            }
        ]
    });

    let claude = registry.convert_request(&input, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat).unwrap();

    assert_eq!(claude["messages"][2]["content"][0]["type"], "tool_result");
    assert_eq!(claude["messages"][2]["content"][0]["tool_use_id"], "call_1");
    assert_eq!(
        claude["messages"][2]["content"][0]["content"],
        "[{\"text\":\"line 1\",\"type\":\"input_text\"},{\"text\":\"line 2\",\"type\":\"input_text\"}]"
    );
}

#[test]
fn format_conversion_openai_responses_tool_output_image_items_to_claude() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gpt-5.5",
        "input": [
            { "type": "message", "role": "user", "content": [{ "type": "input_text", "text": "inspect screenshot" }] },
            { "type": "function_call", "call_id": "call_1", "name": "inspect", "arguments": "{}" },
            {
                "type": "function_call_output",
                "call_id": "call_1",
                "output": [
                    { "type": "input_text", "text": "screenshot" },
                    { "type": "input_image", "image_url": "data:image/png;base64,aW1n" }
                ]
            }
        ]
    });

    let claude = registry.convert_request(&input, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat).unwrap();

    assert_eq!(
        claude["messages"][2]["content"][0]["content"],
        "[{\"text\":\"screenshot\",\"type\":\"input_text\"},{\"image_url\":\"data:image/png;base64,aW1n\",\"type\":\"input_image\"}]"
    );
}

#[test]
fn format_conversion_openai_responses_custom_tool_round_trips() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "gpt-5.5",
        "input": [
            { "type": "custom_tool_call", "call_id": "call_1", "name": "apply_patch", "input": "*** Begin Patch" },
            { "type": "custom_tool_call_output", "call_id": "call_1", "name": "apply_patch", "output": "patched" }
        ]
    });

    let responses = registry
        .convert_request(&input, ApiFormat::OpenAiResponses, ApiFormat::OpenAiResponses)
        .unwrap();
    assert_eq!(responses["input"][0]["type"], "custom_tool_call");
    assert_eq!(responses["input"][0]["input"], "*** Begin Patch");
    assert_eq!(responses["input"][1]["type"], "custom_tool_call_output");
    assert_eq!(responses["input"][1]["output"], "patched");

    let openai_error = registry
        .convert_request(&input, ApiFormat::OpenAiResponses, ApiFormat::OpenAiChat)
        .unwrap_err()
        .to_string();
    assert!(openai_error.contains("unsupported input item type custom_tool_call"));
}

#[test]
fn format_conversion_openai_responses_response_custom_tool_round_trips_to_responses() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "id": "resp_1",
        "model": "gpt-5.5",
        "output": [{
            "type": "custom_tool_call",
            "call_id": "call_1",
            "name": "apply_patch",
            "input": "*** Begin Patch"
        }]
    });

    let responses = registry
        .convert_response(&input, ApiFormat::OpenAiResponses, ApiFormat::OpenAiResponses)
        .unwrap();
    assert_eq!(responses["output"][0]["type"], "custom_tool_call");
    assert_eq!(responses["output"][0]["input"], "*** Begin Patch");

    let claude_error = registry
        .convert_response(&input, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat)
        .unwrap_err()
        .to_string();
    assert!(claude_error.contains("unsupported output item type custom_tool_call"));
}

#[test]
fn format_conversion_openai_responses_response_prefers_output_items_over_output_text() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "id": "resp_1",
        "model": "gpt-5.5",
        "output_text": "checking",
        "output": [
            {
                "type": "message",
                "role": "assistant",
                "content": [{ "type": "output_text", "text": "checking" }]
            },
            {
                "type": "function_call",
                "call_id": "call_1",
                "name": "lookup",
                "arguments": "{\"q\":\"eth\"}"
            }
        ]
    });

    let claude = registry.convert_response(&input, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat).unwrap();

    assert_eq!(claude["content"][0]["type"], "text");
    assert_eq!(claude["content"][0]["text"], "checking");
    assert_eq!(claude["content"][1]["type"], "tool_use");
    assert_eq!(claude["content"][1]["id"], "call_1");
    assert_eq!(claude["content"][1]["input"]["q"], "eth");
}

#[test]
fn format_conversion_openai_responses_response_from_claude_preserves_items() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "id": "msg_1",
        "model": "claude-sonnet",
        "content": [
            { "type": "thinking", "thinking": "plan", "signature": "sig_1" },
            { "type": "text", "text": "checking" },
            { "type": "tool_use", "id": "call_1", "name": "lookup", "input": { "q": "x" } }
        ]
    });

    let responses = registry.convert_response(&input, ApiFormat::ClaudeChat, ApiFormat::OpenAiResponses).unwrap();

    assert_eq!(responses["output"][0]["type"], "reasoning");
    assert_eq!(responses["output"][0]["summary"][0]["text"], "plan");
    assert_eq!(responses["output"][1]["type"], "message");
    assert_eq!(responses["output"][1]["content"][0]["type"], "output_text");
    assert_eq!(responses["output"][1]["content"][0]["text"], "checking");
    assert_eq!(responses["output"][2]["type"], "function_call");
    assert_eq!(responses["output"][2]["call_id"], "call_1");
    assert_eq!(responses["output"][2]["arguments"], "{\"q\":\"x\"}");
    assert!(responses.get("output_text").is_none());
}

#[test]
fn format_conversion_openai_responses_unsupported_official_items_error() {
    let registry = FormatConversionRegistry::default();
    let request = json!({
        "model": "gpt-5.5",
        "input": [{
            "type": "tool_search_call",
            "call_id": "search_1",
            "execution": "client",
            "arguments": { "query": "calendar" }
        }]
    });
    let response = json!({
        "id": "resp_1",
        "model": "gpt-5.5",
        "output": [{
            "type": "tool_search_call",
            "call_id": "search_1",
            "execution": "client",
            "arguments": { "query": "calendar" }
        }]
    });

    let request_error = registry
        .convert_request(&request, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat)
        .unwrap_err()
        .to_string();
    let response_error = registry
        .convert_response(&response, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat)
        .unwrap_err()
        .to_string();

    assert!(request_error.contains("unsupported input item type tool_search_call"));
    assert!(response_error.contains("unsupported output item type tool_search_call"));
}

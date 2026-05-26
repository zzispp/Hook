use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::json;

#[test]
fn registry_delegates_standard_request_conversion_to_formats_crate() {
    let input = json!({
        "model": "gpt-5.5",
        "messages": [{ "role": "user", "content": "hello" }],
        "max_tokens": 16
    });

    let claude = FormatConversionRegistry::default()
        .convert_request(&input, ApiFormat::OpenAiChat, ApiFormat::ClaudeChat)
        .unwrap();

    assert_eq!(claude["model"], "gpt-5.5");
    assert_eq!(claude["messages"][0]["role"], "user");
    assert_eq!(claude["messages"][0]["content"], "hello");
    assert_eq!(claude["max_tokens"], 16);
}

#[test]
fn registry_preserves_same_format_payloads_without_normalization() {
    let input = json!({
        "model": "gpt-5.5",
        "input": [{
            "type": "custom_tool_call",
            "call_id": "call_1",
            "name": "apply_patch",
            "input": "*** Begin Patch"
        }]
    });

    let output = FormatConversionRegistry::default()
        .convert_request(&input, ApiFormat::OpenAiResponses, ApiFormat::OpenAiResponses)
        .unwrap();

    assert_eq!(output, input);
}

#[test]
fn registry_rejects_unknown_openai_responses_items_for_cross_format_conversion() {
    let input = json!({
        "model": "gpt-5.5",
        "input": [{
            "type": "custom_tool_call",
            "call_id": "call_1",
            "name": "apply_patch",
            "input": "*** Begin Patch"
        }]
    });

    let error = FormatConversionRegistry::default()
        .convert_request(&input, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat)
        .unwrap_err()
        .to_string();

    assert!(error.contains("unsupported input item type custom_tool_call"));
}

#[test]
fn api_format_keeps_openai_responses_compact_distinct() {
    assert_eq!(ApiFormat::parse("openai:compact").unwrap(), ApiFormat::OpenAiResponsesCompact);
    assert_eq!(ApiFormat::OpenAiResponsesCompact.as_format_id().unwrap(), "openai:responses:compact");
}

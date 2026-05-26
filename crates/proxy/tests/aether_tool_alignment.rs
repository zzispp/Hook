use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::json;

#[test]
fn unsupported_openai_custom_tool_request_fails_visibly() {
    let openai = json!({
        "model": "gpt-5.5",
        "messages": [{ "role": "user", "content": "hi" }],
        "tools": [{
            "type": "custom",
            "custom": {
                "name": "shell",
                "description": "run shell",
                "format": { "type": "text" }
            }
        }],
        "tool_choice": { "type": "custom", "custom": { "name": "shell" } }
    });

    let error = FormatConversionRegistry::default()
        .convert_request(&openai, ApiFormat::OpenAiChat, ApiFormat::OpenAiResponses)
        .unwrap_err()
        .to_string();

    assert!(error.contains("unsupported content in request: openai:chat"));
}

#[test]
fn responses_web_search_tool_converts_through_formats_crate() {
    let responses = json!({
        "model": "gpt-5.5",
        "input": "hi",
        "tools": [{
            "type": "web_search_preview",
            "search_context_size": "high",
            "user_location": { "type": "approximate", "country": "US" }
        }]
    });

    let claude = FormatConversionRegistry::default()
        .convert_request(&responses, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat)
        .unwrap();

    assert_eq!(claude["tools"][0]["name"], "web_search_preview");
    assert_eq!(claude["tools"][0]["input_schema"]["type"], "object");
}

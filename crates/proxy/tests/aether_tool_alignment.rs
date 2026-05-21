use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::json;

#[test]
fn openai_custom_and_responses_hosted_tools_align_with_aether() {
    let registry = FormatConversionRegistry::default();
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
    let responses = json!({
        "model": "gpt-5.5",
        "input": "hi",
        "tools": [{
            "type": "web_search_preview",
            "search_context_size": "high",
            "user_location": { "type": "approximate", "country": "US" }
        }],
        "tool_choice": { "type": "custom", "name": "shell" }
    });

    let responses_from_chat = registry.convert_request(&openai, ApiFormat::OpenAiChat, ApiFormat::OpenAiResponses).unwrap();
    let claude_from_responses = registry.convert_request(&responses, ApiFormat::OpenAiResponses, ApiFormat::ClaudeChat).unwrap();
    let gemini_from_responses = registry.convert_request(&responses, ApiFormat::OpenAiResponses, ApiFormat::GeminiChat).unwrap();

    assert_eq!(responses_from_chat["tools"][0]["type"], "custom");
    assert_eq!(responses_from_chat["tools"][0]["name"], "shell");
    assert_eq!(responses_from_chat["tool_choice"]["type"], "function");
    assert_eq!(responses_from_chat["tool_choice"]["name"], "shell");
    assert_eq!(claude_from_responses["tools"][0]["type"], "web_search_20250305");
    assert_eq!(claude_from_responses["tools"][0]["max_uses"], 10);
    assert_eq!(gemini_from_responses["tools"][0]["googleSearch"], json!({}));
    assert_eq!(gemini_from_responses["toolConfig"]["functionCallingConfig"]["allowedFunctionNames"][0], "shell");
}

use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::json;

#[test]
fn openai_target_preserves_tool_schema_from_formats_crate() {
    let registry = FormatConversionRegistry::default();
    let input = json!({
        "model": "claude-sonnet",
        "tools": [{
            "name": "search",
            "input_schema": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "filter": { "type": "object" }
                }
            }
        }],
        "messages": [{ "role": "user", "content": "search" }]
    });

    let openai = registry.convert_request(&input, ApiFormat::ClaudeChat, ApiFormat::OpenAiChat).unwrap();
    let parameters = &openai["tools"][0]["function"]["parameters"];
    let filter = &parameters["properties"]["filter"];

    assert_eq!(parameters["additionalProperties"], false);
    assert_eq!(filter["type"], "object");
    assert!(filter.get("properties").is_none());
}

#[test]
fn gemini_target_recursively_adds_missing_object_properties() {
    let registry = FormatConversionRegistry::default();
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "query": { "type": ["string", "null"], "minLength": 2, "format": "uri" },
            "mode": { "enum": [1, null, "auto"] },
            "target": {
                "anyOf": [
                    { "type": "null" },
                    { "type": "array", "items": { "$ref": "#/$defs/Item" } },
                    { "type": "object", "properties": { "id": { "type": "string" } }, "required": ["id", "missing"] }
                ]
            }
        },
        "required": ["query", "mode", "missing"],
        "$defs": {
            "Item": { "type": "object", "properties": { "name": { "type": "string", "default": "x" } } }
        }
    });
    let input = json!({
        "model": "gpt-5.5",
        "messages": [{ "role": "user", "content": "json" }],
        "tools": [{ "type": "function", "function": { "name": "search", "parameters": schema.clone() } }],
        "response_format": { "type": "json_schema", "json_schema": { "schema": schema } }
    });

    let gemini = registry.convert_request(&input, ApiFormat::OpenAiChat, ApiFormat::GeminiChat).unwrap();
    let parameters = &gemini["tools"][0]["functionDeclarations"][0]["parameters"];
    let response_schema = &gemini["generationConfig"]["responseSchema"];

    assert_eq!(parameters["$schema"], "https://json-schema.org/draft/2020-12/schema");
    assert_eq!(parameters["additionalProperties"], false);
    assert_eq!(parameters["required"], json!(["query", "mode", "missing"]));
    assert_eq!(parameters["properties"]["query"]["type"], json!(["string", "null"]));
    assert_eq!(parameters["properties"]["query"]["minLength"], 2);
    assert_eq!(parameters["properties"]["query"]["format"], "uri");
    assert_eq!(parameters["properties"]["mode"]["enum"], json!([1, null, "auto"]));
    assert_eq!(parameters["properties"]["target"]["anyOf"][2]["properties"]["id"]["type"], "string");
    assert_eq!(parameters["$defs"]["Item"]["properties"]["name"]["type"], "string");
    assert_eq!(response_schema, parameters);
}

use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::json;

#[test]
fn openai_target_adds_properties_to_object_tool_schema() {
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
    let filter = &openai["tools"][0]["function"]["parameters"]["properties"]["filter"];

    assert_eq!(filter["type"], "object");
    assert_eq!(filter["properties"], json!({}));
}

#[test]
fn gemini_target_cleans_tool_and_response_schemas_like_aether() {
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

    assert!(parameters.get("$schema").is_none());
    assert!(parameters.get("$defs").is_none());
    assert!(parameters.get("additionalProperties").is_none());
    assert_eq!(parameters["required"], json!(["mode"]));
    assert_eq!(parameters["properties"]["query"]["type"], "string");
    assert_eq!(
        parameters["properties"]["query"]["description"],
        "[Constraint: minLen: 2, format: \"uri\"] (nullable)"
    );
    assert_eq!(parameters["properties"]["mode"]["enum"], json!(["1", "null", "auto"]));
    assert_eq!(parameters["properties"]["target"]["type"], "object");
    assert_eq!(parameters["properties"]["target"]["required"], json!(["id"]));
    assert_eq!(parameters["properties"]["target"]["description"], "Accepts: string | array | object");
    assert_eq!(response_schema, parameters);
}

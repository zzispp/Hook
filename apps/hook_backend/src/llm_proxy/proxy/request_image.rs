use proxy::format_conversion::ApiFormat;
use serde_json::{Map, Value};

use crate::llm_proxy::{LlmProxyError, OPENAI_CHAT_FORMAT, OPENAI_CLI_FORMAT, candidate::ProxyCandidate, formats};

pub(super) fn provider_is_openai_image(candidate: &ProxyCandidate) -> bool {
    formats::endpoint_metadata(&candidate.trace.provider_api_format, false)
        .map(|metadata| metadata.data_format == ApiFormat::OpenAiImage)
        .unwrap_or(false)
}

pub(super) fn client_is_openai_chat_or_responses(candidate: &ProxyCandidate) -> bool {
    matches!(candidate.trace.client_api_format.as_str(), OPENAI_CHAT_FORMAT | OPENAI_CLI_FORMAT)
}

pub(super) fn client_source_format(candidate: &ProxyCandidate) -> Result<ApiFormat, LlmProxyError> {
    formats::endpoint_metadata(&candidate.trace.client_api_format, candidate.trace.is_stream).map(|metadata| metadata.data_format)
}

pub(super) fn openai_image_bridge_body(body: Value, candidate: &ProxyCandidate) -> Result<Value, LlmProxyError> {
    if candidate.trace.client_api_format == OPENAI_CLI_FORMAT {
        return Ok(body);
    }
    bridge_openai_chat_image_body(body)
}

pub(super) fn bridge_openai_chat_image_body(body: Value) -> Result<Value, LlmProxyError> {
    let object = body
        .as_object()
        .ok_or_else(|| LlmProxyError::InvalidRequest("request body must be a JSON object".into()))?;
    let messages = object
        .get("messages")
        .cloned()
        .ok_or_else(|| LlmProxyError::InvalidRequest("openai chat image request must include messages".into()))?;
    let mut bridged = Map::new();
    bridged.insert("input".to_string(), messages);
    bridged.insert("tools".to_string(), bridge_openai_chat_image_tools(object));
    bridged.insert("tool_choice".to_string(), Value::Object(image_tool_choice()));
    copy_optional_field(object, &mut bridged, "user");
    copy_optional_field(object, &mut bridged, "n");
    Ok(Value::Object(bridged))
}

fn bridge_openai_chat_image_tools(object: &Map<String, Value>) -> Value {
    let tools = object.get("tools").and_then(Value::as_array).cloned().unwrap_or_default();
    if tools.iter().any(image_generation_tool) {
        return Value::Array(tools);
    }
    let mut output = tools;
    output.push(Value::Object(image_tool_choice()));
    Value::Array(output)
}

fn image_generation_tool(tool: &Value) -> bool {
    tool.get("type")
        .and_then(Value::as_str)
        .is_some_and(|kind| kind.trim().eq_ignore_ascii_case("image_generation"))
}

fn image_tool_choice() -> Map<String, Value> {
    Map::from_iter([("type".to_string(), Value::String("image_generation".to_string()))])
}

fn copy_optional_field(source: &Map<String, Value>, target: &mut Map<String, Value>, key: &str) {
    if let Some(value) = source.get(key) {
        target.insert(key.to_string(), value.clone());
    }
}

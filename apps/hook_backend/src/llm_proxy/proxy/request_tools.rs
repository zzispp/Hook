use serde_json::Value;

use crate::llm_proxy::{IMAGE_GENERATION_CAPABILITY, OPENAI_CHAT_FORMAT, OPENAI_CLI_FORMAT, candidate::ProxyCandidate, capabilities::json_capability_enabled};

pub(super) fn prune_unsupported_image_generation_tool(body: &mut Value, candidate: &ProxyCandidate) {
    if !should_prune_image_generation_tool(body, candidate) {
        return;
    }
    let Some(object) = body.as_object_mut() else {
        return;
    };
    let Some(tools) = object.get_mut("tools").and_then(Value::as_array_mut) else {
        return;
    };
    tools.retain(|tool| !is_image_generation_tool(tool));
    if tools.is_empty() {
        object.remove("tools");
    }
}

fn should_prune_image_generation_tool(body: &Value, candidate: &ProxyCandidate) -> bool {
    client_may_declare_tools(candidate)
        && provider_may_receive_tools(candidate)
        && !key_supports_image_generation(candidate)
        && !openai_request_explicitly_selects_image_generation(&candidate.trace.client_api_format, body)
}

fn client_may_declare_tools(candidate: &ProxyCandidate) -> bool {
    matches!(candidate.trace.client_api_format.as_str(), OPENAI_CHAT_FORMAT | OPENAI_CLI_FORMAT)
}

fn provider_may_receive_tools(candidate: &ProxyCandidate) -> bool {
    matches!(candidate.trace.provider_api_format.as_str(), OPENAI_CHAT_FORMAT | OPENAI_CLI_FORMAT)
}

fn key_supports_image_generation(candidate: &ProxyCandidate) -> bool {
    json_capability_enabled(candidate.key_capabilities.as_ref(), IMAGE_GENERATION_CAPABILITY)
}

pub(super) fn openai_request_explicitly_selects_image_generation(api_format: &str, body: &Value) -> bool {
    if !matches!(api_format, OPENAI_CHAT_FORMAT | OPENAI_CLI_FORMAT) {
        return false;
    }
    let Some(object) = body.as_object() else {
        return false;
    };
    match object.get("tool_choice") {
        Some(Value::String(name)) => name.trim().eq_ignore_ascii_case("image_generation"),
        Some(Value::Object(choice)) => choice
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(|kind| kind.trim().eq_ignore_ascii_case("image_generation")),
        _ => false,
    }
}

fn is_image_generation_tool(tool: &Value) -> bool {
    tool.get("type")
        .and_then(Value::as_str)
        .is_some_and(|kind| kind.trim().eq_ignore_ascii_case("image_generation"))
}

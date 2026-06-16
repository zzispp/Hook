use axum::http::HeaderMap;
use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::{Map, Value};
use types::api_token::ApiToken;

use crate::llm_proxy::{
    IMAGE_GENERATION_CAPABILITY, LlmProxyError, LlmProxyState, OPENAI_CHAT_FORMAT, OPENAI_CLI_FORMAT, OPENAI_IMAGE_EDIT_FORMAT, OPENAI_IMAGE_FORMAT,
    audit::record_scheduled_candidates,
    billing::enforce_preflight_access,
    candidate::{CandidateRequest, CandidateSelection, ProxyCandidate, select_candidates},
    formats, rate_limit,
    rate_limit::ProviderKeyProbeSlotOptions,
};

use super::{body_rules::apply_provider_body_rules, capture::RequestCapture, request_features::routing_request_features};

pub(super) struct PreparedProxyRequest {
    pub(super) request_id: String,
    pub(super) cache_affinity_ttl_minutes: i64,
    pub(super) candidates: Vec<ProxyCandidate>,
    pub(super) body: Value,
    pub(super) service_tier: Option<String>,
    pub(super) is_stream: bool,
    pub(super) force_non_stream: bool,
    pub(super) provider_key_probe_slot_options: Option<ProviderKeyProbeSlotOptions>,
    pub(super) provider_headers: HeaderMap,
}

pub(super) struct AttemptPayload {
    pub(super) body: Value,
    pub(super) original_body: Value,
    pub(super) source_format: ApiFormat,
    pub(super) target_format: ApiFormat,
}

pub(super) async fn prepare_proxy_request(
    state: &LlmProxyState,
    token: &ApiToken,
    body: Value,
    api_format: &'static str,
    force_non_stream: bool,
    provider_key_probe_slot_options: Option<ProviderKeyProbeSlotOptions>,
    capture: RequestCapture,
) -> Result<PreparedProxyRequest, LlmProxyError> {
    let model_name = required_model(&body)?;
    enforce_preflight_access(state, token).await?;
    rate_limit::enforce_request_limits(state, token).await?;
    let is_stream = is_streaming(&body) && !force_non_stream;
    let routing_api_format = routing_api_format(api_format, &body);
    let required_capability = required_capability_for_routing(routing_api_format);
    let features = routing_request_features(api_format, &body, model_name, is_stream, required_capability)?;
    let selection = select_candidates(
        state,
        token,
        CandidateRequest {
            api_format,
            routing_api_format,
            model_name,
            is_stream,
            has_openai_responses_custom_tool_items: has_openai_responses_custom_tool_items(api_format, &body),
            required_capability,
            features,
        },
    )
    .await?;
    record_scheduled_candidates(state, &selection, &capture).await?;
    let service_tier = capture.service_tier();
    Ok(PreparedProxyRequest {
        request_id: selection.request_id,
        cache_affinity_ttl_minutes: selection.cache_affinity_ttl_minutes,
        candidates: selection.candidates,
        body,
        service_tier,
        is_stream,
        force_non_stream,
        provider_key_probe_slot_options,
        provider_headers: HeaderMap::new(),
    })
}

pub(super) async fn prepare_proxy_request_with_candidates(
    state: &LlmProxyState,
    body: Value,
    api_format: &str,
    force_non_stream: bool,
    provider_headers: HeaderMap,
    selection: CandidateSelection,
    capture: RequestCapture,
) -> Result<PreparedProxyRequest, LlmProxyError> {
    required_model(&body)?;
    let is_stream = is_streaming(&body) && !force_non_stream;
    ensure_selection_format(&selection, api_format, is_stream)?;
    record_scheduled_candidates(state, &selection, &capture).await?;
    let service_tier = capture.service_tier();
    Ok(PreparedProxyRequest {
        request_id: selection.request_id,
        cache_affinity_ttl_minutes: selection.cache_affinity_ttl_minutes,
        candidates: selection.candidates,
        body,
        service_tier,
        is_stream,
        force_non_stream,
        provider_key_probe_slot_options: None,
        provider_headers,
    })
}

pub(super) fn attempt_payload(body: Value, candidate: &ProxyCandidate, force_non_stream: bool) -> Result<AttemptPayload, LlmProxyError> {
    let original_body = body.clone();
    let (body, source_format, target_format) = upstream_body(body, &original_body, candidate, force_non_stream)?;
    Ok(AttemptPayload {
        body,
        original_body,
        source_format,
        target_format,
    })
}

fn required_model(body: &Value) -> Result<&str, LlmProxyError> {
    body.get("model")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| LlmProxyError::InvalidRequest("request body must include a non-empty model".into()))
}

fn has_openai_responses_custom_tool_items(api_format: &str, body: &Value) -> bool {
    api_format == OPENAI_CLI_FORMAT && input_items(body).any(is_openai_responses_custom_tool_item)
}

fn input_items(body: &Value) -> impl Iterator<Item = &Value> {
    body.get("input").and_then(Value::as_array).into_iter().flatten()
}

fn is_openai_responses_custom_tool_item(item: &Value) -> bool {
    matches!(item.get("type").and_then(Value::as_str), Some("custom_tool_call" | "custom_tool_call_output"))
}

fn is_streaming(body: &Value) -> bool {
    body.get("stream").and_then(Value::as_bool).unwrap_or(false)
}

fn routing_api_format(api_format: &'static str, body: &Value) -> &'static str {
    if openai_request_explicitly_selects_image_generation(api_format, body) {
        return OPENAI_IMAGE_FORMAT;
    }
    api_format
}

fn required_capability_for_routing(api_format: &str) -> Option<&'static str> {
    matches!(api_format, OPENAI_IMAGE_FORMAT | OPENAI_IMAGE_EDIT_FORMAT).then_some(IMAGE_GENERATION_CAPABILITY)
}

fn openai_request_explicitly_selects_image_generation(api_format: &str, body: &Value) -> bool {
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

fn ensure_selection_format(selection: &CandidateSelection, api_format: &str, is_stream: bool) -> Result<(), LlmProxyError> {
    for candidate in &selection.candidates {
        if candidate.trace.client_api_format != api_format {
            return Err(LlmProxyError::InvalidRequest("fixed candidate client format mismatch".into()));
        }
        if candidate.trace.is_stream != is_stream {
            return Err(LlmProxyError::InvalidRequest("fixed candidate stream flag does not match request body".into()));
        }
    }
    Ok(())
}

fn upstream_body(
    body: Value,
    original_body: &Value,
    candidate: &ProxyCandidate,
    force_non_stream: bool,
) -> Result<(Value, ApiFormat, ApiFormat), LlmProxyError> {
    let mut body = body;
    let is_stream = body.get("stream").and_then(Value::as_bool).unwrap_or(false);
    if provider_is_openai_image(candidate) && client_is_openai_chat_or_responses(candidate) {
        body = openai_image_bridge_body(body, candidate)?;
        rewrite_upstream_body(&mut body, candidate, force_non_stream, ApiFormat::OpenAiImage)?;
        apply_provider_body_rules(&mut body, &candidate.body_rules, original_body)?;
        return Ok((body, client_source_format(candidate)?, ApiFormat::OpenAiImage));
    }
    let (source, target) = formats::conversion_formats(&candidate.trace.client_api_format, &candidate.trace.provider_api_format, is_stream)?;
    if candidate.trace.needs_conversion {
        body = FormatConversionRegistry
            .convert_request(&body, source, target)
            .map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
    }
    rewrite_upstream_body(&mut body, candidate, force_non_stream, target)?;
    apply_reasoning_effort(&mut body, candidate, target)?;
    apply_provider_body_rules(&mut body, &candidate.body_rules, original_body)?;
    Ok((body, source, target))
}

fn provider_is_openai_image(candidate: &ProxyCandidate) -> bool {
    formats::endpoint_metadata(&candidate.trace.provider_api_format, false)
        .map(|metadata| metadata.data_format == ApiFormat::OpenAiImage)
        .unwrap_or(false)
}

fn client_is_openai_chat_or_responses(candidate: &ProxyCandidate) -> bool {
    matches!(candidate.trace.client_api_format.as_str(), OPENAI_CHAT_FORMAT | OPENAI_CLI_FORMAT)
}

fn client_source_format(candidate: &ProxyCandidate) -> Result<ApiFormat, LlmProxyError> {
    formats::endpoint_metadata(&candidate.trace.client_api_format, candidate.trace.is_stream).map(|metadata| metadata.data_format)
}

fn openai_image_bridge_body(body: Value, candidate: &ProxyCandidate) -> Result<Value, LlmProxyError> {
    if candidate.trace.client_api_format == OPENAI_CLI_FORMAT {
        return Ok(body);
    }
    bridge_openai_chat_image_body(body)
}

fn bridge_openai_chat_image_body(body: Value) -> Result<Value, LlmProxyError> {
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
    if tools.iter().any(|tool| {
        tool.get("type")
            .and_then(Value::as_str)
            .is_some_and(|kind| kind.trim().eq_ignore_ascii_case("image_generation"))
    }) {
        return Value::Array(tools);
    }
    let mut output = tools;
    output.push(Value::Object(image_tool_choice()));
    Value::Array(output)
}

fn image_tool_choice() -> Map<String, Value> {
    Map::from_iter([("type".to_string(), Value::String("image_generation".to_string()))])
}

fn copy_optional_field(source: &Map<String, Value>, target: &mut Map<String, Value>, key: &str) {
    if let Some(value) = source.get(key) {
        target.insert(key.to_string(), value.clone());
    }
}

fn rewrite_upstream_body(body: &mut Value, candidate: &ProxyCandidate, force_non_stream: bool, target: ApiFormat) -> Result<(), LlmProxyError> {
    let object = body
        .as_object_mut()
        .ok_or_else(|| LlmProxyError::InvalidRequest("request body must be a JSON object".into()))?;
    let metadata = formats::endpoint_metadata(
        &candidate.trace.provider_api_format,
        object.get("stream").and_then(Value::as_bool).unwrap_or(false),
    )?;
    if metadata.model_in_body {
        object.insert("model".into(), Value::String(candidate.provider_model_name.clone()));
    } else {
        object.remove("model");
    }
    if !metadata.stream_in_body || force_non_stream || metadata.upstream_stream_policy == formats::UpstreamStreamPolicy::ForceNonStream {
        object.remove("stream");
    }
    ensure_stream_usage(object, metadata, target, force_non_stream)?;
    Ok(())
}

fn ensure_stream_usage(
    object: &mut Map<String, Value>,
    metadata: formats::EndpointMetadata,
    target: ApiFormat,
    force_non_stream: bool,
) -> Result<(), LlmProxyError> {
    if target != metadata.data_format || !metadata.include_usage_for_stream || force_non_stream || object.get("stream").and_then(Value::as_bool) != Some(true) {
        return Ok(());
    }
    let stream_options = object.entry("stream_options").or_insert_with(|| Value::Object(Map::new()));
    let options = stream_options
        .as_object_mut()
        .ok_or_else(|| LlmProxyError::InvalidRequest("request field stream_options must be a JSON object".into()))?;
    options.insert("include_usage".into(), Value::Bool(true));
    Ok(())
}

fn apply_reasoning_effort(body: &mut Value, candidate: &ProxyCandidate, target: ApiFormat) -> Result<(), LlmProxyError> {
    let Some(reasoning_effort) = candidate.reasoning_effort.as_deref() else {
        return Ok(());
    };
    let object = body
        .as_object_mut()
        .ok_or_else(|| LlmProxyError::InvalidRequest("request body must be a JSON object".into()))?;
    match target {
        ApiFormat::OpenAiChat => {
            object.insert("reasoning_effort".into(), Value::String(reasoning_effort.to_owned()));
            Ok(())
        }
        ApiFormat::OpenAiResponses | ApiFormat::OpenAiResponsesCompact => {
            reasoning_object(object)?.insert("effort".into(), Value::String(reasoning_effort.to_owned()));
            Ok(())
        }
        _ => Err(LlmProxyError::InvalidRequest(format!(
            "reasoning_effort override is not supported for provider format {}",
            candidate.trace.provider_api_format
        ))),
    }
}

fn reasoning_object(object: &mut Map<String, Value>) -> Result<&mut Map<String, Value>, LlmProxyError> {
    let value = object.entry("reasoning").or_insert_with(|| Value::Object(Map::new()));
    value
        .as_object_mut()
        .ok_or_else(|| LlmProxyError::InvalidRequest("request field reasoning must be a JSON object".into()))
}

#[cfg(test)]
#[path = "request_tests.rs"]
mod tests;

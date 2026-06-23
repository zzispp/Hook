use axum::http::HeaderMap;
use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::Value;
use types::api_token::ApiToken;

use crate::llm_proxy::{
    IMAGE_GENERATION_CAPABILITY, LlmProxyError, LlmProxyState, OPENAI_CLI_FORMAT, OPENAI_IMAGE_EDIT_FORMAT, OPENAI_IMAGE_FORMAT,
    audit::record_scheduled_candidates,
    billing::enforce_preflight_access,
    candidate::{CandidateRequest, CandidateSelection, ProxyCandidate, select_candidates},
    codex_chat_history::CodexChatHistoryStore,
    formats, rate_limit,
    rate_limit::ProviderKeyProbeSlotOptions,
};

use super::{
    body_rules::apply_provider_body_rules,
    capture::RequestCapture,
    request_codex_history,
    request_features::routing_request_features,
    request_image::{client_is_openai_chat_or_responses, client_source_format, openai_image_bridge_body, provider_is_openai_image},
    request_rewrite::{apply_reasoning_effort, rewrite_upstream_body},
    request_tools::{openai_request_explicitly_selects_image_generation, prune_unsupported_image_generation_tool},
};

#[derive(Clone)]
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

#[derive(Clone)]
pub(super) struct AttemptPayload {
    pub(super) body: Value,
    pub(super) original_body: Value,
    pub(super) source_format: ApiFormat,
    pub(super) target_format: ApiFormat,
}

pub(super) struct AttemptContext<'a> {
    pub(super) codex_chat_history: &'a CodexChatHistoryStore,
}

impl<'a> AttemptContext<'a> {
    pub(super) fn from_state(state: &'a LlmProxyState) -> Self {
        Self {
            codex_chat_history: state.codex_chat_history(),
        }
    }
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
            has_openai_responses_tool_outputs_without_previous_response_id: has_openai_responses_tool_outputs_without_previous_response_id(api_format, &body),
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

pub(super) async fn attempt_payload(
    context: AttemptContext<'_>,
    body: Value,
    candidate: &ProxyCandidate,
    force_non_stream: bool,
) -> Result<AttemptPayload, LlmProxyError> {
    let original_body = body.clone();
    let (body, source_format, target_format) = upstream_body(context, body, &original_body, candidate, force_non_stream).await?;
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
    api_format == OPENAI_CLI_FORMAT && input_contains_item(body, is_openai_responses_custom_tool_item)
}

fn has_openai_responses_tool_outputs_without_previous_response_id(api_format: &str, body: &Value) -> bool {
    api_format == OPENAI_CLI_FORMAT && !has_previous_response_id(body) && input_contains_item(body, is_openai_responses_tool_output_item)
}

fn is_openai_responses_custom_tool_item(item: &Value) -> bool {
    matches!(item.get("type").and_then(Value::as_str), Some("custom_tool_call" | "custom_tool_call_output"))
}

fn has_previous_response_id(body: &Value) -> bool {
    body.get("previous_response_id")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
}

fn input_contains_item(body: &Value, predicate: fn(&Value) -> bool) -> bool {
    match body.get("input") {
        Some(Value::Array(items)) => items.iter().any(predicate),
        Some(item @ Value::Object(_)) => predicate(item),
        _ => false,
    }
}

fn is_openai_responses_tool_output_item(item: &Value) -> bool {
    matches!(
        item.get("type").and_then(Value::as_str),
        Some("function_call_output" | "custom_tool_call_output")
    )
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

async fn upstream_body(
    context: AttemptContext<'_>,
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
    request_codex_history::enrich_responses_chat_request(context, &mut body, source, target).await?;
    if candidate.trace.needs_conversion {
        body = FormatConversionRegistry
            .convert_request(&body, source, target)
            .map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
    }
    rewrite_upstream_body(&mut body, candidate, force_non_stream, target)?;
    apply_reasoning_effort(&mut body, candidate, target)?;
    prune_unsupported_image_generation_tool(&mut body, candidate);
    apply_provider_body_rules(&mut body, &candidate.body_rules, original_body)?;
    Ok((body, source, target))
}

#[cfg(test)]
#[path = "request_tests.rs"]
mod tests;

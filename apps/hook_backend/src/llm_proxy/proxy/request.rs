use axum::http::HeaderMap;
use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::{Map, Value};
use types::api_token::ApiToken;

use crate::llm_proxy::{
    LlmProxyError, LlmProxyState,
    audit::record_scheduled_candidates,
    billing::enforce_preflight_access,
    candidate::{CandidateRequest, CandidateSelection, ProxyCandidate, select_candidates},
    formats, rate_limit,
    rate_limit::ProviderKeyProbeSlotOptions,
};

use super::{body_rules::apply_provider_body_rules, capture::RequestCapture};

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
    let selection = select_candidates(
        state,
        token,
        CandidateRequest {
            api_format,
            model_name,
            is_stream,
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

fn is_streaming(body: &Value) -> bool {
    body.get("stream").and_then(Value::as_bool).unwrap_or(false)
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

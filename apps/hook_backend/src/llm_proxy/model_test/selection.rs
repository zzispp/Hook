use super::TESTABLE_FORMATS;
use crate::llm_proxy::{
    LlmProxyError,
    cache::snapshot::{CachedEndpoint, CachedGlobalModel, CachedModelBinding, CachedProvider, CachedProviderKey, SchedulingSnapshot},
    formats,
};

#[derive(Clone)]
pub(super) struct FixedParts {
    pub(super) provider: CachedProvider,
    pub(super) global_model: CachedGlobalModel,
    pub(super) model: CachedModelBinding,
    pub(super) client_api_format: String,
    pub(super) endpoints: Vec<CachedEndpoint>,
    pub(super) keys: Vec<CachedProviderKey>,
    pub(super) force_non_stream: bool,
    pub(super) effective_stream: bool,
}

pub(super) fn fixed_parts(
    snapshot: &SchedulingSnapshot,
    provider_id: &str,
    model_id: &str,
    endpoint_id: &str,
    key_id: &str,
    requested_stream: bool,
) -> Result<FixedParts, LlmProxyError> {
    let provider = active_provider(snapshot, provider_id)?;
    let client_api_format = active_endpoint(&provider, endpoint_id, requested_stream)?.api_format;
    let force_non_stream = force_non_stream(&client_api_format, requested_stream)?;
    let effective_stream = requested_stream && !force_non_stream;
    ensure_test_format_supported(&client_api_format, effective_stream)?;
    let model = active_model(&provider, model_id)?;
    let global_model = active_global_model(snapshot, &model.global_model_id)?;
    let endpoints = eligible_endpoints(&provider, &client_api_format, effective_stream)?;
    let key = eligible_key(&provider, &model.global_model_id, &endpoints, key_id)?;
    let endpoints = endpoints_for_key(endpoints, &key);
    let keys = vec![key];
    Ok(FixedParts {
        provider,
        global_model,
        model,
        client_api_format,
        endpoints,
        keys,
        force_non_stream,
        effective_stream,
    })
}

fn active_provider(snapshot: &SchedulingSnapshot, provider_id: &str) -> Result<CachedProvider, LlmProxyError> {
    snapshot
        .providers
        .iter()
        .find(|provider| provider.id == provider_id && provider.is_active)
        .cloned()
        .ok_or_else(|| LlmProxyError::NotFound("provider not found or inactive".into()))
}

fn active_endpoint(provider: &CachedProvider, endpoint_id: &str, stream: bool) -> Result<CachedEndpoint, LlmProxyError> {
    if endpoint_id.trim().is_empty() {
        return Err(LlmProxyError::InvalidRequest("endpoint_id cannot be blank".into()));
    }
    let endpoint = provider
        .endpoints
        .iter()
        .find(|endpoint| endpoint.id == endpoint_id && endpoint.is_active)
        .cloned()
        .ok_or_else(|| LlmProxyError::InvalidRequest("selected endpoint is not active or does not exist".into()))?;
    let _ = formats::endpoint_metadata(&endpoint.api_format, stream)?;
    Ok(endpoint)
}

fn active_model(provider: &CachedProvider, model_id: &str) -> Result<CachedModelBinding, LlmProxyError> {
    let model = provider
        .models
        .iter()
        .find(|model| model.id == model_id && model.is_active)
        .cloned()
        .ok_or_else(|| LlmProxyError::InvalidRequest("selected provider model is not active or does not exist".into()))?;
    Ok(selected_provider_model(model))
}

fn active_global_model(snapshot: &SchedulingSnapshot, model_id: &str) -> Result<CachedGlobalModel, LlmProxyError> {
    snapshot
        .models
        .iter()
        .find(|model| model.id == model_id && model.is_active)
        .cloned()
        .ok_or_else(|| LlmProxyError::InvalidRequest("bound global model is not active or does not exist".into()))
}

fn eligible_endpoints(provider: &CachedProvider, client_api_format: &str, stream: bool) -> Result<Vec<CachedEndpoint>, LlmProxyError> {
    let (mut exact, converted): (Vec<_>, Vec<_>) = provider
        .endpoints
        .iter()
        .filter(|endpoint| endpoint_allowed(provider, endpoint, client_api_format, stream))
        .cloned()
        .partition(|endpoint| endpoint_exact(endpoint, client_api_format, stream));
    exact.extend(converted);
    if exact.is_empty() {
        return Err(LlmProxyError::InvalidRequest(
            "provider has no compatible active endpoint for selected test format".into(),
        ));
    }
    Ok(exact)
}

fn eligible_key(provider: &CachedProvider, model_id: &str, endpoints: &[CachedEndpoint], key_id: &str) -> Result<CachedProviderKey, LlmProxyError> {
    if key_id.trim().is_empty() {
        return Err(LlmProxyError::InvalidRequest("key_id cannot be blank".into()));
    }
    let current_minute = current_utc_minute();
    let key = provider
        .keys
        .iter()
        .find(|key| key.id == key_id)
        .cloned()
        .ok_or_else(|| LlmProxyError::InvalidRequest("selected API key does not exist for provider".into()))?;
    if endpoints
        .iter()
        .any(|endpoint| key_allowed_for_model_endpoint(&key, model_id, endpoint, current_minute))
    {
        return Ok(key);
    }
    if !key.is_active {
        return Err(LlmProxyError::InvalidRequest("selected API key is inactive".into()));
    }
    if !key_time_range_allowed(&key, current_minute) {
        return Err(LlmProxyError::InvalidRequest("selected API key is outside its active time range".into()));
    }
    if !key_allows_model(&key, model_id) {
        return Err(LlmProxyError::InvalidRequest("selected API key does not allow this model".into()));
    }
    if !endpoints
        .iter()
        .any(|endpoint| key.api_formats.iter().any(|format| format == &endpoint.api_format))
    {
        return Err(LlmProxyError::InvalidRequest(
            "selected API key does not support a compatible test endpoint format".into(),
        ));
    }
    Err(LlmProxyError::InvalidRequest("selected API key is not eligible for this model test".into()))
}

fn endpoints_for_key(endpoints: Vec<CachedEndpoint>, key: &CachedProviderKey) -> Vec<CachedEndpoint> {
    endpoints
        .into_iter()
        .filter(|endpoint| key.api_formats.iter().any(|format| format == &endpoint.api_format))
        .collect()
}

fn ensure_test_format_supported(api_format: &str, stream: bool) -> Result<(), LlmProxyError> {
    if TESTABLE_FORMATS.contains(&api_format) {
        let _ = formats::endpoint_metadata(api_format, stream)?;
        return Ok(());
    }
    Err(LlmProxyError::InvalidRequest(format!(
        "api_format does not support provider model test: {api_format}"
    )))
}

fn endpoint_allowed(provider: &CachedProvider, endpoint: &CachedEndpoint, client_api_format: &str, stream: bool) -> bool {
    endpoint.is_active && (endpoint_exact(endpoint, client_api_format, stream) || conversion_allowed(provider, endpoint, client_api_format, stream))
}

fn conversion_allowed(provider: &CachedProvider, endpoint: &CachedEndpoint, client_api_format: &str, stream: bool) -> bool {
    (provider.enable_format_conversion || endpoint_accepts_conversion(endpoint))
        && formats::formats_compatible(client_api_format, &endpoint.api_format, stream)
        && !endpoint_exact(endpoint, client_api_format, stream)
}

fn endpoint_exact(endpoint: &CachedEndpoint, client_api_format: &str, stream: bool) -> bool {
    formats::formats_exact(client_api_format, &endpoint.api_format, stream).unwrap_or(false)
}

fn endpoint_accepts_conversion(endpoint: &CachedEndpoint) -> bool {
    endpoint
        .format_acceptance_config
        .as_ref()
        .and_then(|value| value.get("enabled"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

fn key_allowed_for_model_endpoint(key: &CachedProviderKey, model_id: &str, endpoint: &CachedEndpoint, current_minute: u16) -> bool {
    key.is_active
        && key_time_range_allowed(key, current_minute)
        && key_allows_model(key, model_id)
        && key.api_formats.iter().any(|format| format == &endpoint.api_format)
}

fn key_allows_model(key: &CachedProviderKey, model_id: &str) -> bool {
    key.allowed_model_ids.is_empty() || key.allowed_model_ids.iter().any(|id| id == model_id)
}

fn key_time_range_allowed(key: &CachedProviderKey, current_minute: u16) -> bool {
    if !key.time_range_enabled {
        return true;
    }
    let (Some(start), Some(end)) = (key.time_range_start_minute, key.time_range_end_minute) else {
        return false;
    };
    types::provider::provider_key_time_range_contains(current_minute, start, end)
}

fn current_utc_minute() -> u16 {
    let time = time::OffsetDateTime::now_utc().time();
    types::provider::provider_key_minute_of_day(u16::from(time.hour()), u16::from(time.minute())).expect("UTC time must have a valid minute of day")
}

fn selected_provider_model(mut model: CachedModelBinding) -> CachedModelBinding {
    if let Some(mapping) = &model.provider_model_mapping {
        model.provider_model_name = mapping.name.clone();
    }
    model
}

fn force_non_stream(api_format: &str, requested_stream: bool) -> Result<bool, LlmProxyError> {
    if !requested_stream {
        return Ok(false);
    }
    let metadata = formats::endpoint_metadata(api_format, requested_stream)?;
    Ok(metadata.upstream_stream_policy == formats::UpstreamStreamPolicy::ForceNonStream)
}

#[cfg(test)]
#[path = "selection_tests.rs"]
mod tests;

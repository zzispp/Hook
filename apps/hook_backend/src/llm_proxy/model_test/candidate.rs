use provider::application::SecretCipher;
use rust_decimal::Decimal;

use super::{TEST_GROUP_CODE, selection::FixedParts};
use crate::llm_proxy::{
    LlmProxyError, LlmProxyState,
    cache::snapshot::{CachedEndpoint, CachedProviderKey},
    candidate::{CandidateEndpointOption, CandidateKeyOption, CandidateRoute, CandidateRouteOption, CandidateTrace, ProxyCandidate, url},
    formats,
};

pub(super) fn proxy_candidate(state: &LlmProxyState, parts: FixedParts, stream: bool) -> Result<ProxyCandidate, LlmProxyError> {
    let route = candidate_route(state, &parts, stream)?;
    let option = &route.options[0];
    let key = &option.key;
    let endpoint = &option.endpoint;
    Ok(ProxyCandidate {
        trace: candidate_trace(&parts, key, endpoint, stream),
        requested_model_name: parts.global_model.name.clone(),
        api_key: key.api_key.clone(),
        base_url: endpoint.base_url.clone(),
        custom_path: endpoint.custom_path.clone(),
        upstream_url: endpoint.upstream_url.clone(),
        provider_model_name: parts.effective_upstream_model_name.clone(),
        reasoning_effort: parts.effective_reasoning_effort.clone(),
        header_rules: endpoint.header_rules.clone(),
        body_rules: endpoint.body_rules.clone(),
        key_capabilities: key.capabilities.clone(),
        price_per_request: parts.global_model.default_price_per_request,
        tiered_pricing: parts.global_model.default_tiered_pricing,
        billing_multiplier: Decimal::ONE,
        max_retries: route_retry_floor(&route)?,
        request_timeout_seconds: parts.provider.request_timeout_seconds,
        stream_first_byte_timeout_seconds: parts.provider.stream_first_byte_timeout_seconds,
        stream_idle_timeout_seconds: parts.provider.stream_idle_timeout_seconds,
        cache_ttl_minutes: key.cache_ttl_minutes,
        key_rpm_limit: key.rpm_limit,
        is_cached: false,
        route,
    })
}

fn candidate_route(state: &LlmProxyState, parts: &FixedParts, stream: bool) -> Result<CandidateRoute, LlmProxyError> {
    let options = route_options(state, parts, stream)?;
    if options.is_empty() {
        return Err(LlmProxyError::NotFound("no provider key supports compatible test endpoint formats".into()));
    }
    Ok(CandidateRoute { options })
}

fn route_options(state: &LlmProxyState, parts: &FixedParts, stream: bool) -> Result<Vec<CandidateRouteOption>, LlmProxyError> {
    let mut output = Vec::new();
    for endpoint in &parts.endpoints {
        for key in parts.keys.iter().filter(|key| key_supports_endpoint(key, &endpoint.api_format)) {
            output.push(CandidateRouteOption {
                endpoint: endpoint_option(parts, endpoint, stream)?,
                key: key_option(state, key)?,
            });
        }
    }
    Ok(output)
}

fn endpoint_option(parts: &FixedParts, endpoint: &CachedEndpoint, stream: bool) -> Result<CandidateEndpointOption, LlmProxyError> {
    let needs_conversion = formats::needs_conversion(&parts.client_api_format, &endpoint.api_format, stream)?;
    Ok(CandidateEndpointOption {
        id: endpoint.id.clone(),
        name: endpoint.api_format.clone(),
        provider_api_format: endpoint.api_format.clone(),
        base_url: endpoint.base_url.clone(),
        custom_path: endpoint.custom_path.clone(),
        upstream_url: url::upstream_url_checked(endpoint, &parts.effective_upstream_model_name, stream)?,
        max_retries: endpoint.max_retries,
        header_rules: endpoint.header_rules.clone(),
        body_rules: endpoint.body_rules.clone(),
        needs_conversion,
    })
}

fn key_option(state: &LlmProxyState, key: &CachedProviderKey) -> Result<CandidateKeyOption, LlmProxyError> {
    Ok(CandidateKeyOption {
        id: key.id.clone(),
        name: key.name.clone(),
        key_preview: key.key_preview.clone(),
        api_key: state.cipher.decrypt_provider_key(&key.encrypted_api_key)?,
        capabilities: key.capabilities.clone(),
        cache_ttl_minutes: key.cache_ttl_minutes,
        rpm_limit: key.rpm_limit,
    })
}

fn candidate_trace(parts: &FixedParts, key: &CandidateKeyOption, endpoint: &CandidateEndpointOption, stream: bool) -> CandidateTrace {
    CandidateTrace {
        token_id: None,
        user_id_snapshot: None,
        username_snapshot: None,
        token_name_snapshot: None,
        token_prefix_snapshot: None,
        group_code: Some(TEST_GROUP_CODE.into()),
        global_model_id: parts.global_model.id.clone(),
        provider_model_id: parts.model.id.clone(),
        model_name_snapshot: parts.global_model.name.clone(),
        provider_id: parts.provider.id.clone(),
        provider_name_snapshot: parts.provider.name.clone(),
        endpoint_id: endpoint.id.clone(),
        endpoint_name_snapshot: endpoint.provider_api_format.clone(),
        key_id: key.id.clone(),
        key_name_snapshot: key.name.clone(),
        key_preview_snapshot: key.key_preview.clone(),
        client_api_format: parts.client_api_format.clone(),
        provider_api_format: endpoint.provider_api_format.clone(),
        needs_conversion: endpoint.needs_conversion,
        is_stream: stream,
        is_cached: false,
        routing_context_key: format!(
            "group={TEST_GROUP_CODE}|model={}|format={}|stream={}|size=unknown|cap=none",
            parts.global_model.id, parts.client_api_format, stream
        ),
        route_config_fingerprint: "model-test-route-fingerprint".into(),
        price_config_fingerprint: "model-test-price-fingerprint".into(),
        candidate_index: 0,
    }
}

fn key_supports_endpoint(key: &CachedProviderKey, api_format: &str) -> bool {
    key.api_formats.iter().any(|format| format == api_format)
}

fn route_retry_floor(route: &CandidateRoute) -> Result<i32, LlmProxyError> {
    i32::try_from(route.options.len().saturating_sub(1))
        .map_err(|_| LlmProxyError::Infrastructure("candidate route option count exceeds retry index range".into()))
}

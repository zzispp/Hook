use provider::application::SecretCipher;
use types::api_token::ApiToken;

use super::{CandidateParts, DEFAULT_MAX_RETRIES, GlobalModelRef};
use crate::llm_proxy::{
    LlmProxyError, LlmProxyState,
    cache::snapshot::CachedBillingGroup,
    candidate::{CandidateEndpointOption, CandidateKeyOption, CandidateRequest, CandidateRoute, CandidateTrace, ProxyCandidate, url},
};

pub(super) async fn proxy_candidates(
    state: &LlmProxyState,
    token: &ApiToken,
    request: CandidateRequest<'_>,
    global_model: &GlobalModelRef,
    group: &CachedBillingGroup,
    parts: &[CandidateParts],
) -> Result<Vec<ProxyCandidate>, LlmProxyError> {
    let mut candidates = Vec::with_capacity(parts.len());
    for (index, part) in parts.iter().enumerate() {
        candidates.push(proxy_candidate(state, token, request, global_model, group, part, index as i32).await?);
    }
    Ok(candidates)
}

async fn proxy_candidate(
    state: &LlmProxyState,
    token: &ApiToken,
    request: CandidateRequest<'_>,
    global_model: &GlobalModelRef,
    group: &CachedBillingGroup,
    parts: &CandidateParts,
    index: i32,
) -> Result<ProxyCandidate, LlmProxyError> {
    let route = candidate_route(state, request, parts)?;
    let endpoint = &route.endpoints[0];
    let key = &route.keys[0];
    Ok(ProxyCandidate {
        trace: candidate_trace(token, request, parts, index),
        api_key: key.api_key.clone(),
        base_url: endpoint.base_url.clone(),
        custom_path: endpoint.custom_path.clone(),
        upstream_url: endpoint.upstream_url.clone(),
        provider_model_name: parts.model.provider_model_name.clone(),
        header_rules: endpoint.header_rules.clone(),
        price_per_request: parts.model.price_per_request.or(global_model.default_price_per_request),
        tiered_pricing: parts
            .model
            .tiered_pricing
            .clone()
            .unwrap_or_else(|| global_model.default_tiered_pricing.clone()),
        billing_multiplier: group.billing_multiplier,
        max_retries: max_retries(parts, &route)?,
        request_timeout_seconds: parts.provider.request_timeout_seconds,
        stream_first_byte_timeout_seconds: parts.provider.stream_first_byte_timeout_seconds,
        cache_ttl_minutes: key.cache_ttl_minutes,
        route,
    })
}

fn candidate_trace(token: &ApiToken, request: CandidateRequest<'_>, parts: &CandidateParts, index: i32) -> CandidateTrace {
    let endpoint = &parts.endpoints[0];
    let key = &parts.keys[0];
    CandidateTrace {
        token_id: Some(token.id.clone()),
        group_code: Some(token.group_code.clone()),
        global_model_id: parts.model.global_model_id.clone(),
        provider_id: parts.provider.id.clone(),
        endpoint_id: endpoint.id.clone(),
        key_id: key.id.clone(),
        client_api_format: request.api_format.to_owned(),
        provider_api_format: endpoint.api_format.clone(),
        needs_conversion: endpoint.api_format != request.api_format,
        is_stream: request.is_stream,
        candidate_index: index,
    }
}

fn max_retries(parts: &CandidateParts, route: &CandidateRoute) -> Result<i32, LlmProxyError> {
    let configured = parts.endpoints[0]
        .max_retries
        .or(parts.provider.max_retries)
        .unwrap_or(DEFAULT_MAX_RETRIES)
        .max(0);
    Ok(configured.max(route_retry_floor(route)?))
}

fn route_retry_floor(route: &CandidateRoute) -> Result<i32, LlmProxyError> {
    let option_count = route
        .endpoints
        .len()
        .checked_mul(route.keys.len())
        .ok_or_else(|| LlmProxyError::Infrastructure("candidate route option count overflowed".into()))?;
    i32::try_from(option_count.saturating_sub(1)).map_err(|_| LlmProxyError::Infrastructure("candidate route option count exceeds retry index range".into()))
}

fn candidate_route(state: &LlmProxyState, request: CandidateRequest<'_>, parts: &CandidateParts) -> Result<CandidateRoute, LlmProxyError> {
    Ok(CandidateRoute {
        endpoints: endpoint_options(request, parts),
        keys: key_options(state, parts)?,
    })
}

fn endpoint_options(request: CandidateRequest<'_>, parts: &CandidateParts) -> Vec<CandidateEndpointOption> {
    parts
        .endpoints
        .iter()
        .map(|endpoint| CandidateEndpointOption {
            id: endpoint.id.clone(),
            provider_api_format: endpoint.api_format.clone(),
            base_url: endpoint.base_url.clone(),
            custom_path: endpoint.custom_path.clone(),
            upstream_url: url::upstream_url(endpoint, &parts.model.provider_model_name, request.is_stream),
            header_rules: endpoint.header_rules.clone(),
            needs_conversion: endpoint.api_format != request.api_format,
        })
        .collect()
}

fn key_options(state: &LlmProxyState, parts: &CandidateParts) -> Result<Vec<CandidateKeyOption>, LlmProxyError> {
    parts
        .keys
        .iter()
        .map(|key| {
            Ok(CandidateKeyOption {
                id: key.id.clone(),
                api_key: state.cipher.decrypt_provider_key(&key.encrypted_api_key)?,
                cache_ttl_minutes: key.cache_ttl_minutes,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_retry_floor_covers_each_endpoint_key_option_once() {
        let route = CandidateRoute {
            endpoints: vec![endpoint("endpoint-a"), endpoint("endpoint-b")],
            keys: vec![key("key-a"), key("key-b"), key("key-c")],
        };

        assert_eq!(route_retry_floor(&route).unwrap(), 5);
    }

    fn endpoint(id: &str) -> CandidateEndpointOption {
        CandidateEndpointOption {
            id: id.into(),
            provider_api_format: "openai_chat".into(),
            base_url: "https://example.com".into(),
            custom_path: None,
            upstream_url: "https://example.com/v1/chat/completions".into(),
            header_rules: None,
            needs_conversion: false,
        }
    }

    fn key(id: &str) -> CandidateKeyOption {
        CandidateKeyOption {
            id: id.into(),
            api_key: "secret".into(),
            cache_ttl_minutes: 5,
        }
    }
}

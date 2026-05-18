use provider::application::SecretCipher;
use types::api_token::ApiToken;

use super::{CandidateParts, DEFAULT_MAX_RETRIES, GlobalModelRef};
use crate::llm_proxy::{
    LlmProxyError, LlmProxyState,
    cache::snapshot::{CachedBillingGroup, CachedEndpoint, CachedProviderKey, CachedUserAccess},
    candidate::{CandidateEndpointOption, CandidateKeyOption, CandidateRequest, CandidateRoute, CandidateRouteOption, CandidateTrace, ProxyCandidate, url},
    formats,
};

pub(super) struct ProxyCandidateBuildInput<'a> {
    pub(super) state: &'a LlmProxyState,
    pub(super) token: &'a ApiToken,
    pub(super) request: CandidateRequest<'a>,
    pub(super) global_model: &'a GlobalModelRef,
    pub(super) group: &'a CachedBillingGroup,
    pub(super) token_user: Option<&'a CachedUserAccess>,
    pub(super) parts: &'a [CandidateParts],
}

pub(super) async fn proxy_candidates(input: ProxyCandidateBuildInput<'_>) -> Result<Vec<ProxyCandidate>, LlmProxyError> {
    let mut candidates = Vec::with_capacity(input.parts.len());
    for (index, part) in input.parts.iter().enumerate() {
        candidates.push(proxy_candidate(&input, part, index as i32).await?);
    }
    Ok(candidates)
}

async fn proxy_candidate(input: &ProxyCandidateBuildInput<'_>, parts: &CandidateParts, index: i32) -> Result<ProxyCandidate, LlmProxyError> {
    let route = candidate_route(input.state, input.request, parts)?;
    let endpoint = &route.options[0].endpoint;
    let key = &route.options[0].key;
    Ok(ProxyCandidate {
        trace: candidate_trace(input.token, input.request, input.global_model, input.token_user, parts, endpoint, key, index),
        requested_model_name: input.request.model_name.to_owned(),
        api_key: key.api_key.clone(),
        base_url: endpoint.base_url.clone(),
        custom_path: endpoint.custom_path.clone(),
        upstream_url: endpoint.upstream_url.clone(),
        provider_model_name: parts.model.provider_model_name.clone(),
        reasoning_effort: parts.model.provider_model_mapping.as_ref().and_then(|mapping| mapping.reasoning_effort.clone()),
        header_rules: endpoint.header_rules.clone(),
        body_rules: endpoint.body_rules.clone(),
        price_per_request: parts.model.price_per_request.or(input.global_model.default_price_per_request),
        tiered_pricing: parts
            .model
            .tiered_pricing
            .clone()
            .unwrap_or_else(|| input.global_model.default_tiered_pricing.clone()),
        billing_multiplier: input.group.billing_multiplier,
        max_retries: max_retries(parts, &route)?,
        request_timeout_seconds: parts.provider.request_timeout_seconds,
        stream_first_byte_timeout_seconds: parts.provider.stream_first_byte_timeout_seconds,
        cache_ttl_minutes: key.cache_ttl_minutes,
        key_rpm_limit: key.rpm_limit,
        route,
    })
}

fn candidate_trace(
    token: &ApiToken,
    request: CandidateRequest<'_>,
    global_model: &GlobalModelRef,
    token_user: Option<&CachedUserAccess>,
    parts: &CandidateParts,
    endpoint: &CandidateEndpointOption,
    key: &CandidateKeyOption,
    index: i32,
) -> CandidateTrace {
    CandidateTrace {
        token_id: Some(token.id.clone()),
        user_id_snapshot: token.user_id.clone(),
        username_snapshot: token_user.map(|user| user.username.clone()),
        token_name_snapshot: Some(token.name.clone()),
        token_prefix_snapshot: Some(token.token_prefix.clone()),
        group_code: Some(token.group_code.clone()),
        global_model_id: parts.model.global_model_id.clone(),
        provider_model_id: parts.model.id.clone(),
        model_name_snapshot: global_model.name.clone(),
        provider_id: parts.provider.id.clone(),
        provider_name_snapshot: parts.provider.name.clone(),
        endpoint_id: endpoint.id.clone(),
        endpoint_name_snapshot: endpoint.name.clone(),
        key_id: key.id.clone(),
        key_name_snapshot: key.name.clone(),
        key_preview_snapshot: key.key_preview.clone(),
        client_api_format: request.api_format.to_owned(),
        provider_api_format: endpoint.provider_api_format.clone(),
        needs_conversion: endpoint.needs_conversion,
        is_stream: request.is_stream,
        candidate_index: index,
    }
}

fn max_retries(parts: &CandidateParts, route: &CandidateRoute) -> Result<i32, LlmProxyError> {
    let configured = route.options[0]
        .endpoint
        .max_retries
        .or(parts.provider.max_retries)
        .unwrap_or(DEFAULT_MAX_RETRIES)
        .max(0);
    Ok(configured.max(route_retry_floor(route)?))
}

fn route_retry_floor(route: &CandidateRoute) -> Result<i32, LlmProxyError> {
    let option_count = route.options.len();
    i32::try_from(option_count.saturating_sub(1)).map_err(|_| LlmProxyError::Infrastructure("candidate route option count exceeds retry index range".into()))
}

fn candidate_route(state: &LlmProxyState, request: CandidateRequest<'_>, parts: &CandidateParts) -> Result<CandidateRoute, LlmProxyError> {
    Ok(CandidateRoute {
        options: route_options(state, request, parts)?,
    })
}

fn route_options(state: &LlmProxyState, request: CandidateRequest<'_>, parts: &CandidateParts) -> Result<Vec<CandidateRouteOption>, LlmProxyError> {
    let mut output = Vec::new();
    for input in route_option_inputs(&parts.endpoints, &parts.keys, &parts.model.global_model_id) {
        output.push(CandidateRouteOption {
            endpoint: endpoint_option(request, parts, input.endpoint)?,
            key: key_option(state, input.key)?,
        });
    }
    if output.is_empty() {
        return Err(LlmProxyError::NotFound("no provider key supports selected endpoint formats".into()));
    }
    Ok(output)
}

struct RouteOptionInput<'a> {
    endpoint: &'a CachedEndpoint,
    key: &'a CachedProviderKey,
}

fn route_option_inputs<'a>(endpoints: &'a [CachedEndpoint], keys: &'a [CachedProviderKey], model_id: &str) -> Vec<RouteOptionInput<'a>> {
    let mut output = Vec::new();
    for endpoint in endpoints {
        for key in keys.iter().filter(|key| key_supports_model_endpoint(key, model_id, &endpoint.api_format)) {
            output.push(RouteOptionInput { endpoint, key });
        }
    }
    output
}

fn endpoint_option(request: CandidateRequest<'_>, parts: &CandidateParts, endpoint: &CachedEndpoint) -> Result<CandidateEndpointOption, LlmProxyError> {
    let needs_conversion = formats::needs_conversion(request.api_format, &endpoint.api_format, request.is_stream)?;
    Ok(CandidateEndpointOption {
        id: endpoint.id.clone(),
        name: endpoint.api_format.clone(),
        provider_api_format: endpoint.api_format.clone(),
        base_url: endpoint.base_url.clone(),
        custom_path: endpoint.custom_path.clone(),
        upstream_url: url::upstream_url_checked(endpoint, &parts.model.provider_model_name, request.is_stream)?,
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
        cache_ttl_minutes: key.cache_ttl_minutes,
        rpm_limit: key.rpm_limit,
    })
}

fn key_supports_endpoint(key: &CachedProviderKey, api_format: &str) -> bool {
    key.api_formats.iter().any(|format| format == api_format)
}

fn key_supports_model_endpoint(key: &CachedProviderKey, model_id: &str, api_format: &str) -> bool {
    key_allows_model(key, model_id) && key_supports_endpoint(key, api_format)
}

fn key_allows_model(key: &CachedProviderKey, model_id: &str) -> bool {
    key.allowed_model_ids.is_empty() || key.allowed_model_ids.iter().any(|id| id == model_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_retry_floor_covers_each_endpoint_key_option_once() {
        let route = CandidateRoute {
            options: vec![
                option("endpoint-a", "key-a"),
                option("endpoint-a", "key-b"),
                option("endpoint-a", "key-c"),
                option("endpoint-b", "key-a"),
                option("endpoint-b", "key-b"),
                option("endpoint-b", "key-c"),
            ],
        };

        assert_eq!(route_retry_floor(&route).unwrap(), 5);
    }

    #[test]
    fn route_option_inputs_only_pairs_keys_with_supported_endpoint_formats() {
        let endpoints = vec![
            cached_endpoint("endpoint-openai", "openai_chat"),
            cached_endpoint("endpoint-gemini", "gemini_chat"),
        ];
        let keys = vec![
            cached_key("key-openai", vec!["openai_chat"], Vec::new()),
            cached_key("key-gemini", vec!["gemini_chat"], Vec::new()),
            cached_key("key-empty", Vec::new(), Vec::new()),
        ];

        let inputs = route_option_inputs(&endpoints, &keys, "model-a");

        let pairs = inputs
            .iter()
            .map(|input| (input.endpoint.id.as_str(), input.key.id.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(pairs, vec![("endpoint-openai", "key-openai"), ("endpoint-gemini", "key-gemini")]);
    }

    #[test]
    fn route_option_inputs_treats_empty_allowed_models_as_unrestricted() {
        let endpoints = vec![cached_endpoint("endpoint-openai", "openai_chat")];
        let keys = vec![cached_key("key-openai", vec!["openai_chat"], Vec::new())];

        let inputs = route_option_inputs(&endpoints, &keys, "model-a");

        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].key.id, "key-openai");
    }

    #[test]
    fn route_option_inputs_filters_keys_by_allowed_models() {
        let endpoints = vec![cached_endpoint("endpoint-openai", "openai_chat")];
        let keys = vec![
            cached_key("key-model-a", vec!["openai_chat"], vec!["model-a"]),
            cached_key("key-model-b", vec!["openai_chat"], vec!["model-b"]),
        ];

        let inputs = route_option_inputs(&endpoints, &keys, "model-a");

        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].key.id, "key-model-a");
    }

    fn option(endpoint_id: &str, key_id: &str) -> CandidateRouteOption {
        CandidateRouteOption {
            endpoint: endpoint(endpoint_id),
            key: key(key_id),
        }
    }

    fn endpoint(id: &str) -> CandidateEndpointOption {
        CandidateEndpointOption {
            id: id.into(),
            name: "openai_chat".into(),
            provider_api_format: "openai_chat".into(),
            base_url: "https://example.com".into(),
            custom_path: None,
            upstream_url: "https://example.com/v1/chat/completions".into(),
            max_retries: None,
            header_rules: None,
            body_rules: None,
            needs_conversion: false,
        }
    }

    fn key(id: &str) -> CandidateKeyOption {
        CandidateKeyOption {
            id: id.into(),
            name: format!("{id}-name"),
            key_preview: "***cret".into(),
            api_key: "secret".into(),
            cache_ttl_minutes: 5,
            rpm_limit: None,
        }
    }

    fn cached_endpoint(id: &str, api_format: &str) -> CachedEndpoint {
        CachedEndpoint {
            id: id.into(),
            provider_id: "provider-a".into(),
            api_format: api_format.into(),
            base_url: "https://example.com".into(),
            custom_path: None,
            max_retries: None,
            is_active: true,
            format_acceptance_config: None,
            header_rules: None,
            body_rules: None,
        }
    }

    fn cached_key(id: &str, api_formats: Vec<&str>, allowed_model_ids: Vec<&str>) -> CachedProviderKey {
        CachedProviderKey {
            id: id.into(),
            provider_id: "provider-a".into(),
            name: format!("{id}-name"),
            api_formats: api_formats.into_iter().map(str::to_owned).collect(),
            allowed_model_ids: allowed_model_ids.into_iter().map(str::to_owned).collect(),
            key_preview: format!("{id}-name"),
            encrypted_api_key: "encrypted".into(),
            internal_priority: 10,
            rpm_limit: None,
            cache_ttl_minutes: 5,
            time_range_enabled: false,
            time_range_start_minute: None,
            time_range_end_minute: None,
            is_active: true,
        }
    }
}

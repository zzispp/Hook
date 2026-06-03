use provider::application::SecretCipher;

use super::CandidateParts;
use crate::llm_proxy::{
    LlmProxyError, LlmProxyState,
    cache::snapshot::{CachedEndpoint, CachedProviderKey},
    candidate::{CandidateEndpointOption, CandidateKeyOption, CandidateRequest, CandidateRoute, CandidateRouteOption, url},
    formats,
};

pub(super) fn candidate_route(state: &LlmProxyState, request: CandidateRequest<'_>, parts: &CandidateParts) -> Result<CandidateRoute, LlmProxyError> {
    Ok(CandidateRoute {
        options: route_options(state, request, parts)?,
    })
}

pub(super) fn route_retry_floor(route: &CandidateRoute) -> Result<i32, LlmProxyError> {
    let option_count = route.options.len();
    i32::try_from(option_count.saturating_sub(1)).map_err(|_| LlmProxyError::Infrastructure("candidate route option count exceeds retry index range".into()))
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
    let needs_conversion = formats::needs_conversion(&parts.routing_api_format, &endpoint.api_format, request.is_stream)?;
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

fn key_supports_model_endpoint(key: &CachedProviderKey, model_id: &str, api_format: &str) -> bool {
    key_allows_model(key, model_id) && key.api_formats.iter().any(|format| format == api_format)
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
            cached_endpoint("endpoint-openai", "openai:chat"),
            cached_endpoint("endpoint-gemini", "gemini:chat"),
        ];
        let keys = vec![
            cached_key("key-openai", vec!["openai:chat"], Vec::new()),
            cached_key("key-gemini", vec!["gemini:chat"], Vec::new()),
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
    fn route_option_inputs_filters_keys_by_allowed_models() {
        let endpoints = vec![cached_endpoint("endpoint-openai", "openai:chat")];
        let keys = vec![
            cached_key("key-model-a", vec!["openai:chat"], vec!["model-a"]),
            cached_key("key-model-b", vec!["openai:chat"], vec!["model-b"]),
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
            name: "openai:chat".into(),
            provider_api_format: "openai:chat".into(),
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
            global_priority_by_format: std::collections::BTreeMap::from([("openai:chat".to_owned(), 10)]),
            rpm_limit: None,
            cache_ttl_minutes: 5,
            time_range_enabled: false,
            time_range_start_minute: None,
            time_range_end_minute: None,
            is_active: true,
        }
    }
}

use types::api_token::ApiToken;
use types::provider::ProviderModelCost;

use super::{CandidateParts, DEFAULT_MAX_RETRIES, GlobalModelRef, dynamic_cost::model_cost_config, route};
use crate::llm_proxy::{
    LlmProxyError, LlmProxyState,
    cache::snapshot::{CachedBillingGroup, CachedUserAccess},
    candidate::{CandidateEndpointOption, CandidateKeyOption, CandidateRequest, CandidateRoute, CandidateTrace, ProxyCandidate},
    routing::{PriceFingerprintInput, RouteFingerprintInput, price_config_fingerprint, route_config_fingerprint, routing_context_key},
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
    let route = route::candidate_route(input.state, &input.request, parts)?;
    let route = effective_route(route, parts.is_cached);
    let endpoint = &route.options[0].endpoint;
    let key = &route.options[0].key;
    let configured_cost = model_cost_config(input.state, &key.id, &parts.model.id).await?;
    Ok(ProxyCandidate {
        trace: candidate_trace(CandidateTraceInput {
            token: input.token,
            request: input.request.clone(),
            global_model: input.global_model,
            group_code: &input.group.code,
            billing_multiplier: input.group.billing_multiplier,
            token_user: input.token_user,
            parts,
            endpoint,
            key,
            configured_cost: configured_cost.as_ref(),
            index,
        }),
        requested_model_name: input.request.model_name.to_owned(),
        api_key: key.api_key.clone(),
        base_url: endpoint.base_url.clone(),
        custom_path: endpoint.custom_path.clone(),
        upstream_url: endpoint.upstream_url.clone(),
        provider_model_name: parts.effective_upstream_model_name.clone(),
        reasoning_effort: parts.effective_reasoning_effort.clone(),
        header_rules: endpoint.header_rules.clone(),
        body_rules: endpoint.body_rules.clone(),
        key_supports_image_generation: key.supports_image_generation,
        price_per_request: input.global_model.default_price_per_request,
        tiered_pricing: input.global_model.default_tiered_pricing.clone(),
        billing_multiplier: input.group.billing_multiplier,
        max_retries: max_retries(parts, &route)?,
        request_timeout_seconds: parts.provider.request_timeout_seconds,
        stream_first_byte_timeout_seconds: parts.provider.stream_first_byte_timeout_seconds,
        stream_idle_timeout_seconds: parts.provider.stream_idle_timeout_seconds,
        cache_ttl_minutes: key.cache_ttl_minutes,
        key_rpm_limit: key.rpm_limit,
        is_cached: parts.is_cached,
        route,
    })
}

fn effective_route(mut route: CandidateRoute, is_cached: bool) -> CandidateRoute {
    if !is_cached {
        return route;
    }
    route.options.truncate(1);
    route
}

struct CandidateTraceInput<'a> {
    token: &'a ApiToken,
    request: CandidateRequest<'a>,
    global_model: &'a GlobalModelRef,
    group_code: &'a str,
    billing_multiplier: rust_decimal::Decimal,
    token_user: Option<&'a CachedUserAccess>,
    parts: &'a CandidateParts,
    endpoint: &'a CandidateEndpointOption,
    key: &'a CandidateKeyOption,
    configured_cost: Option<&'a ProviderModelCost>,
    index: i32,
}

fn candidate_trace(input: CandidateTraceInput<'_>) -> CandidateTrace {
    CandidateTrace {
        token_id: Some(input.token.id.clone()),
        user_id_snapshot: input.token.user_id.clone(),
        username_snapshot: input.token_user.map(|user| user.username.clone()),
        token_name_snapshot: Some(input.token.name.clone()),
        token_prefix_snapshot: Some(input.token.token_prefix.clone()),
        group_code: Some(input.token.group_code.clone()),
        global_model_id: input.parts.model.global_model_id.clone(),
        provider_model_id: input.parts.model.id.clone(),
        model_name_snapshot: input.global_model.name.clone(),
        provider_id: input.parts.provider.id.clone(),
        provider_name_snapshot: input.parts.provider.name.clone(),
        endpoint_id: input.endpoint.id.clone(),
        endpoint_name_snapshot: input.endpoint.name.clone(),
        key_id: input.key.id.clone(),
        key_name_snapshot: input.key.name.clone(),
        key_preview_snapshot: input.key.key_preview.clone(),
        client_api_format: input.parts.client_api_format.clone(),
        provider_api_format: input.endpoint.provider_api_format.clone(),
        needs_conversion: input.endpoint.needs_conversion,
        is_stream: input.request.is_stream,
        is_cached: input.parts.is_cached,
        routing_context_key: routing_context_key(input.group_code, &input.global_model.id, &input.request.features),
        route_config_fingerprint: route_config_fingerprint(RouteFingerprintInput {
            provider_id: &input.parts.provider.id,
            key_id: &input.key.id,
            endpoint_id: &input.endpoint.id,
            global_model_id: &input.parts.model.global_model_id,
            provider_model_id: &input.parts.model.id,
            effective_upstream_model_name: &input.parts.effective_upstream_model_name,
            effective_reasoning_effort: input.parts.effective_reasoning_effort.as_deref(),
            client_api_format: &input.parts.client_api_format,
            provider_api_format: &input.endpoint.provider_api_format,
            is_stream: input.request.is_stream,
            needs_conversion: input.endpoint.needs_conversion,
        }),
        price_config_fingerprint: price_config_fingerprint(PriceFingerprintInput {
            configured_cost: input.configured_cost,
            price_per_request: input.global_model.default_price_per_request,
            tiered_pricing: &input.global_model.default_tiered_pricing,
            billing_multiplier: input.billing_multiplier,
        }),
        candidate_index: input.index,
    }
}

fn max_retries(parts: &CandidateParts, route: &CandidateRoute) -> Result<i32, LlmProxyError> {
    let configured = route.options[0]
        .endpoint
        .max_retries
        .or(parts.provider.max_retries)
        .unwrap_or(DEFAULT_MAX_RETRIES)
        .max(0);
    Ok(configured.max(route::route_retry_floor(route)?))
}

#[path = "dynamic_routing/options.rs"]
mod options;

use types::provider::{ProviderPriorityMode, RouteIdentity, RouteScoreExplanation, RoutingMetricWindow, RoutingProfile, RoutingProfileId};

use super::{
    CandidateParts, GlobalModelRef,
    dynamic_cost::{estimated_cost_from_config, model_cost_config},
    routing_metrics::{ContextRouteStateCatalog, MetricCatalog, ResolvedMetric, RouteFingerprints, RouteStateCatalog},
};
use crate::llm_proxy::{
    LlmProxyError, LlmProxyState,
    cache::snapshot::{CachedBillingGroup, CachedEndpoint, CachedProviderKey},
    candidate::CandidateRequest,
    formats,
    rate_limit::provider_key_rate_limit_snapshot,
    routing::{
        PriceFingerprintInput, RouteFingerprintInput, RoutingScoreCandidate, price_config_fingerprint, profile_by_id, route_config_fingerprint,
        routing_context_key, score_routes,
    },
};

const LIVE_WINDOWS: [RoutingMetricWindow; 4] = [
    RoutingMetricWindow::FiveMinutes,
    RoutingMetricWindow::OneHour,
    RoutingMetricWindow::OneDay,
    RoutingMetricWindow::SevenDays,
];

pub(super) struct DynamicRoutingInput<'a> {
    pub(super) state: &'a LlmProxyState,
    pub(super) parts: Vec<CandidateParts>,
    pub(super) group: &'a CachedBillingGroup,
    pub(super) request: CandidateRequest<'a>,
    pub(super) global_model: &'a GlobalModelRef,
    pub(super) profile_id: RoutingProfileId,
    pub(super) priority_mode: ProviderPriorityMode,
    pub(super) allow_empty: bool,
}

pub(super) struct DynamicRoutingOutput {
    pub(super) parts: Vec<CandidateParts>,
    pub(super) profile: RoutingProfile,
    pub(super) explanations: Vec<RouteScoreExplanation>,
}

pub(super) async fn rank_candidate_parts(input: DynamicRoutingInput<'_>) -> Result<DynamicRoutingOutput, LlmProxyError> {
    let profile = profile_by_id(input.state, input.profile_id).await?.profile;
    rank(input, profile, RoutingMetricWindow::default(), true).await
}

pub(super) async fn rank_candidate_parts_with_profile(
    input: DynamicRoutingInput<'_>,
    profile: RoutingProfile,
    window: RoutingMetricWindow,
) -> Result<DynamicRoutingOutput, LlmProxyError> {
    rank(input, profile, window, false).await
}

async fn rank(
    input: DynamicRoutingInput<'_>,
    profile: RoutingProfile,
    window: RoutingMetricWindow,
    live_stable: bool,
) -> Result<DynamicRoutingOutput, LlmProxyError> {
    let snapshot = input.state.routing_metrics_snapshot().await;
    let catalog = MetricCatalog::from_snapshot(&snapshot, requested_windows(window, live_stable));
    let route_states = RouteStateCatalog::from_snapshot(&snapshot);
    let context_states = ContextRouteStateCatalog::from_snapshot(&snapshot);
    let context_key = routing_context_key(&input.group.code, &input.global_model.id, &input.request.features);
    let scored = score_candidates(ScoreCandidatesContext {
        input: &input,
        catalog: &catalog,
        route_states: &route_states,
        context_states: &context_states,
        context_key: &context_key,
        profile: &profile,
        requested_window: window,
    })
    .await?;
    let explanations = scored.scores.iter().map(|item| item.explanation.clone()).collect::<Vec<_>>();
    let parts = options::ordered_parts(input.parts, &scored.targets, &scored.scores)?;
    if parts.is_empty() && !input.allow_empty {
        return Err(LlmProxyError::Upstream("all provider candidates are excluded by dynamic routing".into()));
    }
    Ok(DynamicRoutingOutput { parts, profile, explanations })
}

struct ScoreCandidatesOutput {
    targets: Vec<options::ScoreTarget>,
    scores: Vec<crate::llm_proxy::routing::ScoredRoute>,
}

struct ScoreCandidatesContext<'a, 'request> {
    input: &'a DynamicRoutingInput<'request>,
    catalog: &'a MetricCatalog,
    route_states: &'a RouteStateCatalog,
    context_states: &'a ContextRouteStateCatalog,
    context_key: &'a str,
    profile: &'a RoutingProfile,
    requested_window: RoutingMetricWindow,
}

async fn score_candidates(context: ScoreCandidatesContext<'_, '_>) -> Result<ScoreCandidatesOutput, LlmProxyError> {
    let targets = options::score_targets(&context.input.parts);
    let mut candidates = Vec::with_capacity(targets.len());
    for target in &targets {
        candidates.push(score_candidate(&context, *target).await?);
    }
    let scores = score_routes(context.profile, context.requested_window, candidates);
    Ok(ScoreCandidatesOutput { targets, scores })
}

async fn score_candidate(context: &ScoreCandidatesContext<'_, '_>, target: options::ScoreTarget) -> Result<RoutingScoreCandidate, LlmProxyError> {
    let input = context.input;
    let part = options::part(&input.parts, target)?;
    let endpoint = options::endpoint(part, target)?;
    let key = options::key_for_endpoint(part, endpoint)?;
    let route = route_identity(&input.request, part, endpoint, key);
    let needs_conversion = formats::needs_conversion(&part.routing_api_format, &endpoint.api_format, input.request.is_stream)?;
    let route_config_fingerprint = current_route_config_fingerprint(input, part, endpoint, key, needs_conversion);
    let configured_cost = model_cost_config(input.state, &key.id, &part.model.id).await?;
    let price_config_fingerprint = current_price_config_fingerprint(input, configured_cost.as_ref());
    let fingerprints = RouteFingerprints {
        route_config: &route_config_fingerprint,
        price_config: &price_config_fingerprint,
    };
    let resolved = resolve_metric(
        input.state,
        context.catalog,
        &route,
        fingerprints,
        ResolveMetricOptions {
            rpm_limit: key.rpm_limit,
            min_samples: context.profile.min_samples,
            prior_sample_cap: context.profile.prior_sample_cap,
            requested_window: context.requested_window,
        },
    )
    .await?;
    let context_samples = context.context_states.samples(context.profile.id, context.context_key, &route, fingerprints);
    let ema = context.route_states.record(context.profile.id, &route, fingerprints);
    Ok(RoutingScoreCandidate {
        provider_name: Some(part.provider.name.clone()),
        key_name: Some(key.name.clone()),
        key_preview: Some(key.key_preview.clone()),
        endpoint_name: Some(endpoint.api_format.clone()),
        metric: resolved.snapshot,
        metric_window: resolved.metric_window,
        metric_freshness_seconds: resolved.metric_freshness_seconds,
        recent_metric: resolved.recent_metric,
        metric_source: resolved.metric_source,
        prior_source: resolved.prior_source,
        prior_sample_count: resolved.prior_sample_count,
        effective_sample_count: resolved.effective_sample_count,
        routing_context_key: Some(context.context_key.to_owned()),
        route_config_fingerprint: Some(route_config_fingerprint),
        price_config_fingerprint: Some(price_config_fingerprint),
        context_route_sample_count: context_samples.route_sample_count,
        context_total_sample_count: context_samples.total_sample_count,
        ema,
        admin_priority: admin_priority(input.group, part, key, endpoint, input.priority_mode),
        estimated_cost: estimated_cost_from_config(configured_cost.as_ref(), input.global_model, &input.request.features),
        needs_conversion,
        is_cached: part.is_cached,
        request_features: input.request.features.clone(),
        route,
    })
}

#[derive(Clone, Copy)]
struct ResolveMetricOptions {
    rpm_limit: Option<i32>,
    min_samples: u64,
    prior_sample_cap: u64,
    requested_window: RoutingMetricWindow,
}

async fn resolve_metric(
    state: &LlmProxyState,
    catalog: &MetricCatalog,
    route: &RouteIdentity,
    fingerprints: RouteFingerprints<'_>,
    options: ResolveMetricOptions,
) -> Result<ResolvedMetric, LlmProxyError> {
    let mut resolved = catalog.resolve(route, fingerprints, options.min_samples, options.prior_sample_cap, options.requested_window);
    if let Some(rate) = provider_key_rate_limit_snapshot(state, &route.key_id, options.rpm_limit).await? {
        resolved.snapshot.rpm_used = rate.used;
        resolved.snapshot.rpm_limit = Some(rate.limit);
    }
    Ok(resolved)
}

fn current_route_config_fingerprint(
    input: &DynamicRoutingInput<'_>,
    part: &CandidateParts,
    endpoint: &CachedEndpoint,
    key: &CachedProviderKey,
    needs_conversion: bool,
) -> String {
    route_config_fingerprint(RouteFingerprintInput {
        provider_id: &part.provider.id,
        key_id: &key.id,
        endpoint_id: &endpoint.id,
        global_model_id: &part.model.global_model_id,
        provider_model_id: &part.model.id,
        effective_upstream_model_name: &part.effective_upstream_model_name,
        effective_reasoning_effort: part.effective_reasoning_effort.as_deref(),
        client_api_format: &part.client_api_format,
        provider_api_format: &endpoint.api_format,
        is_stream: input.request.is_stream,
        needs_conversion,
    })
}

fn current_price_config_fingerprint(input: &DynamicRoutingInput<'_>, configured_cost: Option<&types::provider::ProviderModelCost>) -> String {
    price_config_fingerprint(PriceFingerprintInput {
        configured_cost,
        price_per_request: input.global_model.default_price_per_request,
        tiered_pricing: &input.global_model.default_tiered_pricing,
        billing_multiplier: input.group.billing_multiplier,
    })
}

fn route_identity(request: &CandidateRequest<'_>, part: &CandidateParts, endpoint: &CachedEndpoint, key: &CachedProviderKey) -> RouteIdentity {
    RouteIdentity {
        provider_id: part.provider.id.clone(),
        key_id: key.id.clone(),
        endpoint_id: endpoint.id.clone(),
        global_model_id: part.model.global_model_id.clone(),
        client_api_format: part.client_api_format.clone(),
        provider_api_format: endpoint.api_format.clone(),
        is_stream: request.is_stream,
    }
}

fn admin_priority(group: &CachedBillingGroup, part: &CandidateParts, key: &CachedProviderKey, endpoint: &CachedEndpoint, mode: ProviderPriorityMode) -> i32 {
    let provider_priority = group.provider_priorities.get(&part.provider.id).copied().unwrap_or(part.provider.priority);
    provider_priority + key_priority(group, key, &endpoint.api_format, mode)
}

fn key_priority(group: &CachedBillingGroup, key: &CachedProviderKey, api_format: &str, mode: ProviderPriorityMode) -> i32 {
    group.provider_key_priorities.get(&key.id).copied().unwrap_or_else(|| match mode {
        ProviderPriorityMode::Provider => key.internal_priority,
        ProviderPriorityMode::Key => key.global_priority_by_format.get(api_format).copied().unwrap_or(key.internal_priority),
    })
}

fn requested_windows(window: RoutingMetricWindow, live_stable: bool) -> Vec<RoutingMetricWindow> {
    if live_stable {
        return LIVE_WINDOWS.to_vec();
    }
    vec![window]
}

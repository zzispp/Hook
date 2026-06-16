use std::collections::HashMap;

use storage::provider::{ProviderStore, RoutingMetricRecord, RoutingRouteStateRecord};
use time::OffsetDateTime;
use types::provider::{
    ProviderPriorityMode, RouteIdentity, RouteScoreExplanation, RoutingMetricSnapshot, RoutingMetricWindow, RoutingProfile, RoutingProfileId,
};

use super::{CandidateParts, GlobalModelRef, dynamic_cost::estimated_cost};
use crate::llm_proxy::{
    LlmProxyError, LlmProxyState,
    cache::snapshot::{CachedBillingGroup, CachedEndpoint, CachedProviderKey},
    candidate::CandidateRequest,
    formats,
    rate_limit::provider_key_rate_limit_snapshot,
    routing::{RoutingEmaSnapshot, RoutingScoreCandidate, circuit, profile_by_id, score_routes},
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
    let catalog = metric_catalog(input.state, requested_windows(window, live_stable)).await?;
    let scores = score_candidates(&input, &catalog, &profile, window).await?;
    let explanations = scores.iter().map(|item| item.explanation.clone()).collect::<Vec<_>>();
    let parts = ordered_parts(input.parts, &scores)?;
    if parts.is_empty() && !input.allow_empty {
        return Err(LlmProxyError::Upstream("all provider candidates are excluded by dynamic routing".into()));
    }
    Ok(DynamicRoutingOutput { parts, profile, explanations })
}

async fn score_candidates(
    input: &DynamicRoutingInput<'_>,
    catalog: &MetricCatalog,
    profile: &RoutingProfile,
    requested_window: RoutingMetricWindow,
) -> Result<Vec<crate::llm_proxy::routing::ScoredRoute>, LlmProxyError> {
    let routes = input.parts.iter().map(|part| route_identity(input.request, part)).collect::<Vec<_>>();
    let route_states = route_state_catalog(input.state, &routes).await?;
    let mut candidates = Vec::with_capacity(input.parts.len());
    for part in &input.parts {
        candidates.push(score_candidate(input, catalog, &route_states, profile, requested_window, part).await?);
    }
    Ok(score_routes(profile, requested_window, candidates))
}

async fn score_candidate(
    input: &DynamicRoutingInput<'_>,
    catalog: &MetricCatalog,
    route_states: &RouteStateCatalog,
    profile: &RoutingProfile,
    requested_window: RoutingMetricWindow,
    part: &CandidateParts,
) -> Result<RoutingScoreCandidate, LlmProxyError> {
    let route = route_identity(input.request, part);
    let resolved = resolve_metric(input.state, catalog, &route, primary_key(part).rpm_limit, profile.min_samples, requested_window).await?;
    let endpoint = primary_endpoint(part);
    let key = primary_key(part);
    Ok(RoutingScoreCandidate {
        provider_name: Some(part.provider.name.clone()),
        key_name: Some(key.name.clone()),
        key_preview: Some(key.key_preview.clone()),
        endpoint_name: Some(endpoint.api_format.clone()),
        metric: resolved.snapshot,
        metric_window: resolved.metric_window,
        metric_freshness_seconds: resolved.metric_freshness_seconds,
        recent_metric: resolved.recent_metric,
        ema: route_states.record(&route),
        circuit_state: circuit::candidate_state(input.state, &route).await?,
        admin_priority: admin_priority(input.group, part, input.priority_mode),
        estimated_cost: estimated_cost(input.state, &key.id, &part.model.id, input.global_model).await?,
        needs_conversion: formats::needs_conversion(&part.routing_api_format, &endpoint.api_format, input.request.is_stream)?,
        is_cached: part.is_cached,
        route,
    })
}

fn ordered_parts(parts: Vec<CandidateParts>, scores: &[crate::llm_proxy::routing::ScoredRoute]) -> Result<Vec<CandidateParts>, LlmProxyError> {
    let mut by_index = parts.into_iter().map(Some).collect::<Vec<_>>();
    let mut output = Vec::new();
    for score in scores.iter().filter(|item| !item.excluded) {
        let part = by_index
            .get_mut(score.original_index)
            .and_then(Option::take)
            .ok_or_else(|| LlmProxyError::Infrastructure("dynamic routing score referenced an invalid candidate index".into()))?;
        output.push(part);
    }
    Ok(output)
}

async fn metric_catalog(state: &LlmProxyState, windows: Vec<RoutingMetricWindow>) -> Result<MetricCatalog, LlmProxyError> {
    let store = ProviderStore::new(state.database.clone());
    let mut entries = Vec::with_capacity(windows.len());
    for window in windows {
        let records = store.list_routing_metrics(window).await?;
        entries.push(MetricCatalogEntry {
            window,
            records: records.into_iter().map(|record| (record.route.clone(), record)).collect(),
        });
    }
    Ok(MetricCatalog { entries })
}

async fn resolve_metric(
    state: &LlmProxyState,
    catalog: &MetricCatalog,
    route: &RouteIdentity,
    rpm_limit: Option<i32>,
    min_samples: u64,
    requested_window: RoutingMetricWindow,
) -> Result<ResolvedMetric, LlmProxyError> {
    let source = catalog
        .best_record(route, min_samples)
        .or_else(|| catalog.richest_record(route))
        .or_else(|| catalog.record(route, requested_window));
    let recent_metric = if source.map(|(window, _)| window) != Some(requested_window) {
        catalog.record(route, requested_window).map(|(_, record)| record.snapshot.clone())
    } else {
        None
    };
    let mut snapshot = source.map(|(_, record)| record.snapshot.clone()).unwrap_or_default();
    if let Some(rate) = provider_key_rate_limit_snapshot(state, &route.key_id, rpm_limit).await? {
        snapshot.rpm_used = rate.used;
        snapshot.rpm_limit = Some(rate.limit);
    }
    Ok(ResolvedMetric {
        snapshot,
        metric_window: source.map(|(window, _)| window).unwrap_or(requested_window),
        metric_freshness_seconds: source.map(|(_, record)| freshness_seconds(record)).unwrap_or(0),
        recent_metric,
    })
}

fn route_identity(request: CandidateRequest<'_>, part: &CandidateParts) -> RouteIdentity {
    RouteIdentity {
        provider_id: part.provider.id.clone(),
        key_id: primary_key(part).id.clone(),
        endpoint_id: primary_endpoint(part).id.clone(),
        global_model_id: part.model.global_model_id.clone(),
        client_api_format: part.client_api_format.clone(),
        provider_api_format: primary_endpoint(part).api_format.clone(),
        is_stream: request.is_stream,
    }
}

fn admin_priority(group: &CachedBillingGroup, part: &CandidateParts, mode: ProviderPriorityMode) -> i32 {
    let provider_priority = group.provider_priorities.get(&part.provider.id).copied().unwrap_or(part.provider.priority);
    provider_priority + key_priority(group, primary_key(part), &primary_endpoint(part).api_format, mode)
}

fn key_priority(group: &CachedBillingGroup, key: &CachedProviderKey, api_format: &str, mode: ProviderPriorityMode) -> i32 {
    group.provider_key_priorities.get(&key.id).copied().unwrap_or_else(|| match mode {
        ProviderPriorityMode::Provider => key.internal_priority,
        ProviderPriorityMode::Key => key.global_priority_by_format.get(api_format).copied().unwrap_or(key.internal_priority),
    })
}

fn primary_endpoint(parts: &CandidateParts) -> &CachedEndpoint {
    &parts.endpoints[0]
}

fn primary_key(parts: &CandidateParts) -> &CachedProviderKey {
    let endpoint = primary_endpoint(parts);
    parts
        .keys
        .iter()
        .find(|key| key.api_formats.iter().any(|format| format == &endpoint.api_format))
        .expect("candidate parts must contain at least one key for the primary endpoint")
}

fn requested_windows(window: RoutingMetricWindow, live_stable: bool) -> Vec<RoutingMetricWindow> {
    if live_stable {
        return LIVE_WINDOWS.to_vec();
    }
    vec![window]
}

fn freshness_seconds(record: &RoutingMetricRecord) -> i64 {
    (OffsetDateTime::now_utc() - record.last_seen_at).whole_seconds().max(0)
}

async fn route_state_catalog(state: &LlmProxyState, routes: &[RouteIdentity]) -> Result<RouteStateCatalog, LlmProxyError> {
    let records = ProviderStore::new(state.database.clone()).list_routing_route_states_for_routes(routes).await?;
    Ok(RouteStateCatalog {
        records: records.into_iter().map(|record| (record.route.clone(), record)).collect(),
    })
}

fn route_state_freshness_seconds(record: &RoutingRouteStateRecord) -> i64 {
    (OffsetDateTime::now_utc() - record.last_updated_at).whole_seconds().max(0)
}

struct MetricCatalog {
    entries: Vec<MetricCatalogEntry>,
}

impl MetricCatalog {
    fn record(&self, route: &RouteIdentity, window: RoutingMetricWindow) -> Option<(RoutingMetricWindow, &RoutingMetricRecord)> {
        self.entries
            .iter()
            .find(|entry| entry.window == window)
            .and_then(|entry| entry.records.get(route).map(|record| (entry.window, record)))
    }

    fn best_record(&self, route: &RouteIdentity, min_samples: u64) -> Option<(RoutingMetricWindow, &RoutingMetricRecord)> {
        self.entries.iter().find_map(|entry| {
            entry
                .records
                .get(route)
                .filter(|record| record.snapshot.sample_count >= min_samples)
                .map(|record| (entry.window, record))
        })
    }

    fn richest_record(&self, route: &RouteIdentity) -> Option<(RoutingMetricWindow, &RoutingMetricRecord)> {
        self.entries
            .iter()
            .filter_map(|entry| entry.records.get(route).map(|record| (entry.window, record)))
            .max_by_key(|(_, record)| record.snapshot.sample_count)
    }
}

struct MetricCatalogEntry {
    window: RoutingMetricWindow,
    records: HashMap<RouteIdentity, RoutingMetricRecord>,
}

struct RouteStateCatalog {
    records: HashMap<RouteIdentity, RoutingRouteStateRecord>,
}

impl RouteStateCatalog {
    fn record(&self, route: &RouteIdentity) -> Option<RoutingEmaSnapshot> {
        let record = self.records.get(route)?;
        Some(RoutingEmaSnapshot {
            success_rate: record.ema_success_rate,
            ttfb_avg_ms: record.ema_ttfb_ms,
            latency_avg_ms: record.ema_latency_ms,
            output_tps: record.ema_output_tps,
            sample_count: record.sample_count,
            freshness_seconds: route_state_freshness_seconds(record),
        })
    }
}

struct ResolvedMetric {
    snapshot: RoutingMetricSnapshot,
    metric_window: RoutingMetricWindow,
    metric_freshness_seconds: i64,
    recent_metric: Option<RoutingMetricSnapshot>,
}

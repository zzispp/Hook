mod dynamic_routing;
mod matching;
mod proxy_candidate;
mod route;
mod scheduler;
#[cfg(test)]
mod tests;

use types::{
    api_token::ApiToken,
    model::TieredPricingConfig,
    provider::{
        ProviderSchedulingMode, RouteScoreExplanation, RoutingAffinitySummary, RoutingCacheAffinityMode, RoutingMetricWindow, RoutingPreviewRequest,
        RoutingPreviewResponse, RoutingProfile, RoutingProfileId, RoutingRankingResponse, RoutingRankingsRequest, RoutingSimulationMode,
    },
};
use uuid::Uuid;

use self::{
    matching::{MatchingCandidatePartsInput, matching_candidate_parts},
    proxy_candidate::{ProxyCandidateBuildInput, proxy_candidates},
    scheduler::{OrderCandidatePartsInput, order_candidate_parts},
};
use super::{CandidateRequest, CandidateSelection, LlmProxyError, LlmProxyState};
use crate::llm_proxy::{
    AffinityRecord, AffinitySelection,
    cache::snapshot::{CachedEndpoint, CachedGlobalModel, CachedModelBinding, CachedProvider, CachedProviderKey, SchedulingSnapshot},
    model_access::{active_group, ensure_group_allows_model, ensure_token_allows_model, ensure_user_allows_model},
};

pub(super) use crate::llm_proxy::model_access::{token_user_for_snapshot, user_access_for_token};

pub(super) const DEFAULT_MAX_RETRIES: i32 = 2;

pub(super) type CandidatePartKey = (String, String, String);

pub async fn select_candidates(state: &LlmProxyState, token: &ApiToken, request: CandidateRequest<'_>) -> Result<CandidateSelection, LlmProxyError> {
    let request_id = Uuid::now_v7().to_string();
    let snapshot = state.scheduling_snapshot().await?;
    let model = resolve_global_model(&snapshot, request.model_name)?;
    ensure_token_allows_model(token, &model.id)?;
    let token_user = token_user_for_snapshot(&snapshot, token)?;
    let user_access = user_access_for_token(token, token_user);
    ensure_user_allows_model(user_access, &model.id)?;
    let group = active_group(&snapshot, token, user_access)?;
    ensure_group_allows_model(group, &model.id)?;
    let profile_id = effective_routing_profile_id(group, &model);
    let profile = crate::llm_proxy::routing::profile_by_id(state, profile_id).await?.profile;
    let affinity_context = cache_affinity_context(state, &snapshot, profile.cache_affinity_mode, Some(&token.id), &model.id, request.api_format).await?;
    let affinity = affinity_context.selection();
    let ordered_affinity = affinity_context.prefer_cached_selection(affinity.as_ref());
    let cooled_provider_ids = cooled_provider_ids(state, &snapshot).await?;
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access,
        model_id: &model.id,
        request,
        affinity: ordered_affinity,
        scheduling_mode: snapshot.scheduling_mode,
        request_id: &request_id,
        cooled_provider_ids: &cooled_provider_ids,
    });
    if parts.is_empty() {
        return Err(LlmProxyError::NotFound(format!("no active provider candidate for model {}", model.name)));
    }

    let ordered = order_candidate_parts(OrderCandidatePartsInput {
        parts,
        token,
        group,
        user_access,
        request,
        model_id: &model.id,
        request_id: &request_id,
        affinity: ordered_affinity.cloned(),
        mode: snapshot.scheduling_mode,
        priority_mode: snapshot.provider_priority_mode,
    })?;
    let ordered = mark_score_bonus_affinity(ordered, affinity.as_ref(), affinity_context.mode);
    let routed = dynamic_routing::rank_candidate_parts_with_profile(
        dynamic_routing::DynamicRoutingInput {
            state,
            parts: ordered,
            group,
            request_id: &request_id,
            request,
            global_model: &model,
            priority_mode: snapshot.provider_priority_mode,
            allow_empty: false,
        },
        profile,
        RoutingMetricWindow::default(),
        true,
    )
    .await?;
    let candidates = proxy_candidates(ProxyCandidateBuildInput {
        state,
        token,
        request,
        global_model: &model,
        group,
        token_user,
        cache_affinity_enabled: affinity_context.enabled(),
        parts: &routed.parts,
    })
    .await?;
    Ok(CandidateSelection {
        request_id,
        cache_affinity_ttl_minutes: affinity_context.ttl_minutes(snapshot.cache_affinity_ttl_minutes),
        routing_profile_id: Some(routed.profile.id),
        routing_profile_version: Some(routed.profile.version),
        routing_explanations: routed.explanations,
        candidates,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EffectiveCacheAffinityMode {
    Disabled,
    ScoreBonus,
    PreferCached,
}

struct CacheAffinityContext {
    mode: EffectiveCacheAffinityMode,
    record: Option<AffinityRecord>,
}

impl CacheAffinityContext {
    fn enabled(&self) -> bool {
        self.mode != EffectiveCacheAffinityMode::Disabled
    }

    fn selection(&self) -> Option<AffinitySelection> {
        self.record.as_ref().map(AffinitySelection::from)
    }

    fn prefer_cached_selection<'a>(&self, selection: Option<&'a AffinitySelection>) -> Option<&'a AffinitySelection> {
        (self.mode == EffectiveCacheAffinityMode::PreferCached).then_some(selection).flatten()
    }

    fn ttl_minutes(&self, configured: i64) -> i64 {
        if self.enabled() { configured } else { 0 }
    }
}

async fn cache_affinity_context(
    state: &LlmProxyState,
    snapshot: &SchedulingSnapshot,
    profile_mode: RoutingCacheAffinityMode,
    token_id: Option<&str>,
    model_id: &str,
    api_format: &str,
) -> Result<CacheAffinityContext, LlmProxyError> {
    let mode = effective_cache_affinity_mode(snapshot.scheduling_mode, profile_mode);
    let record = match (mode, token_id) {
        (EffectiveCacheAffinityMode::Disabled, _) | (_, None) => None,
        (_, Some(token_id)) => state.cached_affinity(token_id, model_id, api_format).await?,
    };
    Ok(CacheAffinityContext { mode, record })
}

fn effective_cache_affinity_mode(scheduling_mode: ProviderSchedulingMode, profile_mode: RoutingCacheAffinityMode) -> EffectiveCacheAffinityMode {
    if scheduling_mode != ProviderSchedulingMode::CacheAffinity {
        return EffectiveCacheAffinityMode::Disabled;
    }
    match profile_mode {
        RoutingCacheAffinityMode::Disabled => EffectiveCacheAffinityMode::Disabled,
        RoutingCacheAffinityMode::ScoreBonus => EffectiveCacheAffinityMode::ScoreBonus,
        RoutingCacheAffinityMode::PreferCached => EffectiveCacheAffinityMode::PreferCached,
    }
}

fn mark_score_bonus_affinity(parts: Vec<CandidateParts>, affinity: Option<&AffinitySelection>, mode: EffectiveCacheAffinityMode) -> Vec<CandidateParts> {
    if mode == EffectiveCacheAffinityMode::Disabled {
        return parts;
    }
    parts.into_iter().map(|part| mark_affinity_part(part, affinity)).collect()
}

fn mark_affinity_part(mut part: CandidateParts, affinity: Option<&AffinitySelection>) -> CandidateParts {
    part.affinity_bonus = part.affinity_bonus || affinity.is_some_and(|affinity| part_matches_affinity(&part, affinity));
    part
}

fn part_matches_affinity(part: &CandidateParts, affinity: &AffinitySelection) -> bool {
    let Some(endpoint) = part.endpoints.first() else {
        return false;
    };
    if part.provider.id != affinity.provider_id || endpoint.id != affinity.endpoint_id {
        return false;
    }
    part.keys
        .iter()
        .any(|key| key.id == affinity.key_id && key.api_formats.iter().any(|format| format == &endpoint.api_format))
}

pub(super) async fn routing_rankings(state: &LlmProxyState, request: RoutingRankingsRequest) -> Result<RoutingRankingResponse, LlmProxyError> {
    let window = request.window;
    let simulation_mode = request.simulation_mode;
    let profile = crate::llm_proxy::routing::profile_by_id(state, request.profile_id).await?.profile;
    let output = preview_output(state, PreviewInput::from_ranking(request)?, profile.clone(), window).await?;
    Ok(RoutingRankingResponse {
        profile,
        window,
        simulation_mode,
        affinity: output.affinity,
        items: output.items,
    })
}

pub(super) async fn routing_preview(state: &LlmProxyState, request: RoutingPreviewRequest) -> Result<RoutingPreviewResponse, LlmProxyError> {
    let profile = crate::llm_proxy::routing::profile_by_id(state, request.profile_id).await?.profile;
    let items = preview_output(state, PreviewInput::from_preview(request), profile.clone(), RoutingMetricWindow::default()).await?;
    Ok(RoutingPreviewResponse { profile, items: items.items })
}

async fn preview_output(
    state: &LlmProxyState,
    input: PreviewInput,
    profile: RoutingProfile,
    window: RoutingMetricWindow,
) -> Result<PreviewOutput, LlmProxyError> {
    let snapshot = state.scheduling_snapshot().await?;
    let group = snapshot
        .groups
        .iter()
        .find(|group| group.code == input.group_code)
        .ok_or_else(|| LlmProxyError::InvalidRequest(format!("billing group not found for routing preview: {}", input.group_code)))?;
    let model = resolve_global_model(&snapshot, &input.model)?;
    let request = CandidateRequest {
        api_format: &input.api_format,
        routing_api_format: &input.api_format,
        model_name: &input.model,
        is_stream: input.is_stream,
        has_openai_responses_custom_tool_items: false,
        required_capability: None,
    };
    let affinity_context = preview_affinity_context(state, &snapshot, &profile, &input, &model).await?;
    let affinity = affinity_context.selection();
    let ordered_affinity = affinity_context.prefer_cached_selection(affinity.as_ref());
    let cooled_provider_ids = cooled_provider_ids(state, &snapshot).await?;
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: &model.id,
        request,
        affinity: ordered_affinity,
        scheduling_mode: snapshot.scheduling_mode,
        request_id: "routing-preview",
        cooled_provider_ids: &cooled_provider_ids,
    });
    let parts = mark_score_bonus_affinity(parts, affinity.as_ref(), affinity_context.mode);
    let output = dynamic_routing::rank_candidate_parts_with_profile(
        dynamic_routing::DynamicRoutingInput {
            state,
            parts,
            group,
            request_id: "routing-preview",
            request,
            global_model: &model,
            priority_mode: snapshot.provider_priority_mode,
            allow_empty: true,
        },
        profile,
        window,
        input.simulation_mode == RoutingSimulationMode::Production,
    )
    .await?;
    Ok(PreviewOutput {
        items: filter_explanations(output.explanations, input.include_excluded),
        affinity: affinity_context.record.as_ref().map(affinity_summary),
    })
}

struct PreviewInput {
    group_code: String,
    model: String,
    api_format: String,
    is_stream: bool,
    include_excluded: bool,
    simulation_mode: RoutingSimulationMode,
    token_id: Option<String>,
}

struct PreviewOutput {
    items: Vec<RouteScoreExplanation>,
    affinity: Option<RoutingAffinitySummary>,
}

impl PreviewInput {
    fn from_ranking(request: RoutingRankingsRequest) -> Result<Self, LlmProxyError> {
        Ok(Self {
            group_code: required_filter(request.group_code, "group_code")?,
            model: required_filter(request.model, "model")?,
            api_format: required_filter(request.api_format, "api_format")?,
            is_stream: request.is_stream.unwrap_or(false),
            include_excluded: request.include_excluded,
            simulation_mode: request.simulation_mode,
            token_id: request.token_id,
        })
    }

    fn from_preview(request: RoutingPreviewRequest) -> Self {
        Self {
            group_code: request.group_code,
            model: request.model,
            api_format: request.api_format,
            is_stream: request.is_stream,
            include_excluded: true,
            simulation_mode: RoutingSimulationMode::Window,
            token_id: None,
        }
    }
}

async fn preview_affinity_context(
    state: &LlmProxyState,
    snapshot: &SchedulingSnapshot,
    profile: &RoutingProfile,
    input: &PreviewInput,
    model: &GlobalModelRef,
) -> Result<CacheAffinityContext, LlmProxyError> {
    if input.simulation_mode != RoutingSimulationMode::Production {
        return Ok(CacheAffinityContext {
            mode: EffectiveCacheAffinityMode::Disabled,
            record: None,
        });
    }
    cache_affinity_context(
        state,
        snapshot,
        profile.cache_affinity_mode,
        input.token_id.as_deref(),
        &model.id,
        &input.api_format,
    )
    .await
}

fn affinity_summary(record: &AffinityRecord) -> RoutingAffinitySummary {
    RoutingAffinitySummary {
        provider_id: record.provider_id.clone(),
        endpoint_id: record.endpoint_id.clone(),
        key_id: record.key_id.clone(),
        api_format: record.api_format.clone(),
        model_id: record.model_id.clone(),
        request_count: record.request_count,
    }
}

fn required_filter(value: Option<String>, name: &str) -> Result<String, LlmProxyError> {
    value.ok_or_else(|| LlmProxyError::InvalidRequest(format!("routing rankings requires {name}")))
}

fn filter_explanations(items: Vec<RouteScoreExplanation>, include_excluded: bool) -> Vec<RouteScoreExplanation> {
    if include_excluded {
        return items;
    }
    items.into_iter().filter(|item| item.exclusion_reason.is_none()).collect()
}

#[derive(Clone)]
pub(super) struct GlobalModelRef {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) default_price_per_request: Option<rust_decimal::Decimal>,
    pub(super) default_tiered_pricing: TieredPricingConfig,
    pub(super) routing_profile_id: Option<RoutingProfileId>,
}

#[derive(Clone)]
pub(super) struct CandidateParts {
    pub(super) provider: CachedProvider,
    pub(super) endpoints: Vec<CachedEndpoint>,
    pub(super) keys: Vec<CachedProviderKey>,
    pub(super) model: CachedModelBinding,
    pub(super) client_api_format: String,
    pub(super) routing_api_format: String,
    pub(super) affinity_bonus: bool,
    pub(super) is_cached: bool,
}

fn resolve_global_model(snapshot: &SchedulingSnapshot, model_name: &str) -> Result<GlobalModelRef, LlmProxyError> {
    let record = snapshot
        .models
        .iter()
        .find(|model| model.name == model_name)
        .or_else(|| snapshot.models.iter().find(|model| model.id == model_name))
        .ok_or_else(|| LlmProxyError::NotFound(format!("model not found: {model_name}")))?;
    if !record.is_active {
        return Err(LlmProxyError::Forbidden(format!("model is inactive: {}", record.name)));
    }
    Ok(model_ref(record))
}

fn model_ref(model: &CachedGlobalModel) -> GlobalModelRef {
    GlobalModelRef {
        id: model.id.clone(),
        name: model.name.clone(),
        default_price_per_request: model.default_price_per_request,
        default_tiered_pricing: model.default_tiered_pricing.clone(),
        routing_profile_id: model.routing_profile_id,
    }
}

fn effective_routing_profile_id(group: &crate::llm_proxy::cache::snapshot::CachedBillingGroup, model: &GlobalModelRef) -> RoutingProfileId {
    model.routing_profile_id.or(group.routing_profile_id).unwrap_or(RoutingProfileId::Balanced)
}

async fn cooled_provider_ids(state: &LlmProxyState, snapshot: &SchedulingSnapshot) -> Result<std::collections::HashSet<String>, LlmProxyError> {
    let provider_ids = snapshot.providers.iter().map(|provider| provider.id.clone()).collect::<Vec<_>>();
    state.cooled_provider_ids(&provider_ids).await
}

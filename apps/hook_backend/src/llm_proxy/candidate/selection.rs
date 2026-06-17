mod dynamic_cost;
mod dynamic_routing;
mod matching;
mod matching_order;
mod proxy_candidate;
mod route;
mod routing_metrics;
mod scheduler;
#[cfg(test)]
mod tests;
mod token_context;

use types::{
    api_token::ApiToken,
    model::TieredPricingConfig,
    provider::{RouteIdentity, RouteScoreExplanation, RoutingProfileId, RoutingRankingResponse, RoutingRankingsRequest, RoutingRequestFeatures},
};
use uuid::Uuid;

use self::{
    proxy_candidate::{ProxyCandidateBuildInput, proxy_candidates},
    token_context::token_routing_context,
};
use super::{CandidateRequest, CandidateSelection, LlmProxyError, LlmProxyState};
use crate::llm_proxy::{
    cache::snapshot::{CachedBillingGroup, CachedEndpoint, CachedGlobalModel, CachedModelBinding, CachedProvider, CachedProviderKey, SchedulingSnapshot},
    candidate::selection::{
        matching::{MatchingCandidatePartsInput, matching_candidate_parts},
        scheduler::{OrderCandidatePartsInput, order_candidate_parts},
    },
    model_access::ensure_group_allows_model,
};
use proxy::scheduler::ModelAccessPolicy;

pub(super) const DEFAULT_MAX_RETRIES: i32 = 2;

pub(super) type CandidatePartKey = (String, String, String);

pub async fn select_candidates(state: &LlmProxyState, token: &ApiToken, request: CandidateRequest<'_>) -> Result<CandidateSelection, LlmProxyError> {
    let request_id = Uuid::now_v7().to_string();
    let context = token_routing_context(state, token, request.clone(), request_id.clone()).await?;
    let ordered = context.ordered_parts(state, token, request.clone()).await?;
    let routed = dynamic_routing::rank_candidate_parts(dynamic_routing::DynamicRoutingInput {
        state,
        parts: ordered,
        group: &context.group,
        request: request.clone(),
        global_model: &context.global_model,
        profile_id: context.routing_profile_id,
        priority_mode: context.priority_mode,
        allow_empty: false,
    })
    .await?;
    let candidates = proxy_candidates(ProxyCandidateBuildInput {
        state,
        token,
        request,
        global_model: &context.global_model,
        group: &context.group,
        token_user: context.token_user.as_ref(),
        parts: &routed.parts,
    })
    .await?;
    Ok(CandidateSelection {
        request_id,
        cache_affinity_ttl_minutes: context.cache_affinity_ttl_minutes,
        routing_profile_id: Some(routed.profile.id),
        routing_profile_version: Some(routed.profile.version),
        routing_explanations: routed.explanations,
        candidates,
    })
}

pub(super) async fn routing_rankings(state: &LlmProxyState, request: RoutingRankingsRequest) -> Result<RoutingRankingResponse, LlmProxyError> {
    validate_rankings_request(&request)?;
    let request_id_seed = request_id_seed(request.request_id_seed)?;
    let window = request.window;
    let include_excluded = request.include_excluded;
    let candidate_request = CandidateRequest {
        api_format: &request.api_format,
        routing_api_format: &request.api_format,
        model_name: &request.model,
        is_stream: request.is_stream,
        has_openai_responses_custom_tool_items: false,
        required_capability: None,
        features: RoutingRequestFeatures::unknown(&request.api_format, request.is_stream, None),
    };
    let context = ranking_context(state, &request.group_code, candidate_request.clone(), request_id_seed.clone()).await?;
    let profile = crate::llm_proxy::routing::profile_by_id(state, context.routing_profile_id).await?.profile;
    let output = dynamic_routing::rank_candidate_parts_with_profile(
        dynamic_routing::DynamicRoutingInput {
            state,
            parts: context.parts,
            group: &context.group,
            request: candidate_request,
            global_model: &context.global_model,
            profile_id: profile.id,
            priority_mode: context.priority_mode,
            allow_empty: true,
        },
        profile,
        window,
    )
    .await?;
    let selected = selected_route(&output.explanations);
    Ok(RoutingRankingResponse {
        profile: output.profile,
        window,
        selected,
        request_id_seed,
        items: filter_explanations(output.explanations, include_excluded),
    })
}

pub(super) struct RankingContext {
    group: CachedBillingGroup,
    global_model: GlobalModelRef,
    routing_profile_id: RoutingProfileId,
    priority_mode: types::provider::ProviderPriorityMode,
    parts: Vec<CandidateParts>,
}

fn filter_explanations(items: Vec<RouteScoreExplanation>, include_excluded: bool) -> Vec<RouteScoreExplanation> {
    if include_excluded {
        return items;
    }
    items.into_iter().filter(|item| item.exclusion_reason.is_none()).collect()
}

fn request_id_seed(value: Option<String>) -> Result<String, LlmProxyError> {
    match value {
        Some(seed) if seed.trim().is_empty() => Err(LlmProxyError::InvalidRequest("request_id_seed cannot be empty".into())),
        Some(seed) => Ok(seed),
        None => Ok(Uuid::now_v7().to_string()),
    }
}

fn selected_route(items: &[RouteScoreExplanation]) -> Option<RouteIdentity> {
    items.iter().find(|item| item.exclusion_reason.is_none()).map(|item| item.route.clone())
}

fn validate_rankings_request(request: &RoutingRankingsRequest) -> Result<(), LlmProxyError> {
    require_non_empty(&request.group_code, "group_code")?;
    require_non_empty(&request.model, "model")?;
    require_non_empty(&request.api_format, "api_format")
}

fn require_non_empty(value: &str, name: &str) -> Result<(), LlmProxyError> {
    if value.trim().is_empty() {
        return Err(LlmProxyError::InvalidRequest(format!("routing rankings requires {name}")));
    }
    Ok(())
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
    pub(super) is_cached: bool,
}

pub(super) fn resolve_global_model(snapshot: &SchedulingSnapshot, model_name: &str) -> Result<GlobalModelRef, LlmProxyError> {
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

pub(super) fn effective_routing_profile_id(group: &crate::llm_proxy::cache::snapshot::CachedBillingGroup, model: &GlobalModelRef) -> RoutingProfileId {
    model.routing_profile_id.or(group.routing_profile_id).unwrap_or(RoutingProfileId::Balanced)
}

pub(super) async fn cooled_provider_ids(state: &LlmProxyState, snapshot: &SchedulingSnapshot) -> Result<std::collections::HashSet<String>, LlmProxyError> {
    let provider_ids = snapshot.providers.iter().map(|provider| provider.id.clone()).collect::<Vec<_>>();
    state.cooled_provider_ids(&provider_ids).await
}

async fn ranking_context(state: &LlmProxyState, group_code: &str, request: CandidateRequest<'_>, request_id: String) -> Result<RankingContext, LlmProxyError> {
    let snapshot = state.scheduling_snapshot().await?;
    let cooled_provider_ids = cooled_provider_ids(state, &snapshot).await?;
    ranking_context_from_snapshot(&snapshot, group_code, request, &request_id, &cooled_provider_ids)
}

fn ranking_group(snapshot: &SchedulingSnapshot, group_code: &str) -> Result<CachedBillingGroup, LlmProxyError> {
    let group = snapshot
        .groups
        .iter()
        .find(|group| group.code == group_code)
        .ok_or_else(|| LlmProxyError::InvalidRequest(format!("billing group not found: {group_code}")))?;
    if !group.is_active {
        return Err(LlmProxyError::Forbidden(format!("billing group is inactive: {}", group.code)));
    }
    Ok(group.clone())
}

pub(super) fn ranking_context_from_snapshot(
    snapshot: &SchedulingSnapshot,
    group_code: &str,
    request: CandidateRequest<'_>,
    request_id: &str,
    cooled_provider_ids: &std::collections::HashSet<String>,
) -> Result<RankingContext, LlmProxyError> {
    let global_model = resolve_global_model(snapshot, request.model_name)?;
    let group = ranking_group(snapshot, group_code)?;
    ensure_group_allows_model(&group, &global_model.id)?;
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot,
        group: &group,
        user_access: None,
        model_id: &global_model.id,
        request: request.clone(),
        affinity: None,
        scheduling_mode: snapshot.scheduling_mode,
        request_id,
        cooled_provider_ids,
    });
    let parts = order_candidate_parts(OrderCandidatePartsInput {
        parts,
        group: &group,
        user_access: None,
        model_access_policy: ModelAccessPolicy::All,
        request,
        model_id: &global_model.id,
        request_id,
        affinity: None,
        mode: snapshot.scheduling_mode,
        priority_mode: snapshot.provider_priority_mode,
    })?;
    Ok(RankingContext {
        routing_profile_id: effective_routing_profile_id(&group, &global_model),
        priority_mode: snapshot.provider_priority_mode,
        group,
        global_model,
        parts,
    })
}

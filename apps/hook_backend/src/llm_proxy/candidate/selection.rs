mod dynamic_cost;
mod dynamic_routing;
mod matching;
mod proxy_candidate;
mod route;
mod scheduler;
#[cfg(test)]
mod tests;
mod token_context;

use storage::api_token::ApiTokenStore;
use types::{
    api_token::ApiToken,
    model::TieredPricingConfig,
    provider::{RouteIdentity, RouteScoreExplanation, RoutingProfileId, RoutingRankingResponse, RoutingRankingsRequest},
};
use uuid::Uuid;

use self::{
    proxy_candidate::{ProxyCandidateBuildInput, proxy_candidates},
    token_context::token_routing_context,
};
use super::{CandidateRequest, CandidateSelection, LlmProxyError, LlmProxyState};
use crate::llm_proxy::cache::snapshot::{CachedEndpoint, CachedGlobalModel, CachedModelBinding, CachedProvider, CachedProviderKey, SchedulingSnapshot};

pub(super) use crate::llm_proxy::model_access::{token_user_for_snapshot, user_access_for_token};

pub(super) const DEFAULT_MAX_RETRIES: i32 = 2;

pub(super) type CandidatePartKey = (String, String, String);

pub async fn select_candidates(state: &LlmProxyState, token: &ApiToken, request: CandidateRequest<'_>) -> Result<CandidateSelection, LlmProxyError> {
    let request_id = Uuid::now_v7().to_string();
    let context = token_routing_context(state, token, request, request_id.clone()).await?;
    let ordered = context.ordered_parts(state, token, request).await?;
    let routed = dynamic_routing::rank_candidate_parts(dynamic_routing::DynamicRoutingInput {
        state,
        parts: ordered,
        group: &context.group,
        request,
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
    let token = routing_token(state, &request.api_token_id).await?;
    let candidate_request = CandidateRequest {
        api_format: &request.api_format,
        routing_api_format: &request.api_format,
        model_name: &request.model,
        is_stream: request.is_stream,
        has_openai_responses_custom_tool_items: false,
        required_capability: None,
    };
    let context = token_routing_context(state, &token, candidate_request, request_id_seed.clone()).await?;
    let profile = crate::llm_proxy::routing::profile_by_id(state, context.routing_profile_id).await?.profile;
    let parts = context.ordered_parts(state, &token, candidate_request).await?;
    let output = dynamic_routing::rank_candidate_parts_with_profile(
        dynamic_routing::DynamicRoutingInput {
            state,
            parts,
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

fn filter_explanations(items: Vec<RouteScoreExplanation>, include_excluded: bool) -> Vec<RouteScoreExplanation> {
    if include_excluded {
        return items;
    }
    items.into_iter().filter(|item| item.exclusion_reason.is_none()).collect()
}

async fn routing_token(state: &LlmProxyState, token_id: &str) -> Result<ApiToken, LlmProxyError> {
    let token = ApiTokenStore::new(state.database())
        .find_token(token_id)
        .await?
        .ok_or_else(|| LlmProxyError::InvalidRequest(format!("api token not found: {token_id}")))?;
    crate::llm_proxy::auth::validate_token(token)
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
    require_non_empty(&request.api_token_id, "api_token_id")?;
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

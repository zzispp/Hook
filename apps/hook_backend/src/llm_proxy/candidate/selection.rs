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
        RouteScoreExplanation, RoutingMetricWindow, RoutingPreviewRequest, RoutingPreviewResponse, RoutingProfile, RoutingProfileId, RoutingRankingResponse,
        RoutingRankingsRequest,
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
    AffinitySelection,
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
    let affinity = if matches!(snapshot.scheduling_mode, types::provider::ProviderSchedulingMode::CacheAffinity) {
        state
            .cached_affinity(&token.id, &model.id, request.api_format)
            .await?
            .as_ref()
            .map(AffinitySelection::from)
    } else {
        None
    };
    let cooled_provider_ids = cooled_provider_ids(state, &snapshot).await?;
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access,
        model_id: &model.id,
        request,
        affinity: affinity.as_ref(),
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
        affinity,
        mode: snapshot.scheduling_mode,
        priority_mode: snapshot.provider_priority_mode,
    })?;
    let routed = dynamic_routing::rank_candidate_parts(dynamic_routing::DynamicRoutingInput {
        state,
        parts: ordered,
        group,
        request,
        global_model: &model,
        profile_id: effective_routing_profile_id(group, &model),
        priority_mode: snapshot.provider_priority_mode,
        allow_empty: false,
    })
    .await?;
    let candidates = proxy_candidates(ProxyCandidateBuildInput {
        state,
        token,
        request,
        global_model: &model,
        group,
        token_user,
        parts: &routed.parts,
    })
    .await?;
    Ok(CandidateSelection {
        request_id,
        cache_affinity_ttl_minutes: snapshot.cache_affinity_ttl_minutes,
        routing_profile_id: Some(routed.profile.id),
        routing_profile_version: Some(routed.profile.version),
        routing_explanations: routed.explanations,
        candidates,
    })
}

pub(super) async fn routing_rankings(state: &LlmProxyState, request: RoutingRankingsRequest) -> Result<RoutingRankingResponse, LlmProxyError> {
    let window = request.window;
    let profile = crate::llm_proxy::routing::profile_by_id(state, request.profile_id).await?.profile;
    let output = preview_output(state, PreviewInput::from_ranking(request)?, profile.clone(), window).await?;
    Ok(RoutingRankingResponse {
        profile,
        window,
        items: output,
    })
}

pub(super) async fn routing_preview(state: &LlmProxyState, request: RoutingPreviewRequest) -> Result<RoutingPreviewResponse, LlmProxyError> {
    let profile = crate::llm_proxy::routing::profile_by_id(state, request.profile_id).await?.profile;
    let items = preview_output(state, PreviewInput::from_preview(request), profile.clone(), RoutingMetricWindow::default()).await?;
    Ok(RoutingPreviewResponse { profile, items })
}

async fn preview_output(
    state: &LlmProxyState,
    input: PreviewInput,
    profile: RoutingProfile,
    window: RoutingMetricWindow,
) -> Result<Vec<RouteScoreExplanation>, LlmProxyError> {
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
    let cooled_provider_ids = cooled_provider_ids(state, &snapshot).await?;
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: &model.id,
        request,
        affinity: None,
        scheduling_mode: snapshot.scheduling_mode,
        request_id: "routing-preview",
        cooled_provider_ids: &cooled_provider_ids,
    });
    let output = dynamic_routing::rank_candidate_parts_with_profile(
        dynamic_routing::DynamicRoutingInput {
            state,
            parts,
            group,
            request,
            global_model: &model,
            profile_id: profile.id,
            priority_mode: snapshot.provider_priority_mode,
            allow_empty: true,
        },
        profile,
        window,
    )
    .await?;
    Ok(filter_explanations(output.explanations, input.include_excluded))
}

struct PreviewInput {
    group_code: String,
    model: String,
    api_format: String,
    is_stream: bool,
    include_excluded: bool,
}

impl PreviewInput {
    fn from_ranking(request: RoutingRankingsRequest) -> Result<Self, LlmProxyError> {
        Ok(Self {
            group_code: required_filter(request.group_code, "group_code")?,
            model: required_filter(request.model, "model")?,
            api_format: required_filter(request.api_format, "api_format")?,
            is_stream: request.is_stream.unwrap_or(false),
            include_excluded: request.include_excluded,
        })
    }

    fn from_preview(request: RoutingPreviewRequest) -> Self {
        Self {
            group_code: request.group_code,
            model: request.model,
            api_format: request.api_format,
            is_stream: request.is_stream,
            include_excluded: true,
        }
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

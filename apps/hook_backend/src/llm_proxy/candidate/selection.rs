mod matching;
mod proxy_candidate;
mod route;
mod scheduler;
#[cfg(test)]
mod tests;

use types::{api_token::ApiToken, model::TieredPricingConfig};
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
    let group = active_group(&snapshot, token, token_user)?;
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
    let candidates = proxy_candidates(ProxyCandidateBuildInput {
        state,
        token,
        request,
        global_model: &model,
        group,
        token_user,
        parts: &ordered,
    })
    .await?;
    Ok(CandidateSelection {
        request_id,
        cache_affinity_ttl_minutes: snapshot.cache_affinity_ttl_minutes,
        candidates,
    })
}

#[derive(Clone)]
pub(super) struct GlobalModelRef {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) default_price_per_request: Option<rust_decimal::Decimal>,
    pub(super) default_tiered_pricing: TieredPricingConfig,
}

#[derive(Clone)]
pub(super) struct CandidateParts {
    pub(super) provider: CachedProvider,
    pub(super) endpoints: Vec<CachedEndpoint>,
    pub(super) keys: Vec<CachedProviderKey>,
    pub(super) model: CachedModelBinding,
    pub(super) client_api_format: String,
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
    }
}

async fn cooled_provider_ids(state: &LlmProxyState, snapshot: &SchedulingSnapshot) -> Result<std::collections::HashSet<String>, LlmProxyError> {
    let provider_ids = snapshot.providers.iter().map(|provider| provider.id.clone()).collect::<Vec<_>>();
    state.cooled_provider_ids(&provider_ids).await
}

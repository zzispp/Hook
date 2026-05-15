mod matching;
mod proxy_candidate;
mod scheduler;
#[cfg(test)]
mod tests;

use types::{
    api_token::{ApiToken, ApiTokenType, ModelAccessMode},
    model::TieredPricingConfig,
};
use uuid::Uuid;

use self::{
    matching::{MatchingCandidatePartsInput, matching_candidate_parts},
    proxy_candidate::{ProxyCandidateBuildInput, proxy_candidates},
    scheduler::{OrderCandidatePartsInput, order_candidate_parts},
};
use super::{CandidateRequest, CandidateSelection, LlmProxyError, LlmProxyState};
use crate::llm_proxy::cache::snapshot::{
    CachedBillingGroup, CachedEndpoint, CachedGlobalModel, CachedModelBinding, CachedProvider, CachedProviderKey, CachedUserAccess, SchedulingSnapshot,
};

pub(super) const DEFAULT_MAX_RETRIES: i32 = 2;

pub(super) type CandidatePartKey = (String, String);

pub async fn select_candidates(state: &LlmProxyState, token: &ApiToken, request: CandidateRequest<'_>) -> Result<CandidateSelection, LlmProxyError> {
    let request_id = Uuid::now_v7().to_string();
    let snapshot = state.scheduling_snapshot().await?;
    let model = resolve_global_model(&snapshot, request.model_name)?;
    ensure_token_allows_model(token, &model.id)?;
    let token_user = token_user_for_snapshot(&snapshot, token)?;
    let user_access = user_access_for_token(token, token_user);
    ensure_user_allows_model(user_access, &model.id)?;
    let group = active_group(&snapshot, token)?;
    ensure_group_allows_model(group, &model.id)?;
    let affinity_key = state.cached_affinity_key(&token.id, &model.id, request.api_format).await?;
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access,
        model_id: &model.id,
        request,
        affinity_key: affinity_key.as_deref(),
        scheduling_mode: snapshot.scheduling_mode,
        request_id: &request_id,
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
        affinity_key,
        mode: snapshot.scheduling_mode,
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
    Ok(CandidateSelection { request_id, candidates })
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

fn active_group<'a>(snapshot: &'a SchedulingSnapshot, token: &ApiToken) -> Result<&'a CachedBillingGroup, LlmProxyError> {
    let group = snapshot
        .groups
        .iter()
        .find(|group| group.code == token.group_code)
        .ok_or_else(|| LlmProxyError::Forbidden(format!("billing group not found: {}", token.group_code)))?;
    if !group.is_active {
        return Err(LlmProxyError::Forbidden(format!("billing group is inactive: {}", group.code)));
    }
    Ok(group)
}

fn ensure_token_allows_model(token: &ApiToken, model_id: &str) -> Result<(), LlmProxyError> {
    if token.model_access_mode == ModelAccessMode::All || token.allowed_model_ids.iter().any(|id| id == model_id) {
        return Ok(());
    }
    Err(LlmProxyError::Forbidden(format!("model is not allowed by token: {model_id}")))
}

fn ensure_group_allows_model(group: &CachedBillingGroup, model_id: &str) -> Result<(), LlmProxyError> {
    if ids_allow(&group.allowed_model_ids, model_id) {
        return Ok(());
    }
    Err(LlmProxyError::Forbidden(format!(
        "model is not allowed by billing group {}: {model_id}",
        group.code
    )))
}

fn ensure_user_allows_model(access: Option<&CachedUserAccess>, model_id: &str) -> Result<(), LlmProxyError> {
    if access.is_none_or(|access| ids_allow(&access.allowed_model_ids, model_id)) {
        return Ok(());
    }
    Err(LlmProxyError::Forbidden(format!("model is not allowed by user: {model_id}")))
}

fn token_user_for_snapshot<'a>(snapshot: &'a SchedulingSnapshot, token: &ApiToken) -> Result<Option<&'a CachedUserAccess>, LlmProxyError> {
    let Some(user_id) = token.user_id.as_ref() else {
        if token.token_type == ApiTokenType::User {
            return Err(LlmProxyError::Forbidden(format!("user token missing user id: {}", token.id)));
        }
        return Ok(None);
    };
    let user = snapshot.users.iter().find(|user| user.id == *user_id);
    if token.token_type == ApiTokenType::User && user.is_none() {
        return Err(LlmProxyError::new_api_forbidden("user is disabled or unavailable", "new_api_error"));
    }
    if token.token_type == ApiTokenType::User && user.is_some_and(|user| !user.is_active) {
        return Err(LlmProxyError::new_api_forbidden("user is disabled or unavailable", "new_api_error"));
    }
    Ok(user)
}

fn user_access_for_token<'a>(token: &ApiToken, token_user: Option<&'a CachedUserAccess>) -> Option<&'a CachedUserAccess> {
    if token.token_type != ApiTokenType::User {
        return None;
    }
    token_user
}

fn model_ref(model: &CachedGlobalModel) -> GlobalModelRef {
    GlobalModelRef {
        id: model.id.clone(),
        name: model.name.clone(),
        default_price_per_request: model.default_price_per_request,
        default_tiered_pricing: model.default_tiered_pricing.clone(),
    }
}

fn ids_allow(ids: &[String], id: &str) -> bool {
    ids.is_empty() || ids.iter().any(|item| item == id)
}

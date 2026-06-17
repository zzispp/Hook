use proxy::scheduler::ModelAccessPolicy;
use types::{
    api_token::ApiToken,
    api_token::ModelAccessMode,
    provider::{ProviderPriorityMode, ProviderSchedulingMode, RoutingProfileId},
};

use super::{
    CandidateParts, GlobalModelRef,
    matching::{MatchingCandidatePartsInput, matching_candidate_parts},
    scheduler::{OrderCandidatePartsInput, order_candidate_parts},
};
use crate::llm_proxy::{
    AffinitySelection, LlmProxyError, LlmProxyState,
    cache::snapshot::{CachedBillingGroup, CachedUserAccess, SchedulingSnapshot},
    candidate::CandidateRequest,
    model_access::{
        active_group, ensure_group_allows_model, ensure_token_allows_model, ensure_user_allows_model, token_user_for_snapshot, user_access_for_token,
    },
};

pub(super) struct TokenRoutingContext {
    pub(super) request_id: String,
    pub(super) snapshot: SchedulingSnapshot,
    pub(super) global_model: GlobalModelRef,
    pub(super) group: CachedBillingGroup,
    pub(super) token_user: Option<CachedUserAccess>,
    pub(super) user_access: Option<CachedUserAccess>,
    pub(super) affinity: Option<AffinitySelection>,
    pub(super) cache_affinity_ttl_minutes: i64,
    pub(super) routing_profile_id: RoutingProfileId,
    pub(super) priority_mode: ProviderPriorityMode,
    pub(super) scheduling_mode: ProviderSchedulingMode,
}

pub(super) async fn token_routing_context(
    state: &LlmProxyState,
    token: &ApiToken,
    request: CandidateRequest<'_>,
    request_id: String,
) -> Result<TokenRoutingContext, LlmProxyError> {
    let snapshot = state.scheduling_snapshot().await?;
    let global_model = super::resolve_global_model(&snapshot, request.model_name)?;
    ensure_token_allows_model(token, &global_model.id)?;
    let scope = token_scope(&snapshot, token, &global_model.id)?;
    let affinity = cache_affinity(state, &snapshot, token, &global_model, &request).await?;
    Ok(TokenRoutingContext {
        request_id,
        cache_affinity_ttl_minutes: snapshot.cache_affinity_ttl_minutes,
        routing_profile_id: super::effective_routing_profile_id(&scope.group, &global_model),
        priority_mode: snapshot.provider_priority_mode,
        scheduling_mode: snapshot.scheduling_mode,
        snapshot,
        global_model,
        group: scope.group,
        token_user: scope.token_user,
        user_access: scope.user_access,
        affinity,
    })
}

impl TokenRoutingContext {
    pub(super) async fn ordered_parts(
        &self,
        state: &LlmProxyState,
        token: &ApiToken,
        request: CandidateRequest<'_>,
    ) -> Result<Vec<CandidateParts>, LlmProxyError> {
        let cooled_provider_ids = super::cooled_provider_ids(state, &self.snapshot).await?;
        let parts = matching_candidate_parts(MatchingCandidatePartsInput {
            snapshot: &self.snapshot,
            group: &self.group,
            user_access: self.user_access.as_ref(),
            model_id: &self.global_model.id,
            request: request.clone(),
            affinity: self.affinity.as_ref(),
            scheduling_mode: self.scheduling_mode,
            request_id: &self.request_id,
            cooled_provider_ids: &cooled_provider_ids,
        });
        if parts.is_empty() {
            return Err(LlmProxyError::NotFound(format!(
                "no active provider candidate for model {}",
                self.global_model.name
            )));
        }
        order_candidate_parts(OrderCandidatePartsInput {
            parts,
            group: &self.group,
            user_access: self.user_access.as_ref(),
            model_access_policy: token_model_policy(token),
            request,
            model_id: &self.global_model.id,
            request_id: &self.request_id,
            affinity: self.affinity.clone(),
            mode: self.scheduling_mode,
            priority_mode: self.priority_mode,
        })
    }
}

fn token_model_policy(token: &ApiToken) -> ModelAccessPolicy {
    match token.model_access_mode {
        ModelAccessMode::All => ModelAccessPolicy::All,
        ModelAccessMode::Limited => ModelAccessPolicy::Limited(token.allowed_model_ids.clone()),
    }
}

fn token_scope(snapshot: &SchedulingSnapshot, token: &ApiToken, model_id: &str) -> Result<TokenScope, LlmProxyError> {
    let token_user = token_user_for_snapshot(snapshot, token)?.cloned();
    let user_access = user_access_for_token(token, token_user.as_ref());
    ensure_user_allows_model(user_access, model_id)?;
    let group = active_group(snapshot, token, user_access)?.clone();
    ensure_group_allows_model(&group, model_id)?;
    let user_access = user_access.cloned();
    Ok(TokenScope {
        group,
        token_user,
        user_access,
    })
}

async fn cache_affinity(
    state: &LlmProxyState,
    snapshot: &SchedulingSnapshot,
    token: &ApiToken,
    model: &GlobalModelRef,
    request: &CandidateRequest<'_>,
) -> Result<Option<AffinitySelection>, LlmProxyError> {
    if !matches!(snapshot.scheduling_mode, ProviderSchedulingMode::CacheAffinity) {
        return Ok(None);
    }
    Ok(state
        .cached_affinity(&token.id, &model.id, request.api_format)
        .await?
        .as_ref()
        .map(AffinitySelection::from))
}

struct TokenScope {
    group: CachedBillingGroup,
    token_user: Option<CachedUserAccess>,
    user_access: Option<CachedUserAccess>,
}

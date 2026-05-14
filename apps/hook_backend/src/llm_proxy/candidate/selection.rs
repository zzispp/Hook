mod proxy_candidate;
mod scheduler;
#[cfg(test)]
mod tests;

use types::{
    api_token::{ApiToken, ApiTokenType, ModelAccessMode},
    model::TieredPricingConfig,
};
use uuid::Uuid;

use self::{proxy_candidate::proxy_candidates, scheduler::order_candidate_parts};
use super::{CandidateRequest, CandidateSelection, LlmProxyError, LlmProxyState};
use crate::llm_proxy::{
    cache::snapshot::{
        CachedBillingGroup, CachedEndpoint, CachedGlobalModel, CachedModelBinding, CachedProvider, CachedProviderKey, CachedUserAccess, SchedulingSnapshot,
    },
    formats,
};

pub(super) const DEFAULT_MAX_RETRIES: i32 = 2;
const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;

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
    let parts = matching_candidate_parts(
        &snapshot,
        group,
        user_access,
        &model.id,
        request,
        affinity_key.as_deref(),
        snapshot.scheduling_mode,
        &request_id,
    );
    if parts.is_empty() {
        return Err(LlmProxyError::NotFound(format!("no active provider candidate for model {}", model.name)));
    }

    let ordered = order_candidate_parts(
        parts,
        token,
        group,
        user_access,
        request,
        &model.id,
        &request_id,
        affinity_key,
        snapshot.scheduling_mode,
    )?;
    let candidates = proxy_candidates(state, token, request, &model, group, token_user, &ordered).await?;
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

fn matching_candidate_parts(
    snapshot: &SchedulingSnapshot,
    group: &CachedBillingGroup,
    user_access: Option<&CachedUserAccess>,
    model_id: &str,
    request: CandidateRequest<'_>,
    affinity_key: Option<&str>,
    scheduling_mode: types::provider::ProviderSchedulingMode,
    request_id: &str,
) -> Vec<CandidateParts> {
    let mut candidates = Vec::new();
    for provider in snapshot.providers.iter().filter(|provider| provider_allowed(group, user_access, provider)) {
        append_provider_candidate(provider, model_id, request, affinity_key, scheduling_mode, request_id, &mut candidates);
    }
    candidates
}

fn append_provider_candidate(
    provider: &CachedProvider,
    model_id: &str,
    request: CandidateRequest<'_>,
    affinity_key: Option<&str>,
    scheduling_mode: types::provider::ProviderSchedulingMode,
    request_id: &str,
    output: &mut Vec<CandidateParts>,
) {
    let Some(model) = provider_model(provider, model_id, affinity_key) else {
        return;
    };
    let endpoints = ordered_endpoints(provider, request);
    let keys = ordered_keys(provider, affinity_key, scheduling_mode, request_id);
    if endpoints.is_empty() || keys.is_empty() {
        return;
    }
    output.push(CandidateParts {
        provider: provider.clone(),
        endpoints,
        keys,
        model: model.clone(),
        client_api_format: request.api_format.to_owned(),
    });
}

fn ordered_endpoints(provider: &CachedProvider, request: CandidateRequest<'_>) -> Vec<CachedEndpoint> {
    let (mut exact, converted): (Vec<_>, Vec<_>) = provider
        .endpoints
        .iter()
        .filter(|endpoint| endpoint_allowed(provider, endpoint, request))
        .cloned()
        .partition(|endpoint| endpoint.api_format == request.api_format);
    exact.extend(converted);
    exact
}

fn ordered_keys(
    provider: &CachedProvider,
    affinity_key: Option<&str>,
    scheduling_mode: types::provider::ProviderSchedulingMode,
    request_id: &str,
) -> Vec<CachedProviderKey> {
    let mut keys = provider.keys.iter().filter(|key| key_allowed(key)).cloned().collect::<Vec<_>>();
    keys.sort_by(|left, right| (left.internal_priority, &left.id).cmp(&(right.internal_priority, &right.id)));
    match scheduling_mode {
        types::provider::ProviderSchedulingMode::CacheAffinity => promote_affinity_key(&mut keys, affinity_key),
        types::provider::ProviderSchedulingMode::LoadBalance => order_keys_for_load_balance(&mut keys, request_id),
        types::provider::ProviderSchedulingMode::FixedOrder => {}
    }
    keys
}

fn promote_affinity_key(keys: &mut Vec<CachedProviderKey>, affinity_key: Option<&str>) {
    let Some(key_id) = affinity_key else {
        return;
    };
    let Some(index) = keys.iter().position(|key| key.id == key_id) else {
        return;
    };
    let key = keys.remove(index);
    keys.insert(0, key);
}

fn order_keys_for_load_balance(keys: &mut [CachedProviderKey], seed: &str) {
    keys.sort_by(|left, right| {
        (left.internal_priority, stable_hash(&format!("{seed}:{}", left.id)), &left.id).cmp(&(
            right.internal_priority,
            stable_hash(&format!("{seed}:{}", right.id)),
            &right.id,
        ))
    });
}

fn provider_model(provider: &CachedProvider, model_id: &str, affinity_key: Option<&str>) -> Option<CachedModelBinding> {
    provider
        .models
        .iter()
        .find(|model| model.global_model_id == model_id && model.is_active)
        .map(|model| selected_provider_model(model, affinity_key))
}

fn endpoint_allowed(provider: &CachedProvider, endpoint: &CachedEndpoint, request: CandidateRequest<'_>) -> bool {
    endpoint.is_active && (endpoint.api_format == request.api_format || conversion_allowed(provider, endpoint, request))
}

fn conversion_allowed(provider: &CachedProvider, endpoint: &CachedEndpoint, request: CandidateRequest<'_>) -> bool {
    (provider.enable_format_conversion || endpoint_accepts_conversion(endpoint))
        && formats::formats_compatible(request.api_format, &endpoint.api_format, request.is_stream)
}

fn endpoint_accepts_conversion(endpoint: &CachedEndpoint) -> bool {
    endpoint
        .format_acceptance_config
        .as_ref()
        .and_then(|value| value.get("enabled"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

fn provider_allowed(group: &CachedBillingGroup, user_access: Option<&CachedUserAccess>, provider: &CachedProvider) -> bool {
    provider.is_active
        && ids_allow(&group.allowed_provider_ids, &provider.id)
        && user_access.is_none_or(|access| ids_allow(&access.allowed_provider_ids, &provider.id))
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
        return Err(LlmProxyError::Forbidden(format!("token user is not active: {user_id}")));
    }
    Ok(user)
}

fn user_access_for_token<'a>(token: &ApiToken, token_user: Option<&'a CachedUserAccess>) -> Option<&'a CachedUserAccess> {
    if token.token_type != ApiTokenType::User {
        return None;
    }
    token_user
}

fn key_allowed(key: &CachedProviderKey) -> bool {
    key.is_active
}

fn selected_provider_model(model: &CachedModelBinding, _affinity_key: Option<&str>) -> CachedModelBinding {
    let mut selected = model.clone();
    selected.provider_model_name = selected_provider_model_name(model);
    selected
}

fn selected_provider_model_name(model: &CachedModelBinding) -> String {
    model
        .provider_model_mapping
        .as_ref()
        .map(|mapping| mapping.name.clone())
        .unwrap_or_else(|| model.provider_model_name.clone())
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

fn stable_hash(value: &str) -> u64 {
    value
        .bytes()
        .fold(FNV_OFFSET_BASIS, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME))
}

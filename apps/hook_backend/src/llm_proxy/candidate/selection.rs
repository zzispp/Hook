mod proxy_candidate;
mod scheduler;

use types::{
    api_token::{ApiToken, ModelAccessMode},
    model::TieredPricingConfig,
};
use uuid::Uuid;

use self::{proxy_candidate::proxy_candidates, scheduler::order_candidate_parts};
use super::{CandidateRequest, CandidateSelection, LlmProxyError, LlmProxyState};
use crate::llm_proxy::{
    cache::snapshot::{CachedBillingGroup, CachedEndpoint, CachedGlobalModel, CachedModelBinding, CachedProvider, CachedProviderKey, SchedulingSnapshot},
    formats,
};

pub(super) const DEFAULT_MAX_RETRIES: i32 = 2;

pub(super) type CandidatePartKey = (String, String, String, String);

pub async fn select_candidates(state: &LlmProxyState, token: &ApiToken, request: CandidateRequest<'_>) -> Result<CandidateSelection, LlmProxyError> {
    let request_id = Uuid::now_v7().to_string();
    let snapshot = state.scheduling_snapshot().await?;
    let model = resolve_global_model(&snapshot, request.model_name)?;
    ensure_token_allows_model(token, &model.id)?;
    let group = active_group(&snapshot, token)?;
    ensure_group_allows_model(group, &model.id)?;
    let parts = matching_candidate_parts(&snapshot, group, &model.id, request);
    if parts.is_empty() {
        return Err(LlmProxyError::NotFound(format!("no active provider candidate for model {}", model.name)));
    }

    let affinity_key = state.cached_affinity_key(&token.id, &model.id, request.api_format).await?;
    let ordered = order_candidate_parts(parts, token, group, request, &model.id, &request_id, affinity_key, snapshot.scheduling_mode)?;
    let candidates = proxy_candidates(state, token, request, &model, group, &ordered).await?;
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
    pub(super) endpoint: CachedEndpoint,
    pub(super) key: CachedProviderKey,
    pub(super) model: CachedModelBinding,
    pub(super) needs_conversion: bool,
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

fn matching_candidate_parts(snapshot: &SchedulingSnapshot, group: &CachedBillingGroup, model_id: &str, request: CandidateRequest<'_>) -> Vec<CandidateParts> {
    let mut candidates = Vec::new();
    for provider in snapshot.providers.iter().filter(|provider| provider_allowed(group, provider)) {
        append_provider_candidates(provider, model_id, request, &mut candidates);
    }
    candidates
}

fn append_provider_candidates(provider: &CachedProvider, model_id: &str, request: CandidateRequest<'_>, output: &mut Vec<CandidateParts>) {
    let Some(model) = provider_model(provider, model_id) else {
        return;
    };
    for endpoint in provider.endpoints.iter().filter(|endpoint| endpoint_allowed(provider, endpoint, request)) {
        append_endpoint_candidates(provider, endpoint, &provider.keys, model, request, output);
    }
}

fn append_endpoint_candidates(
    provider: &CachedProvider,
    endpoint: &CachedEndpoint,
    keys: &[CachedProviderKey],
    model: &CachedModelBinding,
    request: CandidateRequest<'_>,
    output: &mut Vec<CandidateParts>,
) {
    for key in keys.iter().filter(|key| key_allowed(key)) {
        output.push(CandidateParts {
            provider: provider.clone(),
            endpoint: endpoint.clone(),
            key: key.clone(),
            model: model.clone(),
            needs_conversion: endpoint.api_format != request.api_format,
        });
    }
}

fn provider_model<'a>(provider: &'a CachedProvider, model_id: &str) -> Option<&'a CachedModelBinding> {
    provider.models.iter().find(|model| model.global_model_id == model_id && model.is_active)
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

fn provider_allowed(group: &CachedBillingGroup, provider: &CachedProvider) -> bool {
    provider.is_active && ids_allow(&group.allowed_provider_ids, &provider.id)
}

fn key_allowed(key: &CachedProviderKey) -> bool {
    key.is_active
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

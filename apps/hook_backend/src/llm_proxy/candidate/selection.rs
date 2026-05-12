mod proxy_candidate;
mod scheduler;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use storage::{group::GroupStore, model::global_models, provider::ProviderStore, setting::SettingStore};
use types::{
    api_token::{ApiToken, ModelAccessMode},
    group::BillingGroup,
    model::TieredPricingConfig,
    provider::{Provider, ProviderApiKey, ProviderEndpoint, ProviderModelBinding, ProviderSchedulingMode},
};
use uuid::Uuid;

use self::{proxy_candidate::proxy_candidates, scheduler::order_candidate_parts};
use super::{CandidateRequest, CandidateSelection, LlmProxyError, LlmProxyState};
use crate::llm_proxy::formats;

pub(super) const DEFAULT_MAX_RETRIES: i32 = 2;

pub(super) type CandidatePartKey = (String, String, String, String);

pub async fn select_candidates(state: &LlmProxyState, token: &ApiToken, request: CandidateRequest<'_>) -> Result<CandidateSelection, LlmProxyError> {
    let request_id = Uuid::now_v7().to_string();
    let model = resolve_global_model(state, request.model_name).await?;
    ensure_token_allows_model(token, &model.id)?;
    let group = active_group(state, token).await?;
    ensure_group_allows_model(&group, &model.id)?;

    let parts = matching_candidate_parts(state, &group, &model.id, request).await?;
    if parts.is_empty() {
        return Err(LlmProxyError::NotFound(format!("no active provider candidate for model {}", model.name)));
    }

    let affinity_key = state.cached_affinity_key(&token.id, &model.id, request.api_format).await?;
    let scheduling_mode = scheduling_mode(state).await?;
    let ordered = order_candidate_parts(parts, token, &group, request, &model.id, &request_id, affinity_key, scheduling_mode)?;
    let candidates = proxy_candidates(state, token, request, &model, &group, &ordered).await?;
    Ok(CandidateSelection { request_id, candidates })
}

#[derive(Clone)]
pub(super) struct GlobalModelRef {
    id: String,
    name: String,
    default_price_per_request: Option<rust_decimal::Decimal>,
    default_tiered_pricing: TieredPricingConfig,
}

#[derive(Clone)]
pub(super) struct CandidateParts {
    pub(super) provider: Provider,
    pub(super) endpoint: ProviderEndpoint,
    pub(super) key: ProviderApiKey,
    pub(super) model: ProviderModelBinding,
    pub(super) needs_conversion: bool,
}

async fn resolve_global_model(state: &LlmProxyState, model_name: &str) -> Result<GlobalModelRef, LlmProxyError> {
    let by_name = global_models::Entity::find()
        .filter(global_models::Column::Name.eq(model_name))
        .one(state.database.connection())
        .await?;
    let record = match by_name {
        Some(record) => Some(record),
        None => {
            global_models::Entity::find_by_id(model_name.to_owned())
                .one(state.database.connection())
                .await?
        }
    };
    let record = record.ok_or_else(|| LlmProxyError::NotFound(format!("model not found: {model_name}")))?;
    if !record.is_active {
        return Err(LlmProxyError::Forbidden(format!("model is inactive: {}", record.name)));
    }
    Ok(GlobalModelRef {
        id: record.id,
        name: record.name,
        default_price_per_request: record.default_price_per_request,
        default_tiered_pricing: serde_json::from_str(&record.default_tiered_pricing)
            .map_err(|error| LlmProxyError::Infrastructure(format!("invalid model pricing config: {error}")))?,
    })
}

async fn active_group(state: &LlmProxyState, token: &ApiToken) -> Result<BillingGroup, LlmProxyError> {
    let group = GroupStore::new(state.database.clone())
        .find_group(&token.group_code)
        .await?
        .ok_or_else(|| LlmProxyError::Forbidden(format!("billing group not found: {}", token.group_code)))?;
    if !group.is_active {
        return Err(LlmProxyError::Forbidden(format!("billing group is inactive: {}", group.code)));
    }
    Ok(group)
}

async fn scheduling_mode(state: &LlmProxyState) -> Result<ProviderSchedulingMode, LlmProxyError> {
    Ok(SettingStore::new(state.database.clone()).get_system_settings().await?.scheduling_mode)
}

fn ensure_token_allows_model(token: &ApiToken, model_id: &str) -> Result<(), LlmProxyError> {
    if token.model_access_mode == ModelAccessMode::All || token.allowed_model_ids.iter().any(|id| id == model_id) {
        return Ok(());
    }
    Err(LlmProxyError::Forbidden(format!("model is not allowed by token: {model_id}")))
}

fn ensure_group_allows_model(group: &BillingGroup, model_id: &str) -> Result<(), LlmProxyError> {
    if ids_allow(&group.allowed_model_ids, model_id) {
        return Ok(());
    }
    Err(LlmProxyError::Forbidden(format!(
        "model is not allowed by billing group {}: {model_id}",
        group.code
    )))
}

async fn matching_candidate_parts(
    state: &LlmProxyState,
    group: &BillingGroup,
    model_id: &str,
    request: CandidateRequest<'_>,
) -> Result<Vec<CandidateParts>, LlmProxyError> {
    let store = ProviderStore::new(state.database.clone());
    let providers = store.active_providers_for_scheduling().await?;
    let mut candidates = Vec::new();
    for provider in providers.into_iter().filter(|provider| provider_allowed(group, provider)) {
        append_provider_candidates(&store, provider, model_id, request, &mut candidates).await?;
    }
    Ok(candidates)
}

async fn append_provider_candidates(
    store: &ProviderStore,
    provider: Provider,
    model_id: &str,
    request: CandidateRequest<'_>,
    output: &mut Vec<CandidateParts>,
) -> Result<(), LlmProxyError> {
    let Some(model) = provider_model(store, &provider.id, model_id).await? else {
        return Ok(());
    };
    let endpoints = compatible_endpoints(store, &provider, request).await?;
    let keys = store.api_keys_for_provider(&provider.id).await?;
    for endpoint in endpoints {
        append_endpoint_candidates(&provider, &endpoint, &keys, &model, request, output);
    }
    Ok(())
}

fn append_endpoint_candidates(
    provider: &Provider,
    endpoint: &ProviderEndpoint,
    keys: &[ProviderApiKey],
    model: &ProviderModelBinding,
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

async fn provider_model(store: &ProviderStore, provider_id: &str, model_id: &str) -> Result<Option<ProviderModelBinding>, LlmProxyError> {
    let models = store.model_bindings_for_provider(provider_id).await?;
    Ok(models.into_iter().find(|model| model.global_model_id == model_id && model.is_active))
}

async fn compatible_endpoints(store: &ProviderStore, provider: &Provider, request: CandidateRequest<'_>) -> Result<Vec<ProviderEndpoint>, LlmProxyError> {
    let endpoints = store.endpoints_for_provider(&provider.id).await?;
    Ok(endpoints.into_iter().filter(|endpoint| endpoint_allowed(provider, endpoint, request)).collect())
}

fn endpoint_allowed(provider: &Provider, endpoint: &ProviderEndpoint, request: CandidateRequest<'_>) -> bool {
    endpoint.is_active && (endpoint.api_format == request.api_format || conversion_allowed(provider, endpoint, request))
}

fn conversion_allowed(provider: &Provider, endpoint: &ProviderEndpoint, request: CandidateRequest<'_>) -> bool {
    (provider.enable_format_conversion || endpoint_accepts_conversion(endpoint))
        && formats::formats_compatible(request.api_format, &endpoint.api_format, request.is_stream)
}

fn endpoint_accepts_conversion(endpoint: &ProviderEndpoint) -> bool {
    endpoint
        .format_acceptance_config
        .as_ref()
        .and_then(|value| value.get("enabled"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

fn provider_allowed(group: &BillingGroup, provider: &Provider) -> bool {
    provider.is_active && ids_allow(&group.allowed_provider_ids, &provider.id)
}

fn key_allowed(key: &ProviderApiKey) -> bool {
    key.is_active
}

fn ids_allow(ids: &[String], id: &str) -> bool {
    ids.is_empty() || ids.iter().any(|item| item == id)
}

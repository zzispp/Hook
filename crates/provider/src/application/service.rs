use async_trait::async_trait;
use types::provider::{
    ActiveRequestRecordRequest, ActiveRequestRecordResponse, Provider, ProviderApiKey, ProviderApiKeyCreate, ProviderApiKeyPriorityBatchUpdate,
    ProviderApiKeyUpdate, ProviderCooldown, ProviderCooldownListRequest, ProviderCooldownListResponse, ProviderCreate, ProviderEndpoint,
    ProviderEndpointCreate, ProviderEndpointUpdate, ProviderListRequest, ProviderListResponse, ProviderModelBinding, ProviderModelBindingBatchUpdate,
    ProviderModelBindingCreate, ProviderModelBindingUpdate, ProviderModelCostBatchUpsert, ProviderModelCostListResponse, ProviderUpdate,
    ProviderUpstreamModelsResponse, RequestRecordDetail, RequestRecordListRequest, RequestRecordListResponse, UsageRecordListResponse,
};

use crate::application::{GlobalModelCatalog, ProviderError, ProviderRepository, ProviderResult, ProviderUseCase, SecretCipher, UpstreamModelFetcher};

mod key_endpoint_scope;
mod key_permissions;
mod model_bindings;
mod model_costs;
mod request_queries;

use key_endpoint_scope::ensure_api_formats_bound;
use key_permissions::ensure_allowed_models_bound;
use model_bindings::{prepare_model_binding_batch_update, prepare_model_binding_create};
use model_costs::{ensure_model_cost_delete_scope, ensure_model_cost_scope};
use request_queries::{
    sanitize_active_request_record_request, sanitize_provider_cooldown_request, validate_provider_cooldown_request, validate_request_record_list_request,
};

use super::validation::{
    sanitize_api_key, sanitize_api_key_update, sanitize_create, sanitize_endpoint, sanitize_endpoint_update, sanitize_list_request,
    sanitize_model_binding_update, sanitize_model_cost_batch, sanitize_update, validate_api_key, validate_api_key_priority_batch, validate_api_key_update,
    validate_create, validate_endpoint, validate_endpoint_update, validate_list_request, validate_model_binding_update, validate_model_cost_batch,
    validate_update,
};

pub struct ProviderService<R, M, C, F> {
    repository: R,
    models: M,
    cipher: C,
    fetcher: F,
}

impl<R, M, C, F> ProviderService<R, M, C, F>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    F: UpstreamModelFetcher,
{
    pub const fn new(repository: R, models: M, cipher: C, fetcher: F) -> Self {
        Self {
            repository,
            models,
            cipher,
            fetcher,
        }
    }
}

#[async_trait]
impl<R, M, C, F> ProviderUseCase for ProviderService<R, M, C, F>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    F: UpstreamModelFetcher,
{
    async fn create_provider(&self, input: ProviderCreate) -> ProviderResult<Provider> {
        let input = sanitize_create(input);
        validate_create(&input)?;
        reject_duplicate_provider(&self.repository, &input.name).await?;
        self.repository.create_provider(input).await
    }

    async fn update_provider(&self, id: &str, input: ProviderUpdate) -> ProviderResult<Provider> {
        let input = sanitize_update(input);
        validate_update(&input)?;
        self.repository.update_provider(id, input).await
    }

    async fn delete_provider(&self, id: &str) -> ProviderResult<()> {
        self.repository.delete_provider(id).await
    }

    async fn get_provider(&self, id: &str) -> ProviderResult<Provider> {
        self.repository.find_provider(id).await?.ok_or(ProviderError::NotFound)
    }

    async fn list_providers(&self, request: ProviderListRequest) -> ProviderResult<ProviderListResponse> {
        let request = sanitize_list_request(request);
        validate_list_request(&request)?;
        self.repository.list_providers(request).await
    }

    async fn create_endpoint(&self, provider_id: &str, input: ProviderEndpointCreate) -> ProviderResult<ProviderEndpoint> {
        self.ensure_provider(provider_id).await?;
        let input = sanitize_endpoint(input);
        validate_endpoint(&input)?;
        self.repository.create_endpoint(provider_id, input).await
    }

    async fn update_endpoint(&self, provider_id: &str, endpoint_id: &str, input: ProviderEndpointUpdate) -> ProviderResult<ProviderEndpoint> {
        self.ensure_provider(provider_id).await?;
        let input = sanitize_endpoint_update(input);
        validate_endpoint_update(&input)?;
        self.repository.update_endpoint(provider_id, endpoint_id, input).await
    }

    async fn delete_endpoint(&self, provider_id: &str, endpoint_id: &str) -> ProviderResult<()> {
        self.ensure_provider(provider_id).await?;
        self.repository.delete_endpoint(provider_id, endpoint_id).await
    }

    async fn list_endpoints(&self, provider_id: &str) -> ProviderResult<Vec<ProviderEndpoint>> {
        self.ensure_provider(provider_id).await?;
        self.repository.list_endpoints(provider_id).await
    }

    async fn create_api_key(&self, provider_id: &str, input: ProviderApiKeyCreate) -> ProviderResult<ProviderApiKey> {
        self.ensure_provider(provider_id).await?;
        let input = sanitize_api_key(input);
        validate_api_key(&input)?;
        ensure_api_formats_bound(&self.repository, provider_id, &input.api_formats).await?;
        ensure_allowed_models_bound(&self.repository, provider_id, &input.allowed_model_ids).await?;
        let encrypted = self.cipher.encrypt_provider_key(&input.api_key)?;
        self.repository.create_api_key(provider_id, input, encrypted).await
    }

    async fn list_api_keys(&self, provider_id: &str) -> ProviderResult<Vec<ProviderApiKey>> {
        self.ensure_provider(provider_id).await?;
        self.repository.list_api_keys(provider_id).await
    }

    async fn fetch_upstream_models(&self, provider_id: &str) -> ProviderResult<ProviderUpstreamModelsResponse> {
        self.ensure_provider(provider_id).await?;
        let endpoints = active_endpoints(self.repository.list_endpoints(provider_id).await?);
        if endpoints.is_empty() {
            return Err(ProviderError::InvalidInput("provider has no active endpoint".into()));
        }
        let keys = active_api_key_secrets(self.repository.list_api_key_secrets(provider_id).await?);
        if keys.is_empty() {
            return Err(ProviderError::InvalidInput("provider has no active API key".into()));
        }

        let mut errors = Vec::new();
        for endpoint in endpoints {
            for key in keys.iter().filter(|key| key_supports_endpoint(key, &endpoint.api_format)) {
                let decrypted = self.cipher.decrypt_provider_key(&key.encrypted_api_key)?;
                match self.fetcher.fetch_upstream_models(&endpoint, &decrypted).await {
                    Ok(response) => return Ok(response),
                    Err(error) => errors.push(format!("{} / {}: {error}", endpoint.api_format, key.name)),
                }
            }
        }
        Err(ProviderError::Infrastructure(format!(
            "failed to fetch upstream models: {}",
            errors.join(" | ")
        )))
    }

    async fn update_api_key(&self, provider_id: &str, key_id: &str, input: ProviderApiKeyUpdate) -> ProviderResult<ProviderApiKey> {
        self.ensure_provider(provider_id).await?;
        let input = sanitize_api_key_update(input);
        validate_api_key_update(&input)?;
        if let Some(api_formats) = &input.api_formats {
            ensure_api_formats_bound(&self.repository, provider_id, api_formats).await?;
        }
        if let Some(allowed_model_ids) = &input.allowed_model_ids {
            ensure_allowed_models_bound(&self.repository, provider_id, allowed_model_ids).await?;
        }
        let encrypted = input.api_key.as_deref().map(|api_key| self.cipher.encrypt_provider_key(api_key)).transpose()?;
        self.repository.update_api_key(provider_id, key_id, input, encrypted).await
    }

    async fn batch_update_api_key_priorities(&self, input: ProviderApiKeyPriorityBatchUpdate) -> ProviderResult<Vec<ProviderApiKey>> {
        validate_api_key_priority_batch(&input)?;
        self.repository.batch_update_api_key_priorities(input).await
    }

    async fn delete_api_key(&self, provider_id: &str, key_id: &str) -> ProviderResult<()> {
        self.ensure_provider(provider_id).await?;
        self.repository.delete_api_key(provider_id, key_id).await
    }

    async fn create_model_binding(&self, provider_id: &str, input: ProviderModelBindingCreate) -> ProviderResult<ProviderModelBinding> {
        let input = prepare_model_binding_create(&self.repository, &self.models, provider_id, input).await?;
        self.repository.create_model_binding(provider_id, input).await
    }

    async fn batch_update_model_bindings(&self, provider_id: &str, input: ProviderModelBindingBatchUpdate) -> ProviderResult<Vec<ProviderModelBinding>> {
        let input = prepare_model_binding_batch_update(&self.repository, &self.models, provider_id, input).await?;
        self.repository.batch_update_model_bindings(provider_id, input).await
    }

    async fn list_model_bindings(&self, provider_id: &str) -> ProviderResult<Vec<ProviderModelBinding>> {
        self.ensure_provider(provider_id).await?;
        self.repository.list_model_bindings(provider_id).await
    }

    async fn update_model_binding(&self, provider_id: &str, model_id: &str, input: ProviderModelBindingUpdate) -> ProviderResult<ProviderModelBinding> {
        self.ensure_provider(provider_id).await?;
        let input = sanitize_model_binding_update(input);
        validate_model_binding_update(&input)?;
        self.repository.update_model_binding(provider_id, model_id, input).await
    }

    async fn delete_model_binding(&self, provider_id: &str, model_id: &str) -> ProviderResult<()> {
        self.ensure_provider(provider_id).await?;
        self.repository.delete_model_binding(provider_id, model_id).await
    }

    async fn list_model_costs(&self, provider_id: &str) -> ProviderResult<ProviderModelCostListResponse> {
        self.ensure_provider(provider_id).await?;
        self.repository.list_model_costs(provider_id).await
    }

    async fn upsert_model_costs(&self, provider_id: &str, key_id: &str, input: ProviderModelCostBatchUpsert) -> ProviderResult<ProviderModelCostListResponse> {
        self.ensure_provider(provider_id).await?;
        let input = sanitize_model_cost_batch(input);
        validate_model_cost_batch(&input)?;
        ensure_model_cost_scope(&self.repository, provider_id, key_id, &input).await?;
        self.repository.upsert_model_costs(provider_id, key_id, input).await
    }

    async fn delete_model_cost(&self, provider_id: &str, key_id: &str, provider_model_id: &str) -> ProviderResult<()> {
        self.ensure_provider(provider_id).await?;
        ensure_model_cost_delete_scope(&self.repository, provider_id, key_id, provider_model_id).await?;
        self.repository.delete_model_cost(provider_id, key_id, provider_model_id).await
    }

    async fn list_request_records(&self, request: RequestRecordListRequest) -> ProviderResult<RequestRecordListResponse> {
        validate_request_record_list_request(&request)?;
        self.repository.list_request_records(request).await
    }

    async fn list_usage_records(&self, user_id: &str, request: RequestRecordListRequest) -> ProviderResult<UsageRecordListResponse> {
        if user_id.trim().is_empty() {
            return Err(ProviderError::InvalidInput("user_id cannot be blank".into()));
        }
        validate_request_record_list_request(&request)?;
        self.repository.list_usage_records(user_id, request).await
    }

    async fn list_active_request_records(&self, request: ActiveRequestRecordRequest) -> ProviderResult<ActiveRequestRecordResponse> {
        let request = sanitize_active_request_record_request(request);
        self.repository.list_active_request_records(request).await
    }

    async fn get_request_record(&self, request_id: &str) -> ProviderResult<RequestRecordDetail> {
        if request_id.trim().is_empty() {
            return Err(ProviderError::InvalidInput("request_id cannot be blank".into()));
        }
        self.repository.get_request_record(request_id).await
    }

    async fn list_provider_cooldowns(&self, request: ProviderCooldownListRequest) -> ProviderResult<ProviderCooldownListResponse> {
        let request = sanitize_provider_cooldown_request(request);
        validate_provider_cooldown_request(&request)?;
        self.repository.list_provider_cooldowns(request).await
    }

    async fn release_provider_cooldown(&self, provider_id: &str) -> ProviderResult<ProviderCooldown> {
        if provider_id.trim().is_empty() {
            return Err(ProviderError::InvalidInput("provider_id cannot be blank".into()));
        }
        self.repository.release_provider_cooldown(provider_id).await
    }
}

impl<R, M, C, F> ProviderService<R, M, C, F>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    F: UpstreamModelFetcher,
{
    async fn ensure_provider(&self, provider_id: &str) -> ProviderResult<()> {
        self.repository.find_provider(provider_id).await?.ok_or(ProviderError::NotFound)?;
        Ok(())
    }
}

async fn reject_duplicate_provider<R>(repository: &R, name: &str) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    if repository.find_provider(name).await?.is_some() {
        return Err(ProviderError::Conflict(format!("provider already exists: {name}")));
    }
    Ok(())
}

fn active_endpoints(endpoints: Vec<ProviderEndpoint>) -> Vec<ProviderEndpoint> {
    endpoints.into_iter().filter(|endpoint| endpoint.is_active).collect()
}

fn active_api_key_secrets(mut keys: Vec<crate::application::ProviderApiKeySecret>) -> Vec<crate::application::ProviderApiKeySecret> {
    keys.retain(|key| key.is_active);
    keys
}

fn key_supports_endpoint(key: &crate::application::ProviderApiKeySecret, api_format: &str) -> bool {
    key.api_formats.iter().any(|format| format == api_format)
}

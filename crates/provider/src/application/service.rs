use async_trait::async_trait;
use types::provider::{
    ActiveRequestRecordRequest, ActiveRequestRecordResponse, Provider, ProviderApiKey, ProviderApiKeyCreate, ProviderApiKeyUpdate, ProviderCooldown,
    ProviderCooldownListRequest, ProviderCooldownListResponse, ProviderCreate, ProviderEndpoint, ProviderEndpointCreate, ProviderEndpointUpdate,
    ProviderListRequest, ProviderListResponse, ProviderModelBinding, ProviderModelBindingCreate, ProviderModelBindingUpdate, ProviderUpdate,
    ProviderUpstreamModelsResponse, RequestRecordDetail, RequestRecordListRequest, RequestRecordListResponse,
};

use crate::application::{GlobalModelCatalog, ProviderError, ProviderRepository, ProviderResult, ProviderUseCase, SecretCipher, UpstreamModelFetcher};

use super::validation::{
    sanitize_api_key, sanitize_api_key_update, sanitize_create, sanitize_endpoint, sanitize_endpoint_update, sanitize_list_request, sanitize_model_binding,
    sanitize_model_binding_update, sanitize_update, validate_api_key, validate_api_key_update, validate_create, validate_endpoint, validate_endpoint_update,
    validate_list_request, validate_model_binding, validate_model_binding_update, validate_update,
};

const MAX_REQUEST_RECORD_LIMIT: u64 = 100;
const MAX_PROVIDER_COOLDOWN_LIMIT: u64 = 100;

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
            for key in &keys {
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
        let encrypted = input.api_key.as_deref().map(|api_key| self.cipher.encrypt_provider_key(api_key)).transpose()?;
        self.repository.update_api_key(provider_id, key_id, input, encrypted).await
    }

    async fn delete_api_key(&self, provider_id: &str, key_id: &str) -> ProviderResult<()> {
        self.ensure_provider(provider_id).await?;
        self.repository.delete_api_key(provider_id, key_id).await
    }

    async fn create_model_binding(&self, provider_id: &str, input: ProviderModelBindingCreate) -> ProviderResult<ProviderModelBinding> {
        self.ensure_provider(provider_id).await?;
        let input = sanitize_model_binding(input);
        validate_model_binding(&input)?;
        ensure_global_model(&self.models, &input.global_model_id).await?;
        self.repository.create_model_binding(provider_id, input).await
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

    async fn list_request_records(&self, request: RequestRecordListRequest) -> ProviderResult<RequestRecordListResponse> {
        validate_request_record_list_request(&request)?;
        self.repository.list_request_records(request).await
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

async fn ensure_global_model<M>(models: &M, id: &str) -> ProviderResult<()>
where
    M: GlobalModelCatalog,
{
    if !models.global_model_exists(id).await? {
        return Err(ProviderError::InvalidInput(format!("global model does not exist: {id}")));
    }
    Ok(())
}

fn validate_request_record_list_request(request: &RequestRecordListRequest) -> ProviderResult<()> {
    if request.limit == 0 || request.limit > MAX_REQUEST_RECORD_LIMIT {
        return Err(ProviderError::InvalidInput(format!("limit must be between 1 and {MAX_REQUEST_RECORD_LIMIT}")));
    }
    if i64::try_from(request.skip).is_err() {
        return Err(ProviderError::InvalidInput("skip exceeds PostgreSQL integer range".into()));
    }
    if let Some(value) = request.type_filter.as_deref().filter(|value| !value.is_empty())
        && !matches!(value, "stream" | "non_stream")
    {
        return Err(ProviderError::InvalidInput("type must be stream or non_stream".into()));
    }
    Ok(())
}

fn sanitize_active_request_record_request(request: ActiveRequestRecordRequest) -> ActiveRequestRecordRequest {
    let mut ids = request
        .ids
        .into_iter()
        .map(|id| id.trim().to_owned())
        .filter(|id| !id.is_empty())
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    ActiveRequestRecordRequest { ids }
}

fn sanitize_provider_cooldown_request(request: ProviderCooldownListRequest) -> ProviderCooldownListRequest {
    ProviderCooldownListRequest {
        search: request.search.and_then(|value| {
            let trimmed = value.trim().to_owned();
            (!trimmed.is_empty()).then_some(trimmed)
        }),
        ..request
    }
}

fn validate_provider_cooldown_request(request: &ProviderCooldownListRequest) -> ProviderResult<()> {
    if request.limit == 0 || request.limit > MAX_PROVIDER_COOLDOWN_LIMIT {
        return Err(ProviderError::InvalidInput(format!(
            "limit must be between 1 and {MAX_PROVIDER_COOLDOWN_LIMIT}"
        )));
    }
    if request.status_code.is_some_and(|value| !(100..=599).contains(&value)) {
        return Err(ProviderError::InvalidInput("status_code must be between 100 and 599".into()));
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

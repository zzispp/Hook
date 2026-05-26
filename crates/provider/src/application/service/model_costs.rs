use std::collections::HashSet;

use types::provider::{ProviderModelBinding, ProviderModelCostBatchUpsert};

use crate::application::{ProviderError, ProviderRepository, ProviderResult};

pub async fn ensure_model_cost_scope<R>(repository: &R, provider_id: &str, key_id: &str, input: &ProviderModelCostBatchUpsert) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    ensure_key_belongs_to_provider(repository, provider_id, key_id).await?;
    ensure_models_belong_to_provider(repository, provider_id, input).await
}

pub async fn ensure_model_cost_delete_scope<R>(repository: &R, provider_id: &str, key_id: &str, provider_model_id: &str) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    ensure_key_belongs_to_provider(repository, provider_id, key_id).await?;
    ensure_model_belongs_to_provider(repository, provider_id, provider_model_id).await
}

async fn ensure_key_belongs_to_provider<R>(repository: &R, provider_id: &str, key_id: &str) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    let keys = repository.list_api_keys(provider_id).await?;
    if keys.iter().any(|key| key.id == key_id) {
        return Ok(());
    }
    Err(ProviderError::InvalidInput(format!("provider key does not belong to provider: {key_id}")))
}

async fn ensure_models_belong_to_provider<R>(repository: &R, provider_id: &str, input: &ProviderModelCostBatchUpsert) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    let bindings = repository.list_model_bindings(provider_id).await?;
    let ids = binding_ids(&bindings);
    for cost in &input.costs {
        if !ids.contains(cost.provider_model_id.as_str()) {
            return Err(ProviderError::InvalidInput(format!(
                "provider model does not belong to provider: {}",
                cost.provider_model_id
            )));
        }
    }
    Ok(())
}

async fn ensure_model_belongs_to_provider<R>(repository: &R, provider_id: &str, provider_model_id: &str) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    let bindings = repository.list_model_bindings(provider_id).await?;
    if bindings.iter().any(|binding| binding.id == provider_model_id) {
        return Ok(());
    }
    Err(ProviderError::InvalidInput(format!(
        "provider model does not belong to provider: {provider_model_id}"
    )))
}

fn binding_ids(bindings: &[ProviderModelBinding]) -> HashSet<&str> {
    bindings.iter().map(|binding| binding.id.as_str()).collect()
}

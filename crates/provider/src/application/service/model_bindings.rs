use std::collections::HashSet;

use types::provider::{ProviderModelBinding, ProviderModelBindingBatchUpdate, ProviderModelBindingCreate};

use crate::application::{GlobalModelCatalog, ProviderError, ProviderRepository, ProviderResult};

use super::super::validation::{sanitize_model_binding, sanitize_model_binding_batch, validate_model_binding, validate_model_binding_batch};

pub async fn prepare_model_binding_create<R, M>(
    repository: &R,
    models: &M,
    provider_id: &str,
    input: ProviderModelBindingCreate,
) -> ProviderResult<ProviderModelBindingCreate>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
{
    ensure_provider(repository, provider_id).await?;
    let input = sanitize_model_binding(input);
    validate_model_binding(&input)?;
    ensure_global_model(models, &input.global_model_id).await?;
    Ok(input)
}

pub async fn prepare_model_binding_batch_update<R, M>(
    repository: &R,
    models: &M,
    provider_id: &str,
    input: ProviderModelBindingBatchUpdate,
) -> ProviderResult<ProviderModelBindingBatchUpdate>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
{
    ensure_provider(repository, provider_id).await?;
    let input = sanitize_model_binding_batch(input);
    validate_model_binding_batch(&input)?;
    validate_unique_create_global_model_ids(&input.create)?;
    validate_unique_delete_ids(&input.delete_ids)?;
    ensure_global_models(models, &input.create).await?;
    ensure_batch_matches_current_bindings(repository, provider_id, &input).await?;
    Ok(input)
}

async fn ensure_provider<R>(repository: &R, provider_id: &str) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    repository.find_provider(provider_id).await?.ok_or(ProviderError::NotFound)?;
    Ok(())
}

async fn ensure_global_models<M>(models: &M, create: &[ProviderModelBindingCreate]) -> ProviderResult<()>
where
    M: GlobalModelCatalog,
{
    for id in unique_global_model_ids(create) {
        ensure_global_model(models, &id).await?;
    }
    Ok(())
}

async fn ensure_global_model<M>(models: &M, id: &str) -> ProviderResult<()>
where
    M: GlobalModelCatalog,
{
    if models.global_model_exists(id).await? {
        return Ok(());
    }
    Err(ProviderError::InvalidInput(format!("global model does not exist: {id}")))
}

async fn ensure_batch_matches_current_bindings<R>(repository: &R, provider_id: &str, input: &ProviderModelBindingBatchUpdate) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    let current = repository.list_model_bindings(provider_id).await?;
    ensure_delete_ids_exist(&input.delete_ids, &current)?;
    ensure_create_targets_available(&input.create, &input.delete_ids, &current)
}

fn validate_unique_create_global_model_ids(create: &[ProviderModelBindingCreate]) -> ProviderResult<()> {
    let mut ids = HashSet::new();
    for binding in create {
        if !ids.insert(binding.global_model_id.as_str()) {
            return Err(ProviderError::InvalidInput(format!(
                "duplicate global_model_id in batch: {}",
                binding.global_model_id
            )));
        }
    }
    Ok(())
}

fn validate_unique_delete_ids(delete_ids: &[String]) -> ProviderResult<()> {
    let mut ids = HashSet::new();
    for id in delete_ids {
        if !ids.insert(id.as_str()) {
            return Err(ProviderError::InvalidInput(format!("duplicate delete_id in batch: {id}")));
        }
    }
    Ok(())
}

fn ensure_delete_ids_exist(delete_ids: &[String], current: &[ProviderModelBinding]) -> ProviderResult<()> {
    let current_ids = current.iter().map(|binding| binding.id.as_str()).collect::<HashSet<_>>();
    for id in delete_ids {
        if !current_ids.contains(id.as_str()) {
            return Err(ProviderError::NotFound);
        }
    }
    Ok(())
}

fn ensure_create_targets_available(create: &[ProviderModelBindingCreate], delete_ids: &[String], current: &[ProviderModelBinding]) -> ProviderResult<()> {
    let existing = remaining_global_model_ids(current, delete_ids);
    for binding in create {
        if existing.contains(binding.global_model_id.as_str()) {
            return Err(ProviderError::Conflict(format!(
                "provider model binding already exists: {}",
                binding.global_model_id
            )));
        }
    }
    Ok(())
}

fn remaining_global_model_ids<'a>(current: &'a [ProviderModelBinding], delete_ids: &[String]) -> HashSet<&'a str> {
    let deleted = delete_ids.iter().map(String::as_str).collect::<HashSet<_>>();
    current
        .iter()
        .filter(|binding| !deleted.contains(binding.id.as_str()))
        .map(|binding| binding.global_model_id.as_str())
        .collect()
}

fn unique_global_model_ids(create: &[ProviderModelBindingCreate]) -> Vec<String> {
    create
        .iter()
        .map(|binding| binding.global_model_id.as_str())
        .collect::<HashSet<_>>()
        .into_iter()
        .map(str::to_owned)
        .collect()
}

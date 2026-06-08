use types::{
    model::PatchField,
    provider::{
        ProviderGroup, ProviderGroupCreate, ProviderGroupListRequest, ProviderGroupUpdate, ProviderKeyGroup, ProviderKeyGroupCreate, ProviderKeyGroupUpdate,
    },
};

use crate::application::{ProviderError, ProviderRepository, ProviderResult};

use super::super::validation::{
    sanitize_provider_group, sanitize_provider_group_list_request, sanitize_provider_group_update, sanitize_provider_key_group,
    sanitize_provider_key_group_update, validate_provider_group, validate_provider_group_list_request, validate_provider_group_update,
    validate_provider_key_group, validate_provider_key_group_update,
};

pub async fn prepare_provider_group_create<R>(repository: &R, input: ProviderGroupCreate) -> ProviderResult<ProviderGroupCreate>
where
    R: ProviderRepository,
{
    let input = sanitize_provider_group(input);
    validate_provider_group(&input)?;
    reject_duplicate_provider_group(repository, &input.name).await?;
    ensure_providers_exist(repository, &input.provider_ids).await?;
    Ok(input)
}

pub async fn prepare_provider_group_update<R>(repository: &R, id: &str, input: ProviderGroupUpdate) -> ProviderResult<ProviderGroupUpdate>
where
    R: ProviderRepository,
{
    let input = sanitize_provider_group_update(input);
    validate_provider_group_update(&input)?;
    reject_provider_group_name_conflict(repository, id, input.name.as_deref()).await?;
    ensure_patch_providers_exist(repository, &input.provider_ids).await?;
    Ok(input)
}

pub async fn prepare_provider_key_group_create<R>(repository: &R, input: ProviderKeyGroupCreate) -> ProviderResult<ProviderKeyGroupCreate>
where
    R: ProviderRepository,
{
    let input = sanitize_provider_key_group(input);
    validate_provider_key_group(&input)?;
    reject_duplicate_provider_key_group(repository, &input.name).await?;
    ensure_provider_keys_exist(repository, &input.provider_key_ids).await?;
    Ok(input)
}

pub async fn prepare_provider_key_group_update<R>(repository: &R, id: &str, input: ProviderKeyGroupUpdate) -> ProviderResult<ProviderKeyGroupUpdate>
where
    R: ProviderRepository,
{
    let input = sanitize_provider_key_group_update(input);
    validate_provider_key_group_update(&input)?;
    reject_provider_key_group_name_conflict(repository, id, input.name.as_deref()).await?;
    ensure_patch_provider_keys_exist(repository, &input.provider_key_ids).await?;
    Ok(input)
}

pub fn prepare_provider_group_list_request(input: ProviderGroupListRequest) -> ProviderResult<ProviderGroupListRequest> {
    let input = sanitize_provider_group_list_request(input);
    validate_provider_group_list_request(&input)?;
    Ok(input)
}

async fn reject_duplicate_provider_group<R>(repository: &R, name: &str) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    if repository.find_provider_group(name).await?.is_some() {
        return Err(ProviderError::Conflict(format!("provider group already exists: {name}")));
    }
    Ok(())
}

async fn reject_duplicate_provider_key_group<R>(repository: &R, name: &str) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    if repository.find_provider_key_group(name).await?.is_some() {
        return Err(ProviderError::Conflict(format!("provider key group already exists: {name}")));
    }
    Ok(())
}

async fn reject_provider_group_name_conflict<R>(repository: &R, current_id: &str, name: Option<&str>) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    let Some(name) = name else { return Ok(()) };
    reject_group_name_conflict(repository.find_provider_group(name).await?, current_id, "provider group")
}

async fn reject_provider_key_group_name_conflict<R>(repository: &R, current_id: &str, name: Option<&str>) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    let Some(name) = name else { return Ok(()) };
    reject_group_name_conflict(repository.find_provider_key_group(name).await?, current_id, "provider key group")
}

fn reject_group_name_conflict<T>(existing: Option<T>, current_id: &str, label: &str) -> ProviderResult<()>
where
    T: HasProviderGroupId,
{
    if existing.is_some_and(|group| group.group_id() != current_id) {
        return Err(ProviderError::Conflict(format!("{label} name already exists")));
    }
    Ok(())
}

trait HasProviderGroupId {
    fn group_id(&self) -> &str;
}

impl HasProviderGroupId for ProviderGroup {
    fn group_id(&self) -> &str {
        &self.id
    }
}

impl HasProviderGroupId for ProviderKeyGroup {
    fn group_id(&self) -> &str {
        &self.id
    }
}

async fn ensure_patch_providers_exist<R>(repository: &R, patch: &PatchField<Vec<String>>) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    match patch {
        PatchField::Value(value) => ensure_providers_exist(repository, value).await,
        PatchField::Null | PatchField::Missing => Ok(()),
    }
}

async fn ensure_patch_provider_keys_exist<R>(repository: &R, patch: &PatchField<Vec<String>>) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    match patch {
        PatchField::Value(value) => ensure_provider_keys_exist(repository, value).await,
        PatchField::Null | PatchField::Missing => Ok(()),
    }
}

async fn ensure_providers_exist<R>(repository: &R, ids: &[String]) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    for id in ids {
        if repository.find_provider(id).await?.is_none() {
            return Err(ProviderError::InvalidInput(format!("provider does not exist: {id}")));
        }
    }
    Ok(())
}

async fn ensure_provider_keys_exist<R>(repository: &R, ids: &[String]) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    for id in ids {
        if !repository.provider_key_exists(id).await? {
            return Err(ProviderError::InvalidInput(format!("provider key does not exist: {id}")));
        }
    }
    Ok(())
}

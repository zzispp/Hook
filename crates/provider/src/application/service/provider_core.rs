use types::provider::{ProviderCreate, ProviderListRequest};

use crate::application::{ProviderError, ProviderRepository, ProviderResult};

use super::super::validation::{sanitize_create, sanitize_list_request, validate_create, validate_list_request};

pub async fn prepare_provider_create<R>(repository: &R, input: ProviderCreate) -> ProviderResult<ProviderCreate>
where
    R: ProviderRepository,
{
    let input = sanitize_create(input);
    validate_create(&input)?;
    reject_duplicate_provider(repository, &input.name).await?;
    ensure_provider_group(repository, input.provider_group_id.as_deref()).await?;
    Ok(input)
}

pub fn prepare_provider_list_request(input: ProviderListRequest) -> ProviderResult<ProviderListRequest> {
    let input = sanitize_list_request(input);
    validate_list_request(&input)?;
    Ok(input)
}

pub async fn ensure_provider<R>(repository: &R, provider_id: &str) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    repository.find_provider(provider_id).await?.ok_or(ProviderError::NotFound)?;
    Ok(())
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

async fn ensure_provider_group<R>(repository: &R, group_id: Option<&str>) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    let Some(group_id) = group_id else { return Ok(()) };
    match repository.find_provider_group(group_id).await? {
        Some(group) if group.id == group_id => Ok(()),
        _ => Err(ProviderError::InvalidInput(format!("provider group does not exist: {group_id}"))),
    }
}

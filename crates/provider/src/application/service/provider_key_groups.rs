use types::{
    model::PatchField,
    provider::{ProviderKeyGroupCreate, ProviderKeyGroupListRequest, ProviderKeyGroupMemberInput, ProviderKeyGroupUpdate},
};

use crate::application::{ProviderError, ProviderRepository, ProviderResult};

use super::super::validation::{
    sanitize_provider_key_group, sanitize_provider_key_group_list_request, sanitize_provider_key_group_update, validate_provider_key_group,
    validate_provider_key_group_list_request, validate_provider_key_group_update,
};

pub async fn prepare_provider_key_group_create<R>(repository: &R, input: ProviderKeyGroupCreate) -> ProviderResult<ProviderKeyGroupCreate>
where
    R: ProviderRepository,
{
    let input = sanitize_provider_key_group(input);
    validate_provider_key_group(&input)?;
    reject_duplicate_provider_key_group(repository, &input.name).await?;
    ensure_provider_key_members_exist(repository, &input.provider_key_members).await?;
    Ok(input)
}

pub async fn prepare_provider_key_group_update<R>(repository: &R, id: &str, input: ProviderKeyGroupUpdate) -> ProviderResult<ProviderKeyGroupUpdate>
where
    R: ProviderRepository,
{
    let input = sanitize_provider_key_group_update(input);
    validate_provider_key_group_update(&input)?;
    reject_provider_key_group_name_conflict(repository, id, input.name.as_deref()).await?;
    ensure_patch_provider_key_members_exist(repository, &input.provider_key_members).await?;
    Ok(input)
}

pub fn prepare_provider_key_group_list_request(input: ProviderKeyGroupListRequest) -> ProviderResult<ProviderKeyGroupListRequest> {
    let input = sanitize_provider_key_group_list_request(input);
    validate_provider_key_group_list_request(&input)?;
    Ok(input)
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

async fn reject_provider_key_group_name_conflict<R>(repository: &R, current_id: &str, name: Option<&str>) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    let Some(name) = name else { return Ok(()) };
    let existing = repository.find_provider_key_group(name).await?;
    if existing.is_some_and(|group| group.id != current_id) {
        return Err(ProviderError::Conflict("provider key group name already exists".into()));
    }
    Ok(())
}

async fn ensure_patch_provider_key_members_exist<R>(repository: &R, patch: &PatchField<Vec<ProviderKeyGroupMemberInput>>) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    match patch {
        PatchField::Value(value) => ensure_provider_key_members_exist(repository, value).await,
        PatchField::Null | PatchField::Missing => Ok(()),
    }
}

async fn ensure_provider_key_members_exist<R>(repository: &R, members: &[ProviderKeyGroupMemberInput]) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    for member in members {
        if !repository.provider_key_exists(&member.provider_key_id).await? {
            return Err(ProviderError::InvalidInput(format!("provider key does not exist: {}", member.provider_key_id)));
        }
    }
    Ok(())
}

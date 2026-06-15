use async_trait::async_trait;
use types::group::{BillingGroupCreate, BillingGroupListRequest, BillingGroupListResponse, BillingGroupResponse, BillingGroupUpdate};

use crate::application::{GroupError, GroupModelCatalog, GroupProviderCatalog, GroupRepository, GroupResult, GroupUseCase, GroupUserGroupCatalog};

use super::validation::{sanitize_create, sanitize_update, validate_create, validate_list_request, validate_update};

pub struct GroupService<R, M, P, U> {
    repository: R,
    models: M,
    providers: P,
    user_groups: U,
}

impl<R, M, P, U> GroupService<R, M, P, U>
where
    R: GroupRepository,
    M: GroupModelCatalog,
    P: GroupProviderCatalog,
    U: GroupUserGroupCatalog,
{
    pub const fn new(repository: R, models: M, providers: P, user_groups: U) -> Self {
        Self {
            repository,
            models,
            providers,
            user_groups,
        }
    }
}

#[async_trait]
impl<R, M, P, U> GroupUseCase for GroupService<R, M, P, U>
where
    R: GroupRepository,
    M: GroupModelCatalog,
    P: GroupProviderCatalog,
    U: GroupUserGroupCatalog,
{
    async fn create_group(&self, input: BillingGroupCreate) -> GroupResult<BillingGroupResponse> {
        let input = sanitize_create(input);
        validate_create(&input)?;
        ensure_models_exist(&self.models, &input.allowed_model_ids).await?;
        ensure_provider_key_groups_exist(&self.providers, &input.allowed_provider_key_group_ids).await?;
        ensure_user_groups_exist(&self.user_groups, &input.visible_user_group_codes).await?;
        reject_duplicate_code(&self.repository, &input.code).await?;
        self.repository.create_group(input).await
    }

    async fn update_group(&self, id: &str, input: BillingGroupUpdate) -> GroupResult<BillingGroupResponse> {
        let input = sanitize_update(input);
        validate_update(&input)?;
        ensure_patch_models_exist(&self.models, &input.allowed_model_ids).await?;
        ensure_patch_provider_key_groups_exist(&self.providers, &input.allowed_provider_key_group_ids).await?;
        ensure_patch_user_groups_exist(&self.user_groups, &input.visible_user_group_codes).await?;
        self.repository.update_group(id, input).await
    }

    async fn delete_group(&self, id: &str) -> GroupResult<()> {
        let group = self.get_group(id).await?;
        reject_system_group(&group)?;
        reject_group_with_tokens(&self.repository, &group.code).await?;
        self.repository.delete_group(&group.id).await
    }

    async fn get_group(&self, id: &str) -> GroupResult<BillingGroupResponse> {
        self.repository.find_group(id).await?.ok_or(GroupError::NotFound)
    }

    async fn list_groups(&self, request: BillingGroupListRequest) -> GroupResult<BillingGroupListResponse> {
        validate_list_request(&request)?;
        self.repository.list_groups(request).await
    }

    async fn available_groups(&self, user_group_codes: &[String]) -> GroupResult<Vec<BillingGroupResponse>> {
        let active_user_group_codes = active_user_group_codes(&self.user_groups, user_group_codes).await?;
        if active_user_group_codes.is_empty() {
            return Ok(Vec::new());
        }
        self.repository
            .active_groups_for_user_groups(&active_user_group_codes)
            .await
            .map(sanitize_available_groups)
    }
}

async fn active_user_group_codes<U>(user_groups: &U, codes: &[String]) -> GroupResult<Vec<String>>
where
    U: GroupUserGroupCatalog,
{
    let mut active = Vec::new();
    for code in codes {
        if user_groups.active_user_group_exists(code).await? {
            active.push(code.clone());
        }
    }
    Ok(active)
}

fn sanitize_available_groups(groups: Vec<BillingGroupResponse>) -> Vec<BillingGroupResponse> {
    groups.into_iter().map(sanitize_available_group).collect()
}

fn sanitize_available_group(mut group: BillingGroupResponse) -> BillingGroupResponse {
    group.allowed_provider_key_group_ids.clear();
    group
}

async fn reject_duplicate_code<R>(repository: &R, code: &str) -> GroupResult<()>
where
    R: GroupRepository,
{
    if repository.find_group(code).await?.is_some() {
        return Err(GroupError::Conflict(format!("billing group already exists: {code}")));
    }
    Ok(())
}

async fn ensure_patch_models_exist<M>(models: &M, patch: &types::model::PatchField<Vec<String>>) -> GroupResult<()>
where
    M: GroupModelCatalog,
{
    match patch {
        types::model::PatchField::Value(value) => ensure_models_exist(models, value).await,
        types::model::PatchField::Null | types::model::PatchField::Missing => Ok(()),
    }
}

async fn ensure_models_exist<M>(models: &M, ids: &[String]) -> GroupResult<()>
where
    M: GroupModelCatalog,
{
    for id in ids {
        if !models.model_exists(id).await? {
            return Err(GroupError::InvalidInput(format!("global model does not exist: {id}")));
        }
    }
    Ok(())
}

async fn ensure_patch_provider_key_groups_exist<P>(providers: &P, patch: &types::model::PatchField<Vec<String>>) -> GroupResult<()>
where
    P: GroupProviderCatalog,
{
    match patch {
        types::model::PatchField::Value(value) => ensure_provider_key_groups_exist(providers, value).await,
        types::model::PatchField::Null | types::model::PatchField::Missing => Ok(()),
    }
}

async fn ensure_provider_key_groups_exist<P>(providers: &P, ids: &[String]) -> GroupResult<()>
where
    P: GroupProviderCatalog,
{
    for id in ids {
        if !providers.provider_key_group_exists(id).await? {
            return Err(GroupError::InvalidInput(format!("provider key group does not exist: {id}")));
        }
    }
    Ok(())
}

async fn ensure_patch_user_groups_exist<U>(user_groups: &U, patch: &types::model::PatchField<Vec<String>>) -> GroupResult<()>
where
    U: GroupUserGroupCatalog,
{
    match patch {
        types::model::PatchField::Value(value) => ensure_user_groups_exist(user_groups, value).await,
        types::model::PatchField::Null | types::model::PatchField::Missing => Ok(()),
    }
}

async fn ensure_user_groups_exist<U>(user_groups: &U, codes: &[String]) -> GroupResult<()>
where
    U: GroupUserGroupCatalog,
{
    for code in codes {
        if !user_groups.active_user_group_exists(code).await? {
            return Err(GroupError::InvalidInput(format!("active user group does not exist: {code}")));
        }
    }
    Ok(())
}

fn reject_system_group(group: &BillingGroupResponse) -> GroupResult<()> {
    if group.is_system {
        return Err(GroupError::Conflict("system billing group cannot be deleted".into()));
    }
    Ok(())
}

async fn reject_group_with_tokens<R>(repository: &R, code: &str) -> GroupResult<()>
where
    R: GroupRepository,
{
    if repository.group_has_tokens(code).await? {
        return Err(GroupError::Conflict("billing group is bound to API tokens".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use types::group::BillingGroupResponse;

    use super::sanitize_available_group;

    #[test]
    fn sanitize_available_group_hides_key_group_bindings() {
        let group = BillingGroupResponse {
            id: "group-1".into(),
            code: "default".into(),
            name: "Default".into(),
            description: Some("default group".into()),
            billing_multiplier: Decimal::ONE,
            allowed_model_ids: vec!["model-1".into()],
            allowed_provider_key_group_ids: vec!["key-group-1".into(), "key-group-2".into()],
            routing_profile_id: None,
            visible_user_group_codes: vec!["default".into()],
            is_active: true,
            is_system: true,
            sort_order: 0,
            created_at: "2026-05-13T00:00:00Z".into(),
            updated_at: "2026-05-13T00:00:00Z".into(),
        };

        let sanitized = sanitize_available_group(group);

        assert!(sanitized.allowed_provider_key_group_ids.is_empty());
        assert_eq!(sanitized.allowed_model_ids, vec!["model-1".to_string()]);
    }
}

use async_trait::async_trait;
use types::group::{BillingGroupCreate, BillingGroupListRequest, BillingGroupListResponse, BillingGroupResponse, BillingGroupUpdate};

use crate::application::{GroupError, GroupModelCatalog, GroupRepository, GroupResult, GroupUseCase};

use super::validation::{sanitize_create, sanitize_update, validate_create, validate_list_request, validate_update};

pub struct GroupService<R, M> {
    repository: R,
    models: M,
}

impl<R, M> GroupService<R, M>
where
    R: GroupRepository,
    M: GroupModelCatalog,
{
    pub const fn new(repository: R, models: M) -> Self {
        Self { repository, models }
    }
}

#[async_trait]
impl<R, M> GroupUseCase for GroupService<R, M>
where
    R: GroupRepository,
    M: GroupModelCatalog,
{
    async fn create_group(&self, input: BillingGroupCreate) -> GroupResult<BillingGroupResponse> {
        let input = sanitize_create(input);
        validate_create(&input)?;
        ensure_models_exist(&self.models, &input.allowed_model_ids).await?;
        reject_duplicate_code(&self.repository, &input.code).await?;
        self.repository.create_group(input).await
    }

    async fn update_group(&self, id: &str, input: BillingGroupUpdate) -> GroupResult<BillingGroupResponse> {
        let input = sanitize_update(input);
        validate_update(&input)?;
        ensure_patch_models_exist(&self.models, &input.allowed_model_ids).await?;
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

    async fn available_groups(&self) -> GroupResult<Vec<BillingGroupResponse>> {
        self.repository.active_groups().await
    }
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

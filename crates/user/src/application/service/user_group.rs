use async_trait::async_trait;
use constants::{
    pagination::{MAX_PAGE_SIZE, MIN_PAGE_NUMBER, MIN_PAGE_SIZE},
    user_group::DEFAULT_USER_GROUP_CODE,
};
use types::{
    pagination::{Page, PageRequest},
    user::{User, UserListFilters},
    user_group::{UserGroupCreate, UserGroupListRequest, UserGroupPageResponse, UserGroupResponse, UserGroupUpdate},
};

use crate::application::{
    AppError, AppResult, UserGroupBillingCatalog, UserGroupCreateRecord, UserGroupRepository, UserGroupSettingCatalog, UserGroupUpdateRecord, UserGroupUseCase,
};

const MAX_CODE_LENGTH: usize = 64;
const MAX_NAME_LENGTH: usize = 100;
const MAX_DESCRIPTION_LENGTH: usize = 500;

pub struct UserGroupService<R, B, S> {
    repository: R,
    billing: B,
    settings: S,
}

impl<R, B, S> UserGroupService<R, B, S>
where
    R: UserGroupRepository,
    B: UserGroupBillingCatalog,
    S: UserGroupSettingCatalog,
{
    pub const fn new(repository: R, billing: B, settings: S) -> Self {
        Self { repository, billing, settings }
    }
}

#[async_trait]
impl<R, B, S> UserGroupUseCase for UserGroupService<R, B, S>
where
    R: UserGroupRepository,
    B: UserGroupBillingCatalog,
    S: UserGroupSettingCatalog,
{
    async fn create_user_group(&self, input: UserGroupCreate) -> AppResult<UserGroupResponse> {
        let input = sanitize_create(input);
        validate_create(&input)?;
        reject_duplicate_code(&self.repository, &input.code).await?;
        self.repository.create_group(create_record(input)).await
    }

    async fn update_user_group(&self, code: &str, input: UserGroupUpdate) -> AppResult<UserGroupResponse> {
        let input = sanitize_update(input);
        validate_update(&input)?;
        let group = self.get_user_group(code).await?;
        reject_default_disable(code, &input)?;
        reject_default_setting_disable(&self.settings, &group, &input).await?;
        self.repository.update_group(code, update_record(input)).await
    }

    async fn delete_user_group(&self, code: &str) -> AppResult<()> {
        reject_default_code(code, "default user group cannot be deleted")?;
        self.get_user_group(code).await?;
        reject_group_with_users(&self.repository, code).await?;
        reject_group_with_billing_groups(&self.billing, code).await?;
        self.repository.delete_group(code).await
    }

    async fn get_user_group(&self, code: &str) -> AppResult<UserGroupResponse> {
        self.repository.find_group(code).await?.ok_or(AppError::NotFound)
    }

    async fn list_user_groups(&self, request: UserGroupListRequest) -> AppResult<UserGroupPageResponse> {
        validate_page(request.page)?;
        self.repository.list_groups(request).await
    }

    async fn list_user_group_members(&self, code: &str, request: PageRequest, filters: UserListFilters) -> AppResult<Page<User>> {
        validate_page(request)?;
        self.get_user_group(code).await?;
        self.repository.list_group_users(request, member_filters(code, filters)).await
    }
}

fn sanitize_create(input: UserGroupCreate) -> UserGroupCreate {
    UserGroupCreate {
        code: input.code.trim().to_owned(),
        name: input.name.trim().to_owned(),
        description: input.description.and_then(trim_optional),
        ..input
    }
}

fn sanitize_update(input: UserGroupUpdate) -> UserGroupUpdate {
    UserGroupUpdate {
        name: input.name.map(|value| value.trim().to_owned()),
        description: input.description.and_then(trim_optional),
        ..input
    }
}

fn validate_create(input: &UserGroupCreate) -> AppResult<()> {
    validate_code(&input.code)?;
    validate_name(input.name.as_str())?;
    validate_description(input.description.as_deref())
}

fn validate_update(input: &UserGroupUpdate) -> AppResult<()> {
    if input.name.is_none() && input.description.is_none() && input.is_active.is_none() && input.sort_order.is_none() {
        return Err(AppError::InvalidInput("update payload is empty".into()));
    }
    if let Some(name) = input.name.as_deref() {
        validate_name(name)?;
    }
    validate_description(input.description.as_deref())
}

fn validate_code(value: &str) -> AppResult<()> {
    validate_name(value)?;
    if value.len() > MAX_CODE_LENGTH {
        return Err(AppError::InvalidInput(format!("code length must be between 1 and {MAX_CODE_LENGTH}")));
    }
    if !value.chars().all(|item| item.is_ascii_alphanumeric() || item == '_' || item == '-') {
        return Err(AppError::InvalidInput(
            "code can only contain letters, numbers, underscores, and hyphens".into(),
        ));
    }
    Ok(())
}

fn validate_name(value: &str) -> AppResult<()> {
    if value.is_empty() || value.len() > MAX_NAME_LENGTH {
        return Err(AppError::InvalidInput(format!("name length must be between 1 and {MAX_NAME_LENGTH}")));
    }
    Ok(())
}

fn validate_description(value: Option<&str>) -> AppResult<()> {
    if value.is_some_and(|text| text.len() > MAX_DESCRIPTION_LENGTH) {
        return Err(AppError::InvalidInput(format!("description length must be at most {MAX_DESCRIPTION_LENGTH}")));
    }
    Ok(())
}

fn validate_page(page: PageRequest) -> AppResult<()> {
    if page.page < MIN_PAGE_NUMBER {
        return Err(AppError::InvalidInput("page must be greater than 0".into()));
    }
    if page.page_size < MIN_PAGE_SIZE || page.page_size > MAX_PAGE_SIZE {
        return Err(AppError::InvalidInput(format!("page_size must be between {MIN_PAGE_SIZE} and {MAX_PAGE_SIZE}")));
    }
    Ok(())
}

async fn reject_duplicate_code<R>(repository: &R, code: &str) -> AppResult<()>
where
    R: UserGroupRepository,
{
    if repository.find_group(code).await?.is_some() {
        return Err(AppError::Conflict(format!("user group already exists: {code}")));
    }
    Ok(())
}

fn reject_default_disable(code: &str, input: &UserGroupUpdate) -> AppResult<()> {
    if code == DEFAULT_USER_GROUP_CODE && input.is_active == Some(false) {
        return Err(AppError::Conflict("default user group cannot be disabled".into()));
    }
    Ok(())
}

async fn reject_default_setting_disable<S>(settings: &S, group: &UserGroupResponse, input: &UserGroupUpdate) -> AppResult<()>
where
    S: UserGroupSettingCatalog,
{
    if group.is_active && input.is_active == Some(false) && settings.default_user_group_code().await? == group.code {
        return Err(AppError::Conflict("default registration user group cannot be disabled".into()));
    }
    Ok(())
}

fn reject_default_code(code: &str, message: &str) -> AppResult<()> {
    if code == DEFAULT_USER_GROUP_CODE {
        return Err(AppError::Conflict(message.into()));
    }
    Ok(())
}

async fn reject_group_with_users<R>(repository: &R, code: &str) -> AppResult<()>
where
    R: UserGroupRepository,
{
    if repository.group_has_users(code).await? {
        return Err(AppError::Conflict("user group has users".into()));
    }
    Ok(())
}

async fn reject_group_with_billing_groups<B>(billing: &B, code: &str) -> AppResult<()>
where
    B: UserGroupBillingCatalog,
{
    if billing.user_group_has_billing_groups(code).await? {
        return Err(AppError::Conflict("user group is bound to billing groups".into()));
    }
    Ok(())
}

fn create_record(input: UserGroupCreate) -> UserGroupCreateRecord {
    UserGroupCreateRecord {
        code: input.code,
        name: input.name,
        description: input.description,
        is_active: input.is_active.unwrap_or(true),
        is_system: false,
        sort_order: input.sort_order.unwrap_or(0),
    }
}

fn update_record(input: UserGroupUpdate) -> UserGroupUpdateRecord {
    UserGroupUpdateRecord {
        name: input.name,
        description: input.description,
        is_active: input.is_active,
        sort_order: input.sort_order,
    }
}

fn member_filters(code: &str, filters: UserListFilters) -> UserListFilters {
    UserListFilters {
        group_code: Some(code.to_owned()),
        ..filters
    }
}

fn trim_optional(value: String) -> Option<String> {
    let value = value.trim().to_owned();
    if value.is_empty() { None } else { Some(value) }
}

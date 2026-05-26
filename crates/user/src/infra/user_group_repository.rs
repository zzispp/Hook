use async_trait::async_trait;
use storage::{
    Database, StorageError,
    group::GroupStore,
    setting::SettingStore,
    user::{UserGroupRecordInput, UserGroupRecordPatch, UserGroupStore, UserStore},
};
use types::{
    pagination::{Page, PageRequest},
    user::{User, UserListFilters},
    user_group::{UserGroupListRequest, UserGroupPageResponse, UserGroupResponse},
};

use crate::application::{
    AppError, AppResult, UserGroupBillingCatalog, UserGroupCreateRecord, UserGroupRepository, UserGroupSettingCatalog, UserGroupUpdateRecord,
};

#[derive(Clone)]
pub struct StorageUserGroupRepository {
    groups: UserGroupStore,
    users: UserStore,
}

#[derive(Clone)]
pub struct StorageUserGroupBillingCatalog {
    groups: GroupStore,
}

#[derive(Clone)]
pub struct StorageUserGroupSettingCatalog {
    settings: SettingStore,
}

impl StorageUserGroupRepository {
    pub fn new(database: Database) -> Self {
        Self {
            groups: UserGroupStore::new(database.clone()),
            users: UserStore::new(database),
        }
    }
}

impl StorageUserGroupBillingCatalog {
    pub fn new(database: Database) -> Self {
        Self {
            groups: GroupStore::new(database),
        }
    }
}

impl StorageUserGroupSettingCatalog {
    pub fn new(database: Database) -> Self {
        Self {
            settings: SettingStore::new(database),
        }
    }
}

#[async_trait]
impl UserGroupRepository for StorageUserGroupRepository {
    async fn create_group(&self, input: UserGroupCreateRecord) -> AppResult<UserGroupResponse> {
        self.groups.create_group(record_input(input)).await.map(Into::into).map_err(storage_error)
    }

    async fn update_group(&self, code: &str, input: UserGroupUpdateRecord) -> AppResult<UserGroupResponse> {
        self.groups.update_group(code, record_patch(input)).await.map(Into::into).map_err(storage_error)
    }

    async fn delete_group(&self, code: &str) -> AppResult<()> {
        self.groups.delete_group(code).await.map_err(storage_error)
    }

    async fn find_group(&self, code: &str) -> AppResult<Option<UserGroupResponse>> {
        self.groups.find_group(code).await.map(|group| group.map(Into::into)).map_err(storage_error)
    }

    async fn list_groups(&self, request: UserGroupListRequest) -> AppResult<UserGroupPageResponse> {
        self.groups.list_groups(request).await.map_err(storage_error)
    }

    async fn group_has_users(&self, code: &str) -> AppResult<bool> {
        self.groups.group_has_users(code).await.map_err(storage_error)
    }

    async fn list_group_users(&self, request: PageRequest, filters: UserListFilters) -> AppResult<Page<User>> {
        self.users.list(request, filters).await.map_err(storage_error)
    }
}

#[async_trait]
impl UserGroupBillingCatalog for StorageUserGroupBillingCatalog {
    async fn user_group_has_billing_groups(&self, code: &str) -> AppResult<bool> {
        self.groups.user_group_has_billing_groups(code).await.map_err(storage_error)
    }
}

#[async_trait]
impl UserGroupSettingCatalog for StorageUserGroupSettingCatalog {
    async fn default_user_group_code(&self) -> AppResult<String> {
        self.settings
            .get_system_settings()
            .await
            .map(|settings| settings.default_user_group_code)
            .map_err(storage_error)
    }
}

fn record_input(input: UserGroupCreateRecord) -> UserGroupRecordInput {
    UserGroupRecordInput {
        code: input.code,
        name: input.name,
        description: input.description,
        is_active: input.is_active,
        is_system: input.is_system,
        sort_order: input.sort_order,
    }
}

fn record_patch(input: UserGroupUpdateRecord) -> UserGroupRecordPatch {
    UserGroupRecordPatch {
        name: input.name,
        description: input.description,
        is_active: input.is_active,
        sort_order: input.sort_order,
    }
}

fn storage_error(error: StorageError) -> AppError {
    match error {
        StorageError::NotFound => AppError::NotFound,
        StorageError::Conflict(message) => AppError::Conflict(message),
        StorageError::Database(message) => AppError::Infrastructure(message),
    }
}

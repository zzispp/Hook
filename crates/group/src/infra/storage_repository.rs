use async_trait::async_trait;
use storage::{
    Database, StorageError,
    group::{BillingGroupRecordInput, BillingGroupRecordPatch, GroupStore},
    model::ModelStore,
    provider::ProviderStore,
    user::UserGroupStore,
};
use types::group::{BillingGroupCreate, BillingGroupListRequest, BillingGroupListResponse, BillingGroupResponse, BillingGroupUpdate};

use crate::application::{GroupError, GroupModelCatalog, GroupProviderCatalog, GroupRepository, GroupResult, GroupUserGroupCatalog};

#[derive(Clone)]
pub struct StorageGroupRepository {
    store: GroupStore,
}

#[derive(Clone)]
pub struct StorageGroupModelCatalog {
    store: ModelStore,
}

#[derive(Clone)]
pub struct StorageGroupProviderCatalog {
    store: ProviderStore,
}

#[derive(Clone)]
pub struct StorageGroupUserGroupCatalog {
    store: UserGroupStore,
}

impl StorageGroupRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: GroupStore::new(database),
        }
    }
}

impl StorageGroupModelCatalog {
    pub fn new(database: Database) -> Self {
        Self {
            store: ModelStore::new(database),
        }
    }
}

impl StorageGroupProviderCatalog {
    pub fn new(database: Database) -> Self {
        Self {
            store: ProviderStore::new(database),
        }
    }
}

impl StorageGroupUserGroupCatalog {
    pub fn new(database: Database) -> Self {
        Self {
            store: UserGroupStore::new(database),
        }
    }
}

#[async_trait]
impl GroupRepository for StorageGroupRepository {
    async fn create_group(&self, input: BillingGroupCreate) -> GroupResult<BillingGroupResponse> {
        self.store.create_group(record_input(input, false)).await.map(Into::into).map_err(storage_error)
    }

    async fn update_group(&self, id: &str, input: BillingGroupUpdate) -> GroupResult<BillingGroupResponse> {
        self.store.update_group(id, record_patch(input)).await.map(Into::into).map_err(storage_error)
    }

    async fn delete_group(&self, id: &str) -> GroupResult<()> {
        self.store.delete_group(id).await.map_err(storage_error)
    }

    async fn find_group(&self, id_or_code: &str) -> GroupResult<Option<BillingGroupResponse>> {
        self.store
            .find_group(id_or_code)
            .await
            .map(|group| group.map(Into::into))
            .map_err(storage_error)
    }

    async fn list_groups(&self, request: BillingGroupListRequest) -> GroupResult<BillingGroupListResponse> {
        self.store.list_groups(request).await.map_err(storage_error)
    }

    async fn active_groups(&self) -> GroupResult<Vec<BillingGroupResponse>> {
        self.store
            .active_groups()
            .await
            .map(|groups| groups.into_iter().map(Into::into).collect())
            .map_err(storage_error)
    }

    async fn active_groups_for_user_groups(&self, user_group_codes: &[String]) -> GroupResult<Vec<BillingGroupResponse>> {
        self.store
            .active_groups_for_user_groups(user_group_codes)
            .await
            .map(|groups| groups.into_iter().map(Into::into).collect())
            .map_err(storage_error)
    }

    async fn group_has_tokens(&self, code: &str) -> GroupResult<bool> {
        self.store.group_has_tokens(code).await.map_err(storage_error)
    }

    async fn user_group_has_billing_groups(&self, user_group_code: &str) -> GroupResult<bool> {
        self.store.user_group_has_billing_groups(user_group_code).await.map_err(storage_error)
    }
}

#[async_trait]
impl GroupModelCatalog for StorageGroupModelCatalog {
    async fn model_exists(&self, id: &str) -> GroupResult<bool> {
        self.store.get_global_model(id).await.map(|model| model.is_some()).map_err(storage_error)
    }
}

#[async_trait]
impl GroupProviderCatalog for StorageGroupProviderCatalog {
    async fn provider_group_exists(&self, id: &str) -> GroupResult<bool> {
        self.store.find_provider_group(id).await.map(|group| group.is_some()).map_err(storage_error)
    }

    async fn provider_key_group_exists(&self, id: &str) -> GroupResult<bool> {
        self.store.find_provider_key_group(id).await.map(|group| group.is_some()).map_err(storage_error)
    }
}

#[async_trait]
impl GroupUserGroupCatalog for StorageGroupUserGroupCatalog {
    async fn active_user_group_exists(&self, code: &str) -> GroupResult<bool> {
        self.store.active_group_exists(code).await.map_err(storage_error)
    }
}

fn record_input(input: BillingGroupCreate, is_system: bool) -> BillingGroupRecordInput {
    BillingGroupRecordInput {
        code: input.code,
        name: input.name,
        description: input.description,
        billing_multiplier: input.billing_multiplier,
        allowed_model_ids: input.allowed_model_ids,
        allowed_provider_group_ids: input.allowed_provider_group_ids,
        allowed_provider_key_group_ids: input.allowed_provider_key_group_ids,
        visible_user_group_codes: input.visible_user_group_codes,
        is_active: input.is_active.unwrap_or(true),
        is_system,
        sort_order: input.sort_order.unwrap_or(0),
    }
}

fn record_patch(input: BillingGroupUpdate) -> BillingGroupRecordPatch {
    BillingGroupRecordPatch {
        name: input.name,
        description: input.description,
        billing_multiplier: input.billing_multiplier,
        allowed_model_ids: input.allowed_model_ids,
        allowed_provider_group_ids: input.allowed_provider_group_ids,
        allowed_provider_key_group_ids: input.allowed_provider_key_group_ids,
        visible_user_group_codes: input.visible_user_group_codes,
        is_active: input.is_active,
        sort_order: input.sort_order,
    }
}

fn storage_error(error: StorageError) -> GroupError {
    match error {
        StorageError::NotFound => GroupError::NotFound,
        StorageError::Conflict(message) => GroupError::Conflict(message),
        StorageError::Database(message) => GroupError::Infrastructure(message),
    }
}

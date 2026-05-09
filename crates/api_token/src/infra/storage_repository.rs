use async_trait::async_trait;
use storage::{
    Database, StorageError,
    api_token::{ApiTokenRecordInput, ApiTokenRecordPatch, ApiTokenStore},
    group::GroupStore,
    model::ModelStore,
    user::UserStore,
};
use types::{
    api_token::{ApiToken, ApiTokenListRequest, ApiTokenListResponse},
    group::BillingGroupResponse,
    user::UserId,
};

use crate::application::{
    ApiTokenCreateRecord, ApiTokenError, ApiTokenRepository, ApiTokenResult, ApiTokenUpdateRecord, BillingGroupCatalog, ModelAccessCatalog, UserCatalog,
};

#[derive(Clone)]
pub struct StorageApiTokenRepository {
    store: ApiTokenStore,
}

#[derive(Clone)]
pub struct StorageBillingGroupCatalog {
    store: GroupStore,
}

#[derive(Clone)]
pub struct StorageModelAccessCatalog {
    store: ModelStore,
}

#[derive(Clone)]
pub struct StorageUserCatalog {
    store: UserStore,
}

impl StorageApiTokenRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: ApiTokenStore::new(database),
        }
    }
}

impl StorageBillingGroupCatalog {
    pub fn new(database: Database) -> Self {
        Self {
            store: GroupStore::new(database),
        }
    }
}

impl StorageModelAccessCatalog {
    pub fn new(database: Database) -> Self {
        Self {
            store: ModelStore::new(database),
        }
    }
}

impl StorageUserCatalog {
    pub fn new(database: Database) -> Self {
        Self {
            store: UserStore::new(database),
        }
    }
}

#[async_trait]
impl ApiTokenRepository for StorageApiTokenRepository {
    async fn create_token(&self, input: ApiTokenCreateRecord) -> ApiTokenResult<ApiToken> {
        self.store.create_token(record_input(input)).await.map_err(storage_error)
    }

    async fn update_token(&self, user_id: &str, id: &str, input: ApiTokenUpdateRecord) -> ApiTokenResult<ApiToken> {
        self.store.update_token(user_id, id, record_patch(input)).await.map_err(storage_error)
    }

    async fn update_any_token(&self, id: &str, input: ApiTokenUpdateRecord) -> ApiTokenResult<ApiToken> {
        self.store.update_any_token(id, record_patch(input)).await.map_err(storage_error)
    }

    async fn delete_token(&self, user_id: &str, id: &str) -> ApiTokenResult<()> {
        self.store.delete_token(user_id, id).await.map_err(storage_error)
    }

    async fn delete_any_token(&self, id: &str) -> ApiTokenResult<()> {
        self.store.delete_any_token(id).await.map_err(storage_error)
    }

    async fn find_user_token(&self, user_id: &str, id: &str) -> ApiTokenResult<Option<ApiToken>> {
        self.store.find_user_token(user_id, id).await.map_err(storage_error)
    }

    async fn find_token(&self, id: &str) -> ApiTokenResult<Option<ApiToken>> {
        self.store.find_token(id).await.map_err(storage_error)
    }

    async fn find_by_hash(&self, token_hash: &str) -> ApiTokenResult<Option<ApiToken>> {
        self.store.find_by_hash(token_hash).await.map_err(storage_error)
    }

    async fn list_user_tokens(&self, user_id: &str, request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
        self.store.list_user_tokens(user_id, request).await.map_err(storage_error)
    }

    async fn list_admin_tokens(&self, request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
        self.store.list_admin_tokens(request).await.map_err(storage_error)
    }
}

#[async_trait]
impl BillingGroupCatalog for StorageBillingGroupCatalog {
    async fn active_group(&self, code: &str) -> ApiTokenResult<Option<BillingGroupResponse>> {
        let group = self.store.find_group(code).await.map_err(storage_error)?;
        Ok(group.filter(|item| item.is_active).map(Into::into))
    }
}

#[async_trait]
impl ModelAccessCatalog for StorageModelAccessCatalog {
    async fn model_exists(&self, id: &str) -> ApiTokenResult<bool> {
        self.store.get_global_model(id).await.map(|model| model.is_some()).map_err(storage_error)
    }
}

#[async_trait]
impl UserCatalog for StorageUserCatalog {
    async fn user_exists(&self, id: &str) -> ApiTokenResult<bool> {
        self.store
            .find_by_id(UserId(id.to_owned()))
            .await
            .map(|user| user.is_some())
            .map_err(storage_error)
    }
}

fn record_input(input: ApiTokenCreateRecord) -> ApiTokenRecordInput {
    ApiTokenRecordInput {
        user_id: input.user_id,
        token_type: input.token_type,
        name: input.name,
        token_value: input.token_value,
        token_hash: input.token_hash,
        token_prefix: input.token_prefix,
        group_code: input.group_code,
        expires_at: input.expires_at,
        model_access_mode: input.model_access_mode,
        allowed_model_ids: input.allowed_model_ids,
        rate_limit_rpm: input.rate_limit_rpm,
        quota_limit: input.quota_limit,
    }
}

fn record_patch(input: ApiTokenUpdateRecord) -> ApiTokenRecordPatch {
    ApiTokenRecordPatch {
        name: input.name,
        group_code: input.group_code,
        expires_at: input.expires_at,
        model_access_mode: input.model_access_mode,
        allowed_model_ids: input.allowed_model_ids,
        rate_limit_rpm: input.rate_limit_rpm,
        quota_limit: input.quota_limit,
        is_active: input.is_active,
    }
}

fn storage_error(error: StorageError) -> ApiTokenError {
    match error {
        StorageError::NotFound => ApiTokenError::NotFound,
        StorageError::Conflict(message) => ApiTokenError::Conflict(message),
        StorageError::Database(message) => ApiTokenError::Infrastructure(message),
    }
}

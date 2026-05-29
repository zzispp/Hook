use std::collections::BTreeMap;

use async_trait::async_trait;
use storage::{
    Database, StorageError,
    api_token::{ApiTokenRecordInput, ApiTokenRecordPatch, ApiTokenStore, ApiTokenUsageRecord},
    group::GroupStore,
    model::ModelStore,
    model_status::ModelStatusStore,
    setting::SettingStore,
    user::{UserGroupStore, UserStore},
};
use types::{
    api_token::{ApiToken, ApiTokenListRequest, ApiTokenListResponse, ApiTokenOwnerResponse},
    group::BillingGroupResponse,
    user::UserId,
};

use crate::application::{
    ApiTokenCreateRecord, ApiTokenError, ApiTokenRepository, ApiTokenResult, ApiTokenUpdateRecord, BillingGroupCatalog, ModelAccessCatalog, SystemTokenPolicy,
    UserCatalog,
};

#[derive(Clone)]
pub struct StorageApiTokenRepository {
    store: ApiTokenStore,
    model_status: ModelStatusStore,
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
    user_groups: UserGroupStore,
    system_owner: Option<(String, ApiTokenOwnerResponse)>,
}

#[derive(Clone)]
pub struct StorageSystemTokenPolicy {
    store: SettingStore,
}

impl StorageApiTokenRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: ApiTokenStore::new(database.clone()),
            model_status: ModelStatusStore::new(database),
        }
    }

    pub async fn record_usage(&self, input: ApiTokenUsageRecord) -> ApiTokenResult<()> {
        self.store.record_usage(input).await.map_err(storage_error)
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
            store: UserStore::new(database.clone()),
            user_groups: UserGroupStore::new(database),
            system_owner: None,
        }
    }

    pub fn with_system_owner(database: Database, system_owner: Option<(String, ApiTokenOwnerResponse)>) -> Self {
        Self {
            store: UserStore::new(database.clone()),
            user_groups: UserGroupStore::new(database),
            system_owner,
        }
    }
}

impl StorageSystemTokenPolicy {
    pub fn new(database: Database) -> Self {
        Self {
            store: SettingStore::new(database),
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

    async fn delete_expired_tokens(&self) -> ApiTokenResult<u64> {
        self.store.delete_expired_tokens().await.map_err(storage_error)
    }

    async fn count_owner_tokens(&self, user_id: &str, token_type: types::api_token::ApiTokenType) -> ApiTokenResult<u64> {
        self.store.count_owner_tokens(user_id, token_type).await.map_err(storage_error)
    }

    async fn token_has_model_status_checks(&self, id: &str) -> ApiTokenResult<bool> {
        self.model_status.token_has_checks(id).await.map_err(storage_error)
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
        if system_owner_id_matches(&self.system_owner, id) {
            return Ok(true);
        }
        self.store
            .find_by_id(UserId(id.to_owned()))
            .await
            .map(|user| user.is_some())
            .map_err(storage_error)
    }

    async fn user_group_code(&self, id: &str) -> ApiTokenResult<Option<String>> {
        if system_owner_id_matches(&self.system_owner, id) {
            return Ok(Some(constants::user_group::DEFAULT_USER_GROUP_CODE.into()));
        }
        let Some(user) = self.store.find_by_id(UserId(id.to_owned())).await.map_err(storage_error)? else {
            return Ok(None);
        };
        if !self.user_groups.active_group_exists(&user.group_code).await.map_err(storage_error)? {
            return Err(ApiTokenError::InvalidInput(format!("active user group does not exist: {}", user.group_code)));
        }
        Ok(Some(user.group_code))
    }

    async fn owners_by_id(&self, ids: &[String]) -> ApiTokenResult<BTreeMap<String, ApiTokenOwnerResponse>> {
        let (mut owners, database_ids) = split_owner_ids(ids, &self.system_owner);
        if database_ids.is_empty() {
            return Ok(owners);
        }
        owners.extend(self.store.find_by_ids(&database_ids).await.map_err(storage_error)?.into_iter().map(|user| {
            (
                user.id.0,
                ApiTokenOwnerResponse {
                    username: user.username,
                    email: user.email,
                    group_code: user.group_code,
                },
            )
        }));
        Ok(owners)
    }
}

fn split_owner_ids(ids: &[String], system_owner: &Option<(String, ApiTokenOwnerResponse)>) -> (BTreeMap<String, ApiTokenOwnerResponse>, Vec<String>) {
    let mut owners = BTreeMap::new();
    let database_ids = ids
        .iter()
        .filter(|id| {
            if let Some((owner_id, owner)) = system_owner
                && *id == owner_id
            {
                owners.insert(owner_id.clone(), owner.clone());
                return false;
            }
            true
        })
        .cloned()
        .collect();
    (owners, database_ids)
}

fn system_owner_id_matches(system_owner: &Option<(String, ApiTokenOwnerResponse)>, id: &str) -> bool {
    system_owner.as_ref().is_some_and(|(owner_id, _)| owner_id == id)
}

#[async_trait]
impl SystemTokenPolicy for StorageSystemTokenPolicy {
    async fn default_rate_limit_rpm(&self) -> ApiTokenResult<i64> {
        self.store
            .get_system_settings()
            .await
            .map(|settings| settings.default_rate_limit_rpm)
            .map_err(storage_error)
    }

    async fn token_limit_per_user(&self) -> ApiTokenResult<i64> {
        self.store
            .get_system_settings()
            .await
            .map(|settings| settings.token_limit_per_user)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_owner_ids_resolves_system_owner_without_database_lookup() {
        let system_owner = Some((
            "system-user".into(),
            ApiTokenOwnerResponse {
                username: "admin".into(),
                email: "admin@example.test".into(),
                group_code: constants::user_group::DEFAULT_USER_GROUP_CODE.into(),
            },
        ));

        let (owners, database_ids) = split_owner_ids(&["system-user".into(), "db-user".into()], &system_owner);

        assert_eq!(database_ids, vec!["db-user".to_owned()]);
        assert_eq!(
            owners.get("system-user"),
            Some(&ApiTokenOwnerResponse {
                username: "admin".into(),
                email: "admin@example.test".into(),
                group_code: constants::user_group::DEFAULT_USER_GROUP_CODE.into(),
            })
        );
    }
}

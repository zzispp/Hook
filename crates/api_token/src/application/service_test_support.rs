use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::api_token::{
    AdminApiTokenCreate, ApiToken, ApiTokenCreate, ApiTokenListRequest, ApiTokenListResponse, ApiTokenOwnerResponse, ApiTokenType, ModelAccessMode,
};

use super::{
    ApiTokenCreateRecord, ApiTokenRepository, ApiTokenResult, ApiTokenService, ApiTokenUpdateRecord, BillingGroupCatalog, ModelAccessCatalog,
    SystemTokenPolicy, UserCatalog,
};

pub(super) const SYSTEM_ACTOR_ID: &str = "00000000-0000-7000-8000-000000000000";
pub(super) const USER_ID: &str = "user-1";

const DEFAULT_RATE_LIMIT_RPM: i64 = 0;
const DEFAULT_TOKEN_LIMIT_PER_USER: i64 = 5;
const EMPTY_OWNER_TOKEN_COUNT: u64 = 0;

pub(super) type TestApiTokenService = ApiTokenService<MemoryTokenRepository, StaticGroups, StaticModels, ExistingUsers, StaticPolicy>;

pub(super) fn service(repository: MemoryTokenRepository, users: ExistingUsers) -> TestApiTokenService {
    service_with_policy(repository, users, StaticPolicy::default())
}

pub(super) fn service_with_policy(repository: MemoryTokenRepository, users: ExistingUsers, policy: StaticPolicy) -> TestApiTokenService {
    ApiTokenService::new(repository, StaticGroups, StaticModels, users, policy)
}

pub(super) fn admin_create(token_type: ApiTokenType, user_id: Option<&str>) -> AdminApiTokenCreate {
    AdminApiTokenCreate {
        name: "test token".into(),
        token_type,
        user_id: user_id.map(str::to_owned),
        group_code: Some(constants::billing::DEFAULT_SYSTEM_GROUP_CODE.into()),
        expires_at: None,
        model_access_mode: Some(ModelAccessMode::All),
        allowed_model_ids: Vec::new(),
        rate_limit_rpm: Some(DEFAULT_RATE_LIMIT_RPM),
        quota_limit: None,
    }
}

pub(super) fn user_create() -> ApiTokenCreate {
    ApiTokenCreate {
        name: "test token".into(),
        group_code: Some(constants::billing::DEFAULT_SYSTEM_GROUP_CODE.into()),
        expires_at: None,
        model_access_mode: Some(ModelAccessMode::All),
        allowed_model_ids: Vec::new(),
        rate_limit_rpm: Some(DEFAULT_RATE_LIMIT_RPM),
        quota_limit: None,
    }
}

pub(super) fn record_owner(user_id: Option<&str>, token_type: ApiTokenType) -> (Option<String>, ApiTokenType) {
    (user_id.map(str::to_owned), token_type)
}

#[derive(Clone)]
pub(super) struct MemoryTokenRepository {
    created: Arc<Mutex<Vec<ApiTokenCreateRecord>>>,
    owner_token_count: u64,
}

impl MemoryTokenRepository {
    pub(super) fn with_owner_token_count(owner_token_count: u64) -> Self {
        Self {
            owner_token_count,
            ..Self::default()
        }
    }

    pub(super) fn created_records(&self) -> Vec<(Option<String>, ApiTokenType)> {
        self.created
            .lock()
            .unwrap()
            .iter()
            .map(|record| (record.user_id.clone(), record.token_type))
            .collect()
    }
}

impl Default for MemoryTokenRepository {
    fn default() -> Self {
        Self {
            created: Arc::default(),
            owner_token_count: EMPTY_OWNER_TOKEN_COUNT,
        }
    }
}

#[async_trait]
impl ApiTokenRepository for MemoryTokenRepository {
    async fn create_token(&self, input: ApiTokenCreateRecord) -> ApiTokenResult<ApiToken> {
        self.created.lock().unwrap().push(input.clone());
        Ok(token_from_record(input))
    }

    async fn update_token(&self, _user_id: &str, _id: &str, _input: ApiTokenUpdateRecord) -> ApiTokenResult<ApiToken> {
        unimplemented!("not needed for service tests")
    }

    async fn update_any_token(&self, _id: &str, _input: ApiTokenUpdateRecord) -> ApiTokenResult<ApiToken> {
        unimplemented!("not needed for service tests")
    }

    async fn delete_token(&self, _user_id: &str, _id: &str) -> ApiTokenResult<()> {
        unimplemented!("not needed for service tests")
    }

    async fn delete_any_token(&self, _id: &str) -> ApiTokenResult<()> {
        unimplemented!("not needed for service tests")
    }

    async fn find_user_token(&self, _user_id: &str, _id: &str) -> ApiTokenResult<Option<ApiToken>> {
        unimplemented!("not needed for service tests")
    }

    async fn find_token(&self, _id: &str) -> ApiTokenResult<Option<ApiToken>> {
        unimplemented!("not needed for service tests")
    }

    async fn find_by_hash(&self, _token_hash: &str) -> ApiTokenResult<Option<ApiToken>> {
        unimplemented!("not needed for service tests")
    }

    async fn list_user_tokens(&self, _user_id: &str, _request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
        unimplemented!("not needed for service tests")
    }

    async fn list_admin_tokens(&self, _request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
        unimplemented!("not needed for service tests")
    }

    async fn delete_expired_tokens(&self) -> ApiTokenResult<u64> {
        unimplemented!("not needed for service tests")
    }

    async fn count_owner_tokens(&self, _user_id: &str, _token_type: ApiTokenType) -> ApiTokenResult<u64> {
        Ok(self.owner_token_count)
    }
}

fn token_from_record(record: ApiTokenCreateRecord) -> ApiToken {
    ApiToken {
        id: "token-1".into(),
        user_id: record.user_id,
        token_type: record.token_type,
        name: record.name,
        token_value: record.token_value,
        token_hash: record.token_hash,
        token_prefix: record.token_prefix,
        group_code: record.group_code,
        expires_at: None,
        model_access_mode: record.model_access_mode,
        allowed_model_ids: record.allowed_model_ids,
        rate_limit_rpm: record.rate_limit_rpm,
        quota_limit: record.quota_limit,
        used_quota: Decimal::ZERO,
        request_count: 0,
        is_active: true,
        last_used_at: None,
        created_at: "2026-05-11T00:00:00Z".into(),
        updated_at: "2026-05-11T00:00:00Z".into(),
    }
}

pub(super) struct StaticGroups;

#[async_trait]
impl BillingGroupCatalog for StaticGroups {
    async fn active_group(&self, code: &str) -> ApiTokenResult<Option<types::group::BillingGroupResponse>> {
        Ok(Some(types::group::BillingGroupResponse {
            id: "group-1".into(),
            code: code.into(),
            name: "Default".into(),
            description: None,
            billing_multiplier: Decimal::ONE,
            allowed_model_ids: Vec::new(),
            allowed_provider_ids: Vec::new(),
            is_active: true,
            is_system: true,
            sort_order: 0,
            created_at: "2026-05-11T00:00:00Z".into(),
            updated_at: "2026-05-11T00:00:00Z".into(),
        }))
    }
}

pub(super) struct StaticModels;

#[async_trait]
impl ModelAccessCatalog for StaticModels {
    async fn model_exists(&self, _id: &str) -> ApiTokenResult<bool> {
        Ok(true)
    }
}

#[derive(Clone)]
pub(super) struct ExistingUsers {
    ids: Arc<Vec<String>>,
}

impl ExistingUsers {
    pub(super) fn empty() -> Self {
        Self { ids: Arc::new(Vec::new()) }
    }

    pub(super) fn with<const N: usize>(ids: [&str; N]) -> Self {
        Self {
            ids: Arc::new(ids.into_iter().map(str::to_owned).collect()),
        }
    }
}

#[async_trait]
impl UserCatalog for ExistingUsers {
    async fn user_exists(&self, id: &str) -> ApiTokenResult<bool> {
        Ok(self.ids.iter().any(|existing| existing == id))
    }

    async fn owners_by_id(&self, ids: &[String]) -> ApiTokenResult<BTreeMap<String, ApiTokenOwnerResponse>> {
        Ok(ids
            .iter()
            .filter(|id| self.ids.iter().any(|existing| existing == *id))
            .map(|id| {
                (
                    id.clone(),
                    ApiTokenOwnerResponse {
                        username: id.clone(),
                        email: format!("{id}@example.test"),
                    },
                )
            })
            .collect())
    }
}

#[derive(Clone, Copy)]
pub(super) struct StaticPolicy;

impl Default for StaticPolicy {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl SystemTokenPolicy for StaticPolicy {
    async fn default_rate_limit_rpm(&self) -> ApiTokenResult<i64> {
        Ok(DEFAULT_RATE_LIMIT_RPM)
    }

    async fn token_limit_per_user(&self) -> ApiTokenResult<i64> {
        Ok(DEFAULT_TOKEN_LIMIT_PER_USER)
    }
}

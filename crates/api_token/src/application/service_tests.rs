use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::api_token::{
    AdminApiTokenCreate, ApiToken, ApiTokenListRequest, ApiTokenListResponse, ApiTokenType, ModelAccessMode,
};

use super::{
    ApiTokenCreateRecord, ApiTokenRepository, ApiTokenResult, ApiTokenService, ApiTokenUpdateRecord, ApiTokenUseCase, BillingGroupCatalog,
    ModelAccessCatalog, SystemTokenPolicy, UserCatalog,
};

const SYSTEM_ACTOR_ID: &str = "00000000-0000-7000-8000-000000000000";
const USER_ID: &str = "user-1";

#[tokio::test]
async fn admin_independent_token_keeps_user_id_empty() {
    let repository = MemoryTokenRepository::default();
    let service = service(repository.clone(), ExistingUsers::empty());

    let created = service
        .create_admin_token(SYSTEM_ACTOR_ID, admin_create(ApiTokenType::Independent, None))
        .await
        .unwrap();

    assert_eq!(created.token.user_id, None);
    assert_eq!(created.token.token_type, ApiTokenType::Independent);
    assert_eq!(repository.created_records(), vec![record_owner(None, ApiTokenType::Independent)]);
}

#[tokio::test]
async fn admin_user_token_requires_existing_user() {
    let repository = MemoryTokenRepository::default();
    let service = service(repository.clone(), ExistingUsers::with([USER_ID]));

    let created = service
        .create_admin_token(SYSTEM_ACTOR_ID, admin_create(ApiTokenType::User, Some(USER_ID)))
        .await
        .unwrap();

    assert_eq!(created.token.user_id, Some(USER_ID.into()));
    assert_eq!(created.token.token_type, ApiTokenType::User);
    assert_eq!(repository.created_records(), vec![record_owner(Some(USER_ID), ApiTokenType::User)]);
}

#[tokio::test]
async fn admin_user_token_rejects_missing_user_id() {
    let service = service(MemoryTokenRepository::default(), ExistingUsers::with([USER_ID]));

    let result = service
        .create_admin_token(SYSTEM_ACTOR_ID, admin_create(ApiTokenType::User, None))
        .await;

    assert!(result.is_err_and(|error| error.to_string().contains("user_id is required")));
}

#[tokio::test]
async fn admin_user_token_rejects_unknown_user() {
    let service = service(MemoryTokenRepository::default(), ExistingUsers::empty());

    let result = service
        .create_admin_token(SYSTEM_ACTOR_ID, admin_create(ApiTokenType::User, Some(USER_ID)))
        .await;

    assert!(result.is_err_and(|error| error.to_string().contains("user does not exist")));
}

fn service(
    repository: MemoryTokenRepository,
    users: ExistingUsers,
) -> ApiTokenService<MemoryTokenRepository, StaticGroups, StaticModels, ExistingUsers, StaticPolicy> {
    ApiTokenService::new(repository, StaticGroups, StaticModels, users, StaticPolicy)
}

fn admin_create(token_type: ApiTokenType, user_id: Option<&str>) -> AdminApiTokenCreate {
    AdminApiTokenCreate {
        name: "test token".into(),
        token_type,
        user_id: user_id.map(str::to_owned),
        group_code: Some(constants::billing::DEFAULT_SYSTEM_GROUP_CODE.into()),
        expires_at: None,
        model_access_mode: Some(ModelAccessMode::All),
        allowed_model_ids: Vec::new(),
        rate_limit_rpm: Some(0),
        quota_limit: None,
    }
}

fn record_owner(user_id: Option<&str>, token_type: ApiTokenType) -> (Option<String>, ApiTokenType) {
    (user_id.map(str::to_owned), token_type)
}

#[derive(Clone, Default)]
struct MemoryTokenRepository {
    created: Arc<Mutex<Vec<ApiTokenCreateRecord>>>,
}

impl MemoryTokenRepository {
    fn created_records(&self) -> Vec<(Option<String>, ApiTokenType)> {
        self.created
            .lock()
            .unwrap()
            .iter()
            .map(|record| (record.user_id.clone(), record.token_type))
            .collect()
    }
}

#[async_trait]
impl ApiTokenRepository for MemoryTokenRepository {
    async fn create_token(&self, input: ApiTokenCreateRecord) -> ApiTokenResult<ApiToken> {
        self.created.lock().unwrap().push(input.clone());
        Ok(token_from_record(input))
    }

    async fn update_token(&self, _user_id: &str, _id: &str, _input: ApiTokenUpdateRecord) -> ApiTokenResult<ApiToken> {
        unimplemented!("not needed for create_admin_token tests")
    }

    async fn update_any_token(&self, _id: &str, _input: ApiTokenUpdateRecord) -> ApiTokenResult<ApiToken> {
        unimplemented!("not needed for create_admin_token tests")
    }

    async fn delete_token(&self, _user_id: &str, _id: &str) -> ApiTokenResult<()> {
        unimplemented!("not needed for create_admin_token tests")
    }

    async fn delete_any_token(&self, _id: &str) -> ApiTokenResult<()> {
        unimplemented!("not needed for create_admin_token tests")
    }

    async fn find_user_token(&self, _user_id: &str, _id: &str) -> ApiTokenResult<Option<ApiToken>> {
        unimplemented!("not needed for create_admin_token tests")
    }

    async fn find_token(&self, _id: &str) -> ApiTokenResult<Option<ApiToken>> {
        unimplemented!("not needed for create_admin_token tests")
    }

    async fn find_by_hash(&self, _token_hash: &str) -> ApiTokenResult<Option<ApiToken>> {
        unimplemented!("not needed for create_admin_token tests")
    }

    async fn list_user_tokens(&self, _user_id: &str, _request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
        unimplemented!("not needed for create_admin_token tests")
    }

    async fn list_admin_tokens(&self, _request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
        unimplemented!("not needed for create_admin_token tests")
    }

    async fn delete_expired_tokens(&self) -> ApiTokenResult<u64> {
        unimplemented!("not needed for create_admin_token tests")
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

struct StaticGroups;

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

struct StaticModels;

#[async_trait]
impl ModelAccessCatalog for StaticModels {
    async fn model_exists(&self, _id: &str) -> ApiTokenResult<bool> {
        Ok(true)
    }
}

#[derive(Clone)]
struct ExistingUsers {
    ids: Arc<Vec<String>>,
}

impl ExistingUsers {
    fn empty() -> Self {
        Self { ids: Arc::new(Vec::new()) }
    }

    fn with<const N: usize>(ids: [&str; N]) -> Self {
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
}

struct StaticPolicy;

#[async_trait]
impl SystemTokenPolicy for StaticPolicy {
    async fn default_rate_limit_rpm(&self) -> ApiTokenResult<i64> {
        Ok(0)
    }

    async fn auto_delete_expired_tokens(&self) -> ApiTokenResult<bool> {
        Ok(false)
    }
}

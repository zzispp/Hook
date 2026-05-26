use std::collections::BTreeMap;

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::api_token::{ApiToken, ApiTokenListRequest, ApiTokenListResponse, ApiTokenOwnerResponse, ApiTokenType, ModelAccessMode};

use super::{
    ApiTokenCreateRecord, ApiTokenRepository, ApiTokenResult, ApiTokenService, ApiTokenUpdateRecord, ApiTokenUseCase, BillingGroupCatalog, ModelAccessCatalog,
    SystemTokenPolicy, UserCatalog,
};

const OWNER_ID: &str = "user-1";
const OWNER_USERNAME: &str = "alice";
const OWNER_EMAIL: &str = "alice@example.test";
const OWNER_GROUP_CODE: &str = constants::user_group::DEFAULT_USER_GROUP_CODE;

#[tokio::test]
async fn admin_token_list_includes_owner_identity() {
    let service = ApiTokenService::new(ListRepository, StaticGroups, StaticModels, OwnerUsers, StaticPolicy);

    let response = service.list_admin_tokens(list_request()).await.unwrap();

    assert_eq!(response.tokens.len(), 1);
    assert_eq!(response.tokens[0].user_id.as_deref(), Some(OWNER_ID));
    assert_eq!(
        response.tokens[0].owner,
        Some(ApiTokenOwnerResponse {
            username: OWNER_USERNAME.into(),
            email: OWNER_EMAIL.into(),
            group_code: OWNER_GROUP_CODE.into(),
        })
    );
}

struct ListRepository;

#[async_trait]
impl ApiTokenRepository for ListRepository {
    async fn create_token(&self, _input: ApiTokenCreateRecord) -> ApiTokenResult<ApiToken> {
        unimplemented!("not needed for list owner tests")
    }

    async fn update_token(&self, _user_id: &str, _id: &str, _input: ApiTokenUpdateRecord) -> ApiTokenResult<ApiToken> {
        unimplemented!("not needed for list owner tests")
    }

    async fn update_any_token(&self, _id: &str, _input: ApiTokenUpdateRecord) -> ApiTokenResult<ApiToken> {
        unimplemented!("not needed for list owner tests")
    }

    async fn delete_token(&self, _user_id: &str, _id: &str) -> ApiTokenResult<()> {
        unimplemented!("not needed for list owner tests")
    }

    async fn delete_any_token(&self, _id: &str) -> ApiTokenResult<()> {
        unimplemented!("not needed for list owner tests")
    }

    async fn find_user_token(&self, _user_id: &str, _id: &str) -> ApiTokenResult<Option<ApiToken>> {
        unimplemented!("not needed for list owner tests")
    }

    async fn find_token(&self, _id: &str) -> ApiTokenResult<Option<ApiToken>> {
        unimplemented!("not needed for list owner tests")
    }

    async fn find_by_hash(&self, _token_hash: &str) -> ApiTokenResult<Option<ApiToken>> {
        unimplemented!("not needed for list owner tests")
    }

    async fn list_user_tokens(&self, _user_id: &str, _request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
        unimplemented!("not needed for list owner tests")
    }

    async fn list_admin_tokens(&self, _request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
        Ok(ApiTokenListResponse {
            tokens: vec![token().into()],
            total: 1,
        })
    }

    async fn delete_expired_tokens(&self) -> ApiTokenResult<u64> {
        unimplemented!("not needed for list owner tests")
    }

    async fn count_owner_tokens(&self, _user_id: &str, _token_type: ApiTokenType) -> ApiTokenResult<u64> {
        unimplemented!("not needed for list owner tests")
    }
}

struct OwnerUsers;

#[async_trait]
impl UserCatalog for OwnerUsers {
    async fn user_exists(&self, _id: &str) -> ApiTokenResult<bool> {
        unimplemented!("not needed for list owner tests")
    }

    async fn user_group_code(&self, _id: &str) -> ApiTokenResult<Option<String>> {
        unimplemented!("not needed for list owner tests")
    }

    async fn owners_by_id(&self, ids: &[String]) -> ApiTokenResult<BTreeMap<String, ApiTokenOwnerResponse>> {
        assert_eq!(ids, &[OWNER_ID.to_owned()]);
        Ok(BTreeMap::from([(
            OWNER_ID.into(),
            ApiTokenOwnerResponse {
                username: OWNER_USERNAME.into(),
                email: OWNER_EMAIL.into(),
                group_code: OWNER_GROUP_CODE.into(),
            },
        )]))
    }
}

struct StaticGroups;

#[async_trait]
impl BillingGroupCatalog for StaticGroups {
    async fn active_group(&self, _code: &str) -> ApiTokenResult<Option<types::group::BillingGroupResponse>> {
        unimplemented!("not needed for list owner tests")
    }
}

struct StaticModels;

#[async_trait]
impl ModelAccessCatalog for StaticModels {
    async fn model_exists(&self, _id: &str) -> ApiTokenResult<bool> {
        unimplemented!("not needed for list owner tests")
    }
}

struct StaticPolicy;

#[async_trait]
impl SystemTokenPolicy for StaticPolicy {
    async fn default_rate_limit_rpm(&self) -> ApiTokenResult<i64> {
        Ok(0)
    }

    async fn token_limit_per_user(&self) -> ApiTokenResult<i64> {
        Ok(5)
    }
}

fn token() -> ApiToken {
    ApiToken {
        id: "token-1".into(),
        user_id: Some(OWNER_ID.into()),
        token_type: ApiTokenType::User,
        name: "test token".into(),
        token_value: "raw-token".into(),
        token_hash: "hash".into(),
        token_prefix: "hk_test".into(),
        group_code: constants::billing::DEFAULT_SYSTEM_GROUP_CODE.into(),
        expires_at: None,
        model_access_mode: ModelAccessMode::All,
        allowed_model_ids: Vec::new(),
        rate_limit_rpm: Some(0),
        quota_limit: None,
        used_quota: Decimal::ZERO,
        request_count: 0,
        is_active: true,
        last_used_at: None,
        created_at: "2026-05-12T00:00:00Z".into(),
        updated_at: "2026-05-12T00:00:00Z".into(),
    }
}

fn list_request() -> ApiTokenListRequest {
    ApiTokenListRequest {
        limit: 100,
        ..Default::default()
    }
}

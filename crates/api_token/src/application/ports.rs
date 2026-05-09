use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    api_token::{ApiToken, ApiTokenListRequest, ApiTokenListResponse, ApiTokenType, ModelAccessMode},
    group::BillingGroupResponse,
    model::PatchField,
};

use super::ApiTokenResult;

#[derive(Clone, Debug, PartialEq)]
pub struct ApiTokenCreateRecord {
    pub user_id: String,
    pub token_type: ApiTokenType,
    pub name: String,
    pub token_value: String,
    pub token_hash: String,
    pub token_prefix: String,
    pub group_code: String,
    pub expires_at: Option<time::OffsetDateTime>,
    pub model_access_mode: ModelAccessMode,
    pub allowed_model_ids: Vec<String>,
    pub rate_limit_rpm: Option<i64>,
    pub quota_limit: Option<Decimal>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ApiTokenUpdateRecord {
    pub name: Option<String>,
    pub group_code: Option<String>,
    pub expires_at: PatchField<time::OffsetDateTime>,
    pub model_access_mode: Option<ModelAccessMode>,
    pub allowed_model_ids: PatchField<Vec<String>>,
    pub rate_limit_rpm: PatchField<i64>,
    pub quota_limit: PatchField<Decimal>,
    pub is_active: Option<bool>,
}

#[async_trait]
pub trait ApiTokenRepository: Send + Sync + 'static {
    async fn create_token(&self, input: ApiTokenCreateRecord) -> ApiTokenResult<ApiToken>;
    async fn update_token(&self, user_id: &str, id: &str, input: ApiTokenUpdateRecord) -> ApiTokenResult<ApiToken>;
    async fn update_any_token(&self, id: &str, input: ApiTokenUpdateRecord) -> ApiTokenResult<ApiToken>;
    async fn delete_token(&self, user_id: &str, id: &str) -> ApiTokenResult<()>;
    async fn delete_any_token(&self, id: &str) -> ApiTokenResult<()>;
    async fn find_user_token(&self, user_id: &str, id: &str) -> ApiTokenResult<Option<ApiToken>>;
    async fn find_token(&self, id: &str) -> ApiTokenResult<Option<ApiToken>>;
    async fn find_by_hash(&self, token_hash: &str) -> ApiTokenResult<Option<ApiToken>>;
    async fn list_user_tokens(&self, user_id: &str, request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse>;
    async fn list_admin_tokens(&self, request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse>;
}

#[async_trait]
pub trait UserCatalog: Send + Sync + 'static {
    async fn user_exists(&self, id: &str) -> ApiTokenResult<bool>;
}

#[async_trait]
pub trait BillingGroupCatalog: Send + Sync + 'static {
    async fn active_group(&self, code: &str) -> ApiTokenResult<Option<BillingGroupResponse>>;
}

#[async_trait]
pub trait ModelAccessCatalog: Send + Sync + 'static {
    async fn model_exists(&self, id: &str) -> ApiTokenResult<bool>;
}

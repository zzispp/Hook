use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::model::{PatchField, deserialize_patch_value};

const DEFAULT_TOKEN_LIMIT: u64 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelAccessMode {
    All,
    Limited,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiTokenType {
    User,
    Independent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiToken {
    pub id: String,
    pub user_id: Option<String>,
    pub token_type: ApiTokenType,
    pub name: String,
    pub token_value: String,
    pub token_hash: String,
    pub token_prefix: String,
    pub group_code: String,
    pub expires_at: Option<String>,
    pub model_access_mode: ModelAccessMode,
    pub allowed_model_ids: Vec<String>,
    pub rate_limit_rpm: Option<i64>,
    pub quota_limit: Option<Decimal>,
    pub used_quota: Decimal,
    pub request_count: i64,
    pub is_active: bool,
    pub last_used_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct ApiTokenListRequest {
    #[serde(default)]
    pub skip: u64,
    #[serde(default = "default_token_limit")]
    pub limit: u64,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub token_type: Option<ApiTokenType>,
    #[serde(default)]
    pub user_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ApiTokenCreate {
    pub name: String,
    #[serde(default)]
    pub group_code: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub model_access_mode: Option<ModelAccessMode>,
    #[serde(default)]
    pub allowed_model_ids: Vec<String>,
    #[serde(default)]
    pub rate_limit_rpm: Option<i64>,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub quota_limit: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct AdminApiTokenCreate {
    pub name: String,
    pub token_type: ApiTokenType,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub group_code: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub model_access_mode: Option<ModelAccessMode>,
    #[serde(default)]
    pub allowed_model_ids: Vec<String>,
    #[serde(default)]
    pub rate_limit_rpm: Option<i64>,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub quota_limit: Option<Decimal>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct ApiTokenUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub group_code: Option<String>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub expires_at: PatchField<String>,
    #[serde(default)]
    pub model_access_mode: Option<ModelAccessMode>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub allowed_model_ids: PatchField<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub rate_limit_rpm: PatchField<i64>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub quota_limit: PatchField<Decimal>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ApiTokenResponse {
    pub id: String,
    pub user_id: Option<String>,
    pub owner: Option<ApiTokenOwnerResponse>,
    pub token_type: ApiTokenType,
    pub name: String,
    pub token_prefix: String,
    pub group_code: String,
    pub expires_at: Option<String>,
    pub model_access_mode: ModelAccessMode,
    pub allowed_model_ids: Vec<String>,
    pub rate_limit_rpm: Option<i64>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub quota_limit: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float")]
    pub used_quota: Decimal,
    pub request_count: i64,
    pub is_active: bool,
    pub last_used_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ApiTokenOwnerResponse {
    pub username: String,
    pub email: String,
    pub group_code: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ApiTokenCreateResponse {
    pub token: ApiTokenResponse,
    pub raw_token: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ApiTokenSecretResponse {
    pub raw_token: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ApiTokenUsageResponse {
    #[serde(with = "rust_decimal::serde::float")]
    pub used_quota: Decimal,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub quota_limit: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub remaining_quota: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ApiTokenListResponse {
    pub tokens: Vec<ApiTokenResponse>,
    pub total: u64,
}

impl ApiTokenUpdate {
    pub fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.group_code.is_none()
            && self.expires_at.is_missing()
            && self.model_access_mode.is_none()
            && self.allowed_model_ids.is_missing()
            && self.rate_limit_rpm.is_missing()
            && self.quota_limit.is_missing()
            && self.is_active.is_none()
    }
}

impl From<ApiToken> for ApiTokenResponse {
    fn from(value: ApiToken) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            owner: None,
            token_type: value.token_type,
            name: value.name,
            token_prefix: value.token_prefix,
            group_code: value.group_code,
            expires_at: value.expires_at,
            model_access_mode: value.model_access_mode,
            allowed_model_ids: value.allowed_model_ids,
            rate_limit_rpm: value.rate_limit_rpm,
            quota_limit: value.quota_limit,
            used_quota: value.used_quota,
            request_count: value.request_count,
            is_active: value.is_active,
            last_used_at: value.last_used_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl ApiTokenResponse {
    pub fn with_owner(mut self, owner: Option<ApiTokenOwnerResponse>) -> Self {
        self.owner = owner;
        self
    }
}

fn default_token_limit() -> u64 {
    DEFAULT_TOKEN_LIMIT
}

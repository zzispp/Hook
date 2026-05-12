use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use storage::{Database, api_token::ApiTokenStore};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use types::api_token::{ApiToken, ApiTokenType, ModelAccessMode};

use crate::llm_proxy::LlmProxyError;

const TOKEN_CACHE_TTL_SECONDS: u64 = 3600;
const EXPIRED_TOKEN_CACHE_SECONDS: u64 = 60;

#[derive(Deserialize, Serialize)]
struct CachedApiToken {
    id: String,
    user_id: Option<String>,
    token_type: ApiTokenType,
    name: String,
    token_prefix: String,
    group_code: String,
    expires_at: Option<String>,
    model_access_mode: ModelAccessMode,
    allowed_model_ids: Vec<String>,
    rate_limit_rpm: Option<i64>,
    #[serde(with = "rust_decimal::serde::float_option")]
    quota_limit: Option<Decimal>,
    is_active: bool,
    created_at: String,
    updated_at: String,
}

pub async fn load_token(database: &Database, token_hash: &str) -> Result<Option<ApiToken>, LlmProxyError> {
    ApiTokenStore::new(database.clone()).find_by_hash(token_hash).await.map_err(Into::into)
}

pub fn encode_token(token: &ApiToken) -> Result<String, LlmProxyError> {
    serde_json::to_string(&CachedApiToken::from(token)).map_err(json_error)
}

pub fn decode_token(value: &str, token_hash: &str) -> Result<ApiToken, LlmProxyError> {
    let cached = serde_json::from_str::<CachedApiToken>(value).map_err(json_error)?;
    Ok(cached.into_token(token_hash))
}

pub fn token_ttl_seconds(token: &ApiToken) -> u64 {
    let Some(expires_at) = token.expires_at.as_deref() else {
        return TOKEN_CACHE_TTL_SECONDS;
    };
    let Ok(expires_at) = OffsetDateTime::parse(expires_at, &Rfc3339) else {
        return TOKEN_CACHE_TTL_SECONDS;
    };
    let seconds = (expires_at - OffsetDateTime::now_utc()).whole_seconds();
    if seconds <= 0 {
        return EXPIRED_TOKEN_CACHE_SECONDS;
    }
    (seconds as u64).min(TOKEN_CACHE_TTL_SECONDS)
}

impl From<&ApiToken> for CachedApiToken {
    fn from(value: &ApiToken) -> Self {
        Self {
            id: value.id.clone(),
            user_id: value.user_id.clone(),
            token_type: value.token_type,
            name: value.name.clone(),
            token_prefix: value.token_prefix.clone(),
            group_code: value.group_code.clone(),
            expires_at: value.expires_at.clone(),
            model_access_mode: value.model_access_mode,
            allowed_model_ids: value.allowed_model_ids.clone(),
            rate_limit_rpm: value.rate_limit_rpm,
            quota_limit: value.quota_limit,
            is_active: value.is_active,
            created_at: value.created_at.clone(),
            updated_at: value.updated_at.clone(),
        }
    }
}

impl CachedApiToken {
    fn into_token(self, token_hash: &str) -> ApiToken {
        ApiToken {
            id: self.id,
            user_id: self.user_id,
            token_type: self.token_type,
            name: self.name,
            token_value: String::new(),
            token_hash: token_hash.to_owned(),
            token_prefix: self.token_prefix,
            group_code: self.group_code,
            expires_at: self.expires_at,
            model_access_mode: self.model_access_mode,
            allowed_model_ids: self.allowed_model_ids,
            rate_limit_rpm: self.rate_limit_rpm,
            quota_limit: self.quota_limit,
            used_quota: Decimal::ZERO,
            request_count: 0,
            is_active: self.is_active,
            last_used_at: None,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

fn json_error(error: serde_json::Error) -> LlmProxyError {
    LlmProxyError::Infrastructure(format!("proxy auth cache json error: {error}"))
}

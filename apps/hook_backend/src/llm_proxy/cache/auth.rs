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
    #[serde(with = "rust_decimal::serde::float")]
    used_quota: Decimal,
    #[serde(default)]
    request_count: i64,
    is_active: bool,
    #[serde(default)]
    last_used_at: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CachedTokenUsage {
    pub(super) used_quota: Decimal,
    pub(super) request_count: i64,
    pub(super) last_used_at: Option<String>,
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

pub(super) fn seed_cached_usage(token: &ApiToken) -> CachedTokenUsage {
    CachedTokenUsage {
        used_quota: token.used_quota,
        request_count: token.request_count,
        last_used_at: token.last_used_at.clone(),
    }
}

pub(super) fn apply_cached_usage(mut token: ApiToken, usage: &CachedTokenUsage) -> ApiToken {
    token.used_quota = usage.used_quota;
    token.request_count = usage.request_count;
    token.last_used_at = usage.last_used_at.clone();
    token
}

pub(super) fn decode_cached_usage(
    used_quota: Option<String>,
    request_count: Option<String>,
    last_used_at: Option<String>,
) -> Result<Option<CachedTokenUsage>, LlmProxyError> {
    if used_quota.is_none() && request_count.is_none() && last_used_at.is_none() {
        return Ok(None);
    }
    let used_quota = used_quota.ok_or_else(|| LlmProxyError::Infrastructure("cached token usage missing used_quota".into()))?;
    let request_count = request_count.ok_or_else(|| LlmProxyError::Infrastructure("cached token usage missing request_count".into()))?;
    Ok(Some(CachedTokenUsage {
        used_quota: used_quota
            .parse::<Decimal>()
            .map_err(|error| LlmProxyError::Infrastructure(format!("invalid cached token used_quota: {error}")))?,
        request_count: request_count
            .parse::<i64>()
            .map_err(|error| LlmProxyError::Infrastructure(format!("invalid cached token request_count: {error}")))?,
        last_used_at: normalize_cached_time(last_used_at),
    }))
}

fn normalize_cached_time(value: Option<String>) -> Option<String> {
    value.and_then(|value| (!value.trim().is_empty()).then_some(value))
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
            used_quota: value.used_quota,
            request_count: value.request_count,
            is_active: value.is_active,
            last_used_at: value.last_used_at.clone(),
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
            used_quota: self.used_quota,
            request_count: self.request_count,
            is_active: self.is_active,
            last_used_at: self.last_used_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

fn json_error(error: serde_json::Error) -> LlmProxyError {
    LlmProxyError::Infrastructure(format!("proxy auth cache json error: {error}"))
}

#[cfg(test)]
fn increment_cached_usage(mut usage: CachedTokenUsage, cost: Decimal, used_at: OffsetDateTime) -> Result<CachedTokenUsage, LlmProxyError> {
    usage.used_quota += cost;
    usage.request_count += 1;
    usage.last_used_at = Some(
        used_at
            .format(&Rfc3339)
            .map_err(|error| LlmProxyError::Infrastructure(format!("invalid cached token usage timestamp: {error}")))?,
    );
    Ok(usage)
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};
    use types::api_token::{ApiToken, ApiTokenType, ModelAccessMode};

    use super::{apply_cached_usage, decode_token, encode_token, increment_cached_usage, seed_cached_usage};

    #[test]
    fn increment_cached_usage_updates_quota_request_count_and_last_used_at() {
        let token = token();
        let used_at = OffsetDateTime::parse("2026-05-15T13:40:00Z", &Rfc3339).unwrap();

        let usage = increment_cached_usage(seed_cached_usage(&token), Decimal::new(25, 1), used_at).unwrap();
        let decoded = apply_cached_usage(decode_token(&encode_token(&token).unwrap(), &token.token_hash).unwrap(), &usage);

        assert_eq!(decoded.used_quota, Decimal::new(75, 1));
        assert_eq!(decoded.request_count, 4);
        assert_eq!(decoded.last_used_at.as_deref(), Some("2026-05-15T13:40:00Z"));
    }

    fn token() -> ApiToken {
        ApiToken {
            id: "token-1".into(),
            user_id: Some("user-1".into()),
            token_type: ApiTokenType::User,
            name: "primary".into(),
            token_value: String::new(),
            token_hash: "hash-1".into(),
            token_prefix: "sk-test".into(),
            group_code: "default".into(),
            expires_at: None,
            model_access_mode: ModelAccessMode::All,
            allowed_model_ids: Vec::new(),
            rate_limit_rpm: Some(30),
            quota_limit: Some(Decimal::new(100, 0)),
            used_quota: Decimal::new(50, 1),
            request_count: 3,
            is_active: true,
            last_used_at: Some("2026-05-14T10:00:00Z".into()),
            created_at: "2026-05-14T09:00:00Z".into(),
            updated_at: "2026-05-14T09:00:00Z".into(),
        }
    }
}

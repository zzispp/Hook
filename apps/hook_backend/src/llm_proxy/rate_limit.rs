use time::OffsetDateTime;
use types::api_token::{ApiToken, ApiTokenType};

use super::{
    LlmProxyError, LlmProxyState,
    cache::snapshot::{CachedUserAccess, SchedulingSnapshot},
};

const RATE_LIMIT_LUA: &str = r#"
for i = 1, #KEYS do
  local limit = tonumber(ARGV[i + 1])
  local current = tonumber(redis.call('GET', KEYS[i]) or '0')
  if current >= limit then
    return i
  end
end
for i = 1, #KEYS do
  local value = redis.call('INCR', KEYS[i])
  if value == 1 then
    redis.call('EXPIRE', KEYS[i], tonumber(ARGV[1]))
  end
end
return 0
"#;

pub async fn enforce_request_limits(state: &LlmProxyState, token: &ApiToken) -> Result<(), LlmProxyError> {
    let snapshot = state.scheduling_snapshot().await?;
    let scopes = request_scopes(&snapshot, token);
    consume_scopes(state, &scopes).await
}

pub async fn claim_provider_key_limit(state: &LlmProxyState, key_id: &str, rpm_limit: Option<i32>) -> Result<(), LlmProxyError> {
    let Some(limit) = normalized_i64(rpm_limit.map(i64::from)) else {
        return Ok(());
    };
    consume_scopes(state, &[RateLimitScope::provider_key(key_id, limit)]).await
}

fn request_scopes(snapshot: &SchedulingSnapshot, token: &ApiToken) -> Vec<RateLimitScope> {
    let mut scopes = Vec::new();
    if let Some(scope) = user_scope(snapshot, token) {
        scopes.push(scope);
    }
    if let Some(scope) = token_scope(snapshot, token) {
        scopes.push(scope);
    }
    scopes
}

fn user_scope(snapshot: &SchedulingSnapshot, token: &ApiToken) -> Option<RateLimitScope> {
    let access = user_access(snapshot, token)?;
    let limit = normalized_i64(access.rate_limit_rpm)?;
    Some(RateLimitScope::user(&access.id, limit))
}

fn token_scope(snapshot: &SchedulingSnapshot, token: &ApiToken) -> Option<RateLimitScope> {
    let configured = normalized_i64(token.rate_limit_rpm).unwrap_or_default();
    let system = normalized_i64(Some(snapshot.default_rate_limit_rpm)).unwrap_or_default();
    let effective = match (configured, system) {
        (0, 0) => None,
        (0, value) | (value, 0) => Some(value),
        (left, right) => Some(left.min(right)),
    }?;
    Some(RateLimitScope::token(&token.id, effective))
}

fn user_access<'a>(snapshot: &'a SchedulingSnapshot, token: &ApiToken) -> Option<&'a CachedUserAccess> {
    if token.token_type != ApiTokenType::User {
        return None;
    }
    let user_id = token.user_id.as_deref()?;
    snapshot.users.iter().find(|user| user.id == user_id)
}

async fn consume_scopes(state: &LlmProxyState, scopes: &[RateLimitScope]) -> Result<(), LlmProxyError> {
    if scopes.is_empty() {
        return Ok(());
    }
    let now = OffsetDateTime::now_utc();
    let bucket = now.unix_timestamp().div_euclid(60);
    let keys = scopes
        .iter()
        .map(|scope| format!("{}:llm_proxy:rate_limit:{}:{}:{bucket}", state.key_prefix, scope.namespace, scope.id))
        .collect::<Vec<_>>();
    let mut connection = state.affinity.clone();
    let mut command = redis::cmd("EVAL");
    command.arg(RATE_LIMIT_LUA).arg(keys.len());
    for key in &keys {
        command.arg(key);
    }
    command.arg(bucket_ttl_seconds(now));
    for scope in scopes {
        command.arg(scope.limit);
    }
    let blocked: i64 = command.query_async(&mut connection).await.map_err(redis_error)?;
    if blocked == 0 {
        return Ok(());
    }
    let index = usize::try_from(blocked - 1).map_err(|_| LlmProxyError::Infrastructure("rate limit script returned invalid index".into()))?;
    let scope = scopes
        .get(index)
        .ok_or_else(|| LlmProxyError::Infrastructure("rate limit script returned out-of-range index".into()))?;
    Err(LlmProxyError::RateLimited(format!("{} rate limit exceeded", scope.label)))
}

fn bucket_ttl_seconds(now: OffsetDateTime) -> i64 {
    60 - now.unix_timestamp().rem_euclid(60) + 60
}

fn normalized_i64(value: Option<i64>) -> Option<i64> {
    value.filter(|limit| *limit > 0)
}

fn redis_error(error: redis::RedisError) -> LlmProxyError {
    LlmProxyError::Infrastructure(format!("redis error: {error}"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RateLimitScope {
    namespace: &'static str,
    label: &'static str,
    id: String,
    limit: i64,
}

impl RateLimitScope {
    fn user(id: &str, limit: i64) -> Self {
        Self {
            namespace: "user",
            label: "user",
            id: id.to_owned(),
            limit,
        }
    }

    fn token(id: &str, limit: i64) -> Self {
        Self {
            namespace: "token",
            label: "token",
            id: id.to_owned(),
            limit,
        }
    }

    fn provider_key(id: &str, limit: i64) -> Self {
        Self {
            namespace: "provider_key",
            label: "provider key",
            id: id.to_owned(),
            limit,
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use types::{
        api_token::{ApiToken, ApiTokenType, ModelAccessMode},
        provider::ProviderSchedulingMode,
    };

    use super::{CachedUserAccess, RateLimitScope, SchedulingSnapshot, request_scopes, token_scope};

    #[test]
    fn token_limit_follows_system_when_configured_zero() {
        let snapshot = snapshot(7, None);
        let scope = token_scope(&snapshot, &token(ApiTokenType::Independent, None, Some(0)));

        assert_eq!(scope, Some(RateLimitScope::token("token-1", 7)));
    }

    #[test]
    fn token_limit_uses_smaller_of_system_and_token() {
        let snapshot = snapshot(7, None);
        let scope = token_scope(&snapshot, &token(ApiTokenType::Independent, None, Some(3)));

        assert_eq!(scope, Some(RateLimitScope::token("token-1", 3)));
    }

    #[test]
    fn token_limit_uses_configured_value_when_system_unlimited() {
        let snapshot = snapshot(0, None);
        let scope = token_scope(&snapshot, &token(ApiTokenType::Independent, None, Some(3)));

        assert_eq!(scope, Some(RateLimitScope::token("token-1", 3)));
    }

    #[test]
    fn request_scopes_include_user_and_token_for_user_tokens() {
        let snapshot = snapshot(7, Some(2));
        let scopes = request_scopes(&snapshot, &token(ApiTokenType::User, Some("user-1"), Some(5)));

        assert_eq!(scopes, vec![RateLimitScope::user("user-1", 2), RateLimitScope::token("token-1", 5.min(7))]);
    }

    #[test]
    fn request_scopes_skip_user_limit_for_independent_tokens() {
        let snapshot = snapshot(7, Some(2));
        let scopes = request_scopes(&snapshot, &token(ApiTokenType::Independent, None, Some(5)));

        assert_eq!(scopes, vec![RateLimitScope::token("token-1", 5.min(7))]);
    }

    fn snapshot(default_rate_limit_rpm: i64, user_rate_limit_rpm: Option<i64>) -> SchedulingSnapshot {
        SchedulingSnapshot {
            default_rate_limit_rpm,
            scheduling_mode: ProviderSchedulingMode::FixedOrder,
            models: Vec::new(),
            groups: Vec::new(),
            users: vec![CachedUserAccess {
                id: "user-1".into(),
                username: "alice".into(),
                is_active: true,
                allowed_model_ids: Vec::new(),
                allowed_provider_ids: Vec::new(),
                quota_mode: "wallet".into(),
                rate_limit_rpm: user_rate_limit_rpm,
            }],
            providers: Vec::new(),
        }
    }

    fn token(token_type: ApiTokenType, user_id: Option<&str>, rate_limit_rpm: Option<i64>) -> ApiToken {
        ApiToken {
            id: "token-1".into(),
            user_id: user_id.map(str::to_owned),
            token_type,
            name: "token".into(),
            token_value: String::new(),
            token_hash: String::new(),
            token_prefix: "sk-test".into(),
            group_code: "default".into(),
            expires_at: None,
            model_access_mode: ModelAccessMode::All,
            allowed_model_ids: Vec::new(),
            rate_limit_rpm,
            quota_limit: None,
            used_quota: Decimal::ZERO,
            request_count: 0,
            is_active: true,
            last_used_at: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}

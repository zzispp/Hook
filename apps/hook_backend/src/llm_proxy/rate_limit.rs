use std::time::Duration;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProviderKeyProbeSlotOptions {
    pub(crate) min_interval_seconds: i64,
    pub(crate) wait_timeout_seconds: i64,
}

#[derive(Debug)]
pub(crate) enum ProviderKeyProbeSlotClaim {
    Claimed,
    TimedOut(LlmProxyError),
}

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

pub async fn claim_provider_key_probe_slot(
    state: &LlmProxyState,
    key_id: &str,
    options: ProviderKeyProbeSlotOptions,
) -> Result<ProviderKeyProbeSlotClaim, LlmProxyError> {
    let mut connection = state.affinity.clone();
    let (wait_duration, wait_timeout) = provider_key_probe_slot_durations(options)?;
    let claimed = tokio::time::timeout(wait_timeout, async {
        loop {
            if try_claim_provider_key_probe_slot(&mut connection, &state.key_prefix, key_id, options.min_interval_seconds).await? {
                return Ok::<_, LlmProxyError>(ProviderKeyProbeSlotClaim::Claimed);
            }
            tokio::time::sleep(wait_duration).await;
        }
    })
    .await;
    match claimed {
        Ok(result) => result,
        Err(_) => Ok(ProviderKeyProbeSlotClaim::TimedOut(LlmProxyError::Upstream(format!(
            "provider key model status probe slot wait timed out for key {key_id} after {} seconds",
            options.wait_timeout_seconds
        )))),
    }
}

async fn try_claim_provider_key_probe_slot(
    connection: &mut redis::aio::ConnectionManager,
    key_prefix: &str,
    key_id: &str,
    min_interval_seconds: i64,
) -> Result<bool, LlmProxyError> {
    let result: Option<String> = provider_key_probe_slot_command(key_prefix, key_id, min_interval_seconds)
        .query_async(connection)
        .await
        .map_err(redis_error)?;
    Ok(result.is_some())
}

fn provider_key_probe_slot_durations(options: ProviderKeyProbeSlotOptions) -> Result<(Duration, Duration), LlmProxyError> {
    let wait = positive_seconds(
        options.min_interval_seconds,
        "provider key probe minimum interval",
        "provider key probe minimum interval is out of range",
    )?;
    let timeout = positive_seconds(
        options.wait_timeout_seconds,
        "provider key probe wait timeout",
        "provider key probe wait timeout is out of range",
    )?;
    Ok((Duration::from_secs(wait), Duration::from_secs(timeout)))
}

fn positive_seconds(value: i64, label: &str, range_message: &str) -> Result<u64, LlmProxyError> {
    if value <= 0 {
        return Err(LlmProxyError::Infrastructure(format!("{label} must be greater than 0")));
    }
    u64::try_from(value).map_err(|_| LlmProxyError::Infrastructure(range_message.into()))
}

fn provider_key_probe_slot_command(key_prefix: &str, key_id: &str, min_interval_seconds: i64) -> redis::Cmd {
    let mut command = redis::cmd("SET");
    command
        .arg(format!("{key_prefix}:llm_proxy:model_status_probe_slot:{key_id}"))
        .arg(1)
        .arg("NX")
        .arg("EX")
        .arg(min_interval_seconds);
    command
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
#[path = "rate_limit_tests.rs"]
mod tests;

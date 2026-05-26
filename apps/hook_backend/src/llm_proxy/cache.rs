mod auth;
mod options;
mod provider_cooldown;
mod scheduling_snapshot_write;
pub(super) mod snapshot;
mod usage_flush;

pub use options::LlmProxyCacheOptions;
pub use provider_cooldown::ProviderCooldownFailureInput;

use redis::AsyncCommands;
use rust_decimal::Decimal;
use storage::Database;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use types::api_token::ApiToken;
use uuid::Uuid;

use self::snapshot::{CachedUserAccess, SchedulingSnapshot};
use super::LlmProxyError;

const AUTH_USAGE_USED_QUOTA_FIELD: &str = "used_quota";
const AUTH_USAGE_REQUEST_COUNT_FIELD: &str = "request_count";
const AUTH_USAGE_LAST_USED_AT_FIELD: &str = "last_used_at";
const SCHEDULING_REBUILD_LOCK_SECONDS: u64 = 30;
const SCHEDULING_REBUILD_WAIT_MS: u64 = 50;

#[derive(Clone)]
pub struct LlmProxyCache {
    database: Database,
    connection: redis::aio::ConnectionManager,
    key_prefix: String,
    system_users: Vec<CachedUserAccess>,
    scheduling_snapshot_ttl_seconds: u64,
}

impl LlmProxyCache {
    pub fn new(options: LlmProxyCacheOptions) -> Self {
        let cache = Self {
            database: options.database,
            connection: options.connection,
            key_prefix: options.key_prefix,
            system_users: options.system_users,
            scheduling_snapshot_ttl_seconds: options.scheduling_snapshot_ttl_seconds,
        };
        usage_flush::spawn_usage_flush_task(cache.clone());
        cache
    }

    pub async fn api_token_by_hash(&self, token_hash: &str) -> Result<Option<ApiToken>, LlmProxyError> {
        let version = self.auth_version().await?;
        let cache_key = self.auth_token_key(version, token_hash);
        if let Some(token) = self.read_cached_token(&cache_key, token_hash).await? {
            return self.with_runtime_token_usage(token).await.map(Some);
        }
        let token = auth::load_token(&self.database, token_hash).await?;
        if let Some(token) = token.as_ref() {
            self.write_cached_token(cache_key, token).await?;
        }
        match token {
            Some(token) => self.with_runtime_token_usage(token).await.map(Some),
            None => Ok(None),
        }
    }

    pub async fn bump_auth_version(&self) -> Result<(), LlmProxyError> {
        let mut connection = self.connection.clone();
        let _: i64 = connection.incr(self.auth_version_key(), 1).await.map_err(redis_error)?;
        Ok(())
    }

    pub async fn record_token_usage(&self, token_id: &str, cost: Decimal, used_at: OffsetDateTime) -> Result<(), LlmProxyError> {
        let key = self.auth_usage_key(token_id);
        let mut connection = self.connection.clone();
        let exists: bool = connection.exists(&key).await.map_err(redis_error)?;
        if !exists {
            return Ok(());
        }
        let used_at = used_at.format(&Rfc3339).map_err(runtime_usage_time_error)?;
        let _: () = redis::pipe()
            .cmd("HINCRBYFLOAT")
            .arg(&key)
            .arg(AUTH_USAGE_USED_QUOTA_FIELD)
            .arg(cost.to_string())
            .ignore()
            .cmd("HINCRBY")
            .arg(&key)
            .arg(AUTH_USAGE_REQUEST_COUNT_FIELD)
            .arg(1)
            .ignore()
            .cmd("HSET")
            .arg(&key)
            .arg(AUTH_USAGE_LAST_USED_AT_FIELD)
            .arg(used_at)
            .ignore()
            .query_async(&mut connection)
            .await
            .map_err(redis_error)?;
        Ok(())
    }

    pub async fn scheduling_snapshot(&self) -> Result<SchedulingSnapshot, LlmProxyError> {
        if let Some(snapshot) = self.read_scheduling_snapshot().await? {
            return Ok(snapshot);
        }
        let lock_token = self.wait_for_rebuild_lock().await?;
        let result = match self.read_scheduling_snapshot().await {
            Ok(Some(snapshot)) => Ok(snapshot),
            Ok(None) => self.write_fresh_scheduling_snapshot().await,
            Err(error) => Err(error),
        };
        self.release_locked_result(&lock_token, result).await
    }

    pub async fn refresh_scheduling_snapshot(&self) -> Result<SchedulingSnapshot, LlmProxyError> {
        let lock_token = self.wait_for_rebuild_lock().await?;
        let result = self.replace_scheduling_snapshot().await;
        self.release_locked_result(&lock_token, result).await
    }

    async fn read_cached_token(&self, key: &str, token_hash: &str) -> Result<Option<ApiToken>, LlmProxyError> {
        let mut connection = self.connection.clone();
        let value: Option<String> = connection.get(key).await.map_err(redis_error)?;
        value.map(|json| auth::decode_token(&json, token_hash)).transpose()
    }

    async fn write_cached_token(&self, key: String, token: &ApiToken) -> Result<(), LlmProxyError> {
        let mut connection = self.connection.clone();
        let value = auth::encode_token(token)?;
        let seconds = auth::token_ttl_seconds(token);
        let _: () = connection.set_ex(key, value, seconds).await.map_err(redis_error)?;
        Ok(())
    }

    async fn with_runtime_token_usage(&self, token: ApiToken) -> Result<ApiToken, LlmProxyError> {
        if let Some(usage) = self.read_token_usage(&token.id).await? {
            return Ok(auth::apply_cached_usage(token, &usage));
        }
        self.seed_token_usage(&token).await?;
        Ok(auth::apply_cached_usage(token.clone(), &auth::seed_cached_usage(&token)))
    }

    async fn read_token_usage(&self, token_id: &str) -> Result<Option<auth::CachedTokenUsage>, LlmProxyError> {
        let mut connection = self.connection.clone();
        let values: Vec<Option<String>> = redis::cmd("HMGET")
            .arg(self.auth_usage_key(token_id))
            .arg(&[AUTH_USAGE_USED_QUOTA_FIELD, AUTH_USAGE_REQUEST_COUNT_FIELD, AUTH_USAGE_LAST_USED_AT_FIELD])
            .query_async(&mut connection)
            .await
            .map_err(redis_error)?;
        let [used_quota, request_count, last_used_at] = usage_values(values)?;
        auth::decode_cached_usage(used_quota, request_count, last_used_at)
    }

    async fn seed_token_usage(&self, token: &ApiToken) -> Result<(), LlmProxyError> {
        let usage = auth::seed_cached_usage(token);
        let last_used_at = usage.last_used_at.unwrap_or_default();
        let key = self.auth_usage_key(&token.id);
        let mut connection = self.connection.clone();
        let _: () = redis::pipe()
            .cmd("HSETNX")
            .arg(&key)
            .arg(AUTH_USAGE_USED_QUOTA_FIELD)
            .arg(usage.used_quota.to_string())
            .ignore()
            .cmd("HSETNX")
            .arg(&key)
            .arg(AUTH_USAGE_REQUEST_COUNT_FIELD)
            .arg(usage.request_count.to_string())
            .ignore()
            .cmd("HSETNX")
            .arg(&key)
            .arg(AUTH_USAGE_LAST_USED_AT_FIELD)
            .arg(last_used_at)
            .ignore()
            .cmd("EXPIRE")
            .arg(&key)
            .arg(auth::token_ttl_seconds(token))
            .ignore()
            .query_async(&mut connection)
            .await
            .map_err(redis_error)?;
        Ok(())
    }

    async fn auth_version(&self) -> Result<i64, LlmProxyError> {
        let mut connection = self.connection.clone();
        let value: Option<i64> = connection.get(self.auth_version_key()).await.map_err(redis_error)?;
        Ok(value.unwrap_or_default())
    }

    async fn read_scheduling_snapshot(&self) -> Result<Option<SchedulingSnapshot>, LlmProxyError> {
        let mut connection = self.connection.clone();
        let value: Option<String> = connection.get(self.scheduling_snapshot_key()).await.map_err(redis_error)?;
        value.map(|json| snapshot::decode(&json)).transpose()
    }

    async fn write_fresh_scheduling_snapshot(&self) -> Result<SchedulingSnapshot, LlmProxyError> {
        let snapshot = snapshot::load(&self.database, &self.system_users).await?;
        let value = snapshot::encode(&snapshot)?;
        let mut connection = self.connection.clone();
        scheduling_snapshot_write::write(&mut connection, self.scheduling_snapshot_key(), value, self.scheduling_snapshot_ttl_seconds).await?;
        Ok(snapshot)
    }

    async fn replace_scheduling_snapshot(&self) -> Result<SchedulingSnapshot, LlmProxyError> {
        self.clear_scheduling_snapshot().await?;
        self.write_fresh_scheduling_snapshot().await
    }

    async fn clear_scheduling_snapshot(&self) -> Result<(), LlmProxyError> {
        let mut connection = self.connection.clone();
        let _: i32 = connection.del(self.scheduling_snapshot_key()).await.map_err(redis_error)?;
        Ok(())
    }

    async fn wait_for_rebuild_lock(&self) -> Result<String, LlmProxyError> {
        let token = Uuid::now_v7().to_string();
        while !self.try_rebuild_lock(&token).await? {
            tokio::time::sleep(std::time::Duration::from_millis(SCHEDULING_REBUILD_WAIT_MS)).await;
        }
        Ok(token)
    }

    async fn try_rebuild_lock(&self, token: &str) -> Result<bool, LlmProxyError> {
        let mut connection = self.connection.clone();
        let result: Option<String> = redis::cmd("SET")
            .arg(self.scheduling_rebuild_lock_key())
            .arg(token)
            .arg("NX")
            .arg("EX")
            .arg(SCHEDULING_REBUILD_LOCK_SECONDS)
            .query_async(&mut connection)
            .await
            .map_err(redis_error)?;
        Ok(result.is_some())
    }

    async fn release_rebuild_lock(&self, token: &str) -> Result<(), LlmProxyError> {
        let mut connection = self.connection.clone();
        let _: i32 = redis::cmd("EVAL")
            .arg("if redis.call('GET', KEYS[1]) == ARGV[1] then return redis.call('DEL', KEYS[1]) else return 0 end")
            .arg(1)
            .arg(self.scheduling_rebuild_lock_key())
            .arg(token)
            .query_async(&mut connection)
            .await
            .map_err(redis_error)?;
        Ok(())
    }

    async fn release_locked_result<T>(&self, token: &str, result: Result<T, LlmProxyError>) -> Result<T, LlmProxyError> {
        let release = self.release_rebuild_lock(token).await;
        match (result, release) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), Ok(())) => Err(error),
            (Ok(_), Err(error)) => Err(error),
            (Err(error), Err(release_error)) => Err(LlmProxyError::Infrastructure(format!(
                "{error}; additionally failed to release proxy scheduling cache rebuild lock: {release_error}"
            ))),
        }
    }

    fn auth_version_key(&self) -> String {
        format!("{}:llm_proxy:auth:version", self.key_prefix)
    }

    fn auth_token_key(&self, version: i64, token_hash: &str) -> String {
        format!("{}:llm_proxy:auth:v{version}:{token_hash}", self.key_prefix)
    }

    fn auth_usage_key(&self, token_id: &str) -> String {
        format!("{}:llm_proxy:auth:usage:{token_id}", self.key_prefix)
    }

    fn scheduling_snapshot_key(&self) -> String {
        format!("{}:llm_proxy:scheduling:snapshot:v4", self.key_prefix)
    }

    fn scheduling_rebuild_lock_key(&self) -> String {
        format!("{}:llm_proxy:scheduling:rebuild_lock", self.key_prefix)
    }
}

fn redis_error(error: redis::RedisError) -> LlmProxyError {
    LlmProxyError::Infrastructure(format!("redis error: {error}"))
}

fn runtime_usage_time_error(error: time::error::Format) -> LlmProxyError {
    LlmProxyError::Infrastructure(format!("cached token usage timestamp error: {error}"))
}

fn usage_values(values: Vec<Option<String>>) -> Result<[Option<String>; 3], LlmProxyError> {
    values
        .try_into()
        .map_err(|_| LlmProxyError::Infrastructure("cached token usage hmget returned unexpected field count".into()))
}

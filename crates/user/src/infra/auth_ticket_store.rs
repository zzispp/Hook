use async_trait::async_trait;
use redis::AsyncCommands;

use crate::application::{AppError, AppResult, AuthTicketStore, OAuthPendingBinding, OAuthStateRecord, PurposeEmailCodeStore, WalletChallenge};

#[derive(Clone)]
pub struct RedisAuthTicketStore {
    connection: redis::aio::ConnectionManager,
    key_prefix: String,
}

#[derive(Clone)]
pub struct RedisPurposeEmailCodeStore {
    connection: redis::aio::ConnectionManager,
    key_prefix: String,
}

impl RedisAuthTicketStore {
    pub fn new(connection: redis::aio::ConnectionManager, key_prefix: String) -> Self {
        Self { connection, key_prefix }
    }

    fn key(&self, kind: &str, id: &str) -> String {
        format!("{}:auth:{kind}:{id}", self.key_prefix)
    }
}

impl RedisPurposeEmailCodeStore {
    pub fn new(connection: redis::aio::ConnectionManager, key_prefix: String) -> Self {
        Self { connection, key_prefix }
    }

    fn code_key(&self, purpose: &str, email: &str) -> String {
        format!("{}:auth:{purpose}:email_code:{email}", self.key_prefix)
    }

    fn cooldown_key(&self, purpose: &str, email: &str) -> String {
        format!("{}:auth:{purpose}:email_code_cooldown:{email}", self.key_prefix)
    }
}

#[async_trait]
impl AuthTicketStore for RedisAuthTicketStore {
    async fn save_oauth_state(&self, state: &str, record: OAuthStateRecord, ttl_seconds: u64) -> AppResult<()> {
        self.save_json("oauth_state", state, &record, ttl_seconds).await
    }

    async fn consume_oauth_state(&self, state: &str) -> AppResult<Option<OAuthStateRecord>> {
        self.consume_json("oauth_state", state).await
    }

    async fn save_oauth_binding(&self, ticket: &str, record: OAuthPendingBinding, ttl_seconds: u64) -> AppResult<()> {
        self.save_json("oauth_binding", ticket, &record, ttl_seconds).await
    }

    async fn consume_oauth_binding(&self, ticket: &str) -> AppResult<Option<OAuthPendingBinding>> {
        self.consume_json("oauth_binding", ticket).await
    }

    async fn save_wallet_challenge(&self, nonce: &str, record: WalletChallenge, ttl_seconds: u64) -> AppResult<()> {
        self.save_json("wallet_challenge", nonce, &record, ttl_seconds).await
    }

    async fn consume_wallet_challenge(&self, nonce: &str) -> AppResult<Option<WalletChallenge>> {
        self.consume_json("wallet_challenge", nonce).await
    }
}

#[async_trait]
impl PurposeEmailCodeStore for RedisPurposeEmailCodeStore {
    async fn active_email_code(&self, purpose: &str, email: &str) -> AppResult<Option<String>> {
        let mut connection = self.connection.clone();
        connection.get(self.code_key(purpose, email)).await.map_err(redis_error)
    }

    async fn save_email_code(&self, purpose: &str, email: &str, code: &str, ttl_seconds: u64) -> AppResult<()> {
        let mut connection = self.connection.clone();
        connection
            .set_ex::<_, _, ()>(self.code_key(purpose, email), code, ttl_seconds)
            .await
            .map_err(redis_error)
    }

    async fn begin_email_code_cooldown(&self, purpose: &str, email: &str, ttl_seconds: u64) -> AppResult<bool> {
        let mut connection = self.connection.clone();
        let result: Option<String> = redis::cmd("SET")
            .arg(self.cooldown_key(purpose, email))
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut connection)
            .await
            .map_err(redis_error)?;
        Ok(result.is_some())
    }

    async fn consume_email_code(&self, purpose: &str, email: &str, code: &str) -> AppResult<bool> {
        let mut connection = self.connection.clone();
        let consumed: i32 = redis::cmd("EVAL")
            .arg(
                r#"
                if redis.call("GET", KEYS[1]) == ARGV[1] then
                    redis.call("DEL", KEYS[1])
                    return 1
                end
                return 0
                "#,
            )
            .arg(1)
            .arg(self.code_key(purpose, email))
            .arg(code)
            .query_async(&mut connection)
            .await
            .map_err(redis_error)?;
        Ok(consumed == 1)
    }
}

impl RedisAuthTicketStore {
    async fn save_json<T>(&self, kind: &str, id: &str, record: &T, ttl_seconds: u64) -> AppResult<()>
    where
        T: serde::Serialize,
    {
        let mut connection = self.connection.clone();
        let value = serde_json::to_string(record).map_err(json_error)?;
        connection.set_ex::<_, _, ()>(self.key(kind, id), value, ttl_seconds).await.map_err(redis_error)
    }

    async fn consume_json<T>(&self, kind: &str, id: &str) -> AppResult<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let key = self.key(kind, id);
        let mut connection = self.connection.clone();
        let value: Option<String> = redis::cmd("GETDEL").arg(key).query_async(&mut connection).await.map_err(redis_error)?;
        value.map(|item| serde_json::from_str(&item).map_err(json_error)).transpose()
    }
}

fn redis_error(error: redis::RedisError) -> AppError {
    AppError::Infrastructure(format!("redis error: {error}"))
}

fn json_error(error: serde_json::Error) -> AppError {
    AppError::Infrastructure(format!("auth ticket serialization error: {error}"))
}

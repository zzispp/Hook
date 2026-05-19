use async_trait::async_trait;
use redis::AsyncCommands;

use crate::application::{AppError, AppResult, RegistrationEmailCodeStore};

#[derive(Clone)]
pub struct RedisRegistrationEmailCodeStore {
    connection: redis::aio::ConnectionManager,
    key_prefix: String,
}

impl RedisRegistrationEmailCodeStore {
    pub fn new(connection: redis::aio::ConnectionManager, key_prefix: String) -> Self {
        Self { connection, key_prefix }
    }

    fn code_key(&self, email: &str) -> String {
        format!("{}:auth:registration_email_code:{email}", self.key_prefix)
    }

    fn cooldown_key(&self, email: &str) -> String {
        format!("{}:auth:registration_email_code_cooldown:{email}", self.key_prefix)
    }
}

#[async_trait]
impl RegistrationEmailCodeStore for RedisRegistrationEmailCodeStore {
    async fn active_registration_email_code(&self, email: &str) -> AppResult<Option<String>> {
        let mut connection = self.connection.clone();
        connection.get(self.code_key(email)).await.map_err(redis_error)
    }

    async fn save_registration_email_code(&self, email: &str, code: &str, ttl_seconds: u64) -> AppResult<()> {
        let mut connection = self.connection.clone();
        connection
            .set_ex::<_, _, ()>(self.code_key(email), code, ttl_seconds)
            .await
            .map_err(redis_error)
    }

    async fn begin_registration_email_code_cooldown(&self, email: &str, ttl_seconds: u64) -> AppResult<bool> {
        let mut connection = self.connection.clone();
        let result: Option<String> = redis::cmd("SET")
            .arg(self.cooldown_key(email))
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut connection)
            .await
            .map_err(redis_error)?;
        Ok(result.is_some())
    }

    async fn consume_registration_email_code(&self, email: &str, code: &str) -> AppResult<bool> {
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
            .arg(self.code_key(email))
            .arg(code)
            .query_async(&mut connection)
            .await
            .map_err(redis_error)?;
        Ok(consumed == 1)
    }
}

fn redis_error(error: redis::RedisError) -> AppError {
    AppError::Infrastructure(format!("redis error: {error}"))
}

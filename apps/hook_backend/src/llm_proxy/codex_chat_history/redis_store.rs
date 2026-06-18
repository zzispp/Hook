use redis::AsyncCommands;

use super::types::{CachedResponse, CodexChatHistoryError};
use crate::llm_proxy::LlmProxyError;

#[derive(Clone, Debug)]
pub(super) struct RedisHistoryStore {
    connection: redis::aio::ConnectionManager,
    key_prefix: String,
    ttl_seconds: u64,
}

impl RedisHistoryStore {
    pub(super) fn new(connection: redis::aio::ConnectionManager, key_prefix: impl Into<String>, ttl_seconds: u64) -> Self {
        Self {
            connection,
            key_prefix: key_prefix.into(),
            ttl_seconds,
        }
    }

    pub(super) async fn read_response(&self, response_id: &str) -> Result<Option<CachedResponse>, LlmProxyError> {
        let mut connection = self.connection.clone();
        let value: Option<String> = connection.get(self.response_key(response_id)).await.map_err(redis_error)?;
        value.map(|json| serde_json::from_str(&json).map_err(json_error)).transpose()
    }

    pub(super) async fn write_response(&self, response_id: &str, response: &CachedResponse) -> Result<(), LlmProxyError> {
        let value = serde_json::to_string(response).map_err(json_error)?;
        let mut connection = self.connection.clone();
        let _: () = connection
            .set_ex(self.response_key(response_id), value, self.ttl_seconds)
            .await
            .map_err(redis_error)?;
        Ok(())
    }

    pub(super) async fn index_calls(&self, response_id: &str, call_ids: &[String]) -> Result<(), LlmProxyError> {
        let mut connection = self.connection.clone();
        for call_id in call_ids {
            let key = self.call_key(call_id);
            let _: () = redis::pipe()
                .cmd("SADD")
                .arg(&key)
                .arg(response_id)
                .ignore()
                .cmd("EXPIRE")
                .arg(&key)
                .arg(self.ttl_seconds)
                .ignore()
                .query_async(&mut connection)
                .await
                .map_err(redis_error)?;
        }
        Ok(())
    }

    pub(super) async fn call_response_ids(&self, call_id: &str) -> Result<Vec<String>, CodexChatHistoryError> {
        let mut connection = self.connection.clone();
        connection.smembers(self.call_key(call_id)).await.map_err(history_redis_error)
    }

    pub(super) async fn remove_stale_index_entry(&self, call_id: &str, response_id: &str) -> Result<(), CodexChatHistoryError> {
        let mut connection = self.connection.clone();
        let _: i64 = connection.srem(self.call_key(call_id), response_id).await.map_err(history_redis_error)?;
        Ok(())
    }

    pub(super) async fn next_response_seq(&self) -> Result<u64, LlmProxyError> {
        let mut connection = self.connection.clone();
        let key = self.seq_key();
        let seq = connection.incr(&key, 1).await.map_err(redis_error)?;
        expire_seq_key(&mut connection, key, self.ttl_seconds).await?;
        Ok(seq)
    }

    pub(super) async fn touch_seq_expiration(&self) -> Result<(), LlmProxyError> {
        let mut connection = self.connection.clone();
        expire_seq_key(&mut connection, self.seq_key(), self.ttl_seconds).await
    }

    fn response_key(&self, response_id: &str) -> String {
        format!("{}:llm_proxy:codex_chat_history:response:{response_id}", self.key_prefix)
    }

    fn call_key(&self, call_id: &str) -> String {
        format!("{}:llm_proxy:codex_chat_history:call:{call_id}", self.key_prefix)
    }

    fn seq_key(&self) -> String {
        format!("{}:llm_proxy:codex_chat_history:seq", self.key_prefix)
    }
}

async fn expire_seq_key(connection: &mut redis::aio::ConnectionManager, key: String, ttl_seconds: u64) -> Result<(), LlmProxyError> {
    let _: bool = connection.expire(key, ttl_seconds as i64).await.map_err(redis_error)?;
    Ok(())
}

pub(super) fn history_infrastructure_error(error: LlmProxyError) -> CodexChatHistoryError {
    CodexChatHistoryError::Infrastructure(error.to_string())
}

fn history_redis_error(error: redis::RedisError) -> CodexChatHistoryError {
    CodexChatHistoryError::Infrastructure(format!("redis error: {error}"))
}

fn redis_error(error: redis::RedisError) -> LlmProxyError {
    LlmProxyError::Infrastructure(format!("redis error: {error}"))
}

fn json_error(error: serde_json::Error) -> LlmProxyError {
    LlmProxyError::Infrastructure(error.to_string())
}

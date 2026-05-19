use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::LlmProxyError;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AffinityRecord {
    pub provider_id: String,
    pub endpoint_id: String,
    pub key_id: String,
    pub api_format: String,
    pub model_id: String,
    pub created_at: i64,
    pub expire_at: i64,
    pub request_count: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AffinitySelection {
    pub provider_id: String,
    pub endpoint_id: String,
    pub key_id: String,
}

#[derive(Clone, Copy, Debug)]
pub struct SetAffinityInput<'a> {
    pub token_id: &'a str,
    pub model_id: &'a str,
    pub api_format: &'a str,
    pub provider_id: &'a str,
    pub endpoint_id: &'a str,
    pub key_id: &'a str,
    pub ttl_minutes: i64,
}

#[derive(Clone, Copy, Debug)]
pub struct InvalidateAffinityInput<'a> {
    pub token_id: &'a str,
    pub model_id: &'a str,
    pub api_format: &'a str,
    pub provider_id: &'a str,
    pub endpoint_id: &'a str,
    pub key_id: &'a str,
}

pub struct CacheAffinityStore {
    connection: redis::aio::ConnectionManager,
    key_prefix: String,
}

impl CacheAffinityStore {
    pub fn new(connection: redis::aio::ConnectionManager, key_prefix: &str) -> Self {
        Self {
            connection,
            key_prefix: key_prefix.to_owned(),
        }
    }

    pub async fn get(&self, token_id: &str, model_id: &str, api_format: &str) -> Result<Option<AffinityRecord>, LlmProxyError> {
        let mut connection = self.connection.clone();
        let Some(value): Option<String> = connection.get(self.cache_key(token_id, model_id, api_format)).await.map_err(redis_error)? else {
            return Ok(None);
        };
        let record = serde_json::from_str(&value).map_err(|error| LlmProxyError::Infrastructure(error.to_string()))?;
        Ok(Some(record))
    }

    pub async fn set(&self, input: SetAffinityInput<'_>) -> Result<(), LlmProxyError> {
        if input.ttl_minutes <= 0 {
            return Ok(());
        }
        let seconds = input.ttl_minutes as u64 * 60;
        let key = self.cache_key(input.token_id, input.model_id, input.api_format);
        let existing = self.get(input.token_id, input.model_id, input.api_format).await?;
        let value = serde_json::to_string(&next_record(input, existing.as_ref())).map_err(|error| LlmProxyError::Infrastructure(error.to_string()))?;
        let mut connection = self.connection.clone();
        connection.set_ex(key, value, seconds).await.map_err(redis_error)
    }

    pub async fn invalidate(&self, input: InvalidateAffinityInput<'_>) -> Result<(), LlmProxyError> {
        let Some(record) = self.get(input.token_id, input.model_id, input.api_format).await? else {
            return Ok(());
        };
        if !matches_input(&record, &input) {
            return Ok(());
        }
        let mut connection = self.connection.clone();
        connection
            .del(self.cache_key(input.token_id, input.model_id, input.api_format))
            .await
            .map_err(redis_error)
    }

    fn cache_key(&self, token_id: &str, model_id: &str, api_format: &str) -> String {
        format!("{}:llm_proxy:cache_affinity:{token_id}:{api_format}:{model_id}", self.key_prefix)
    }
}

impl From<&AffinityRecord> for AffinitySelection {
    fn from(record: &AffinityRecord) -> Self {
        Self {
            provider_id: record.provider_id.clone(),
            endpoint_id: record.endpoint_id.clone(),
            key_id: record.key_id.clone(),
        }
    }
}

fn next_record(input: SetAffinityInput<'_>, existing: Option<&AffinityRecord>) -> AffinityRecord {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let request_count = existing
        .filter(|record| same_target(record, &input))
        .map_or(1, |record| record.request_count + 1);
    let created_at = existing.filter(|record| same_target(record, &input)).map_or(now, |record| record.created_at);
    AffinityRecord {
        provider_id: input.provider_id.to_owned(),
        endpoint_id: input.endpoint_id.to_owned(),
        key_id: input.key_id.to_owned(),
        api_format: input.api_format.to_owned(),
        model_id: input.model_id.to_owned(),
        created_at,
        expire_at: now + input.ttl_minutes * 60,
        request_count,
    }
}

fn same_target(record: &AffinityRecord, input: &SetAffinityInput<'_>) -> bool {
    record.provider_id == input.provider_id && record.endpoint_id == input.endpoint_id && record.key_id == input.key_id
}

fn matches_input(record: &AffinityRecord, input: &InvalidateAffinityInput<'_>) -> bool {
    record.provider_id == input.provider_id && record.endpoint_id == input.endpoint_id && record.key_id == input.key_id
}

fn redis_error(error: redis::RedisError) -> LlmProxyError {
    LlmProxyError::Infrastructure(error.to_string())
}

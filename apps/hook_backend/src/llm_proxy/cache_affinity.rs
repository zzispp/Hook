use futures_util::TryStreamExt;
use redis::{AsyncCommands, AsyncIter};
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

#[derive(Clone, Copy, Debug)]
pub struct ClearAffinityInput<'a> {
    pub token_id: &'a str,
    pub model_id: &'a str,
    pub api_format: &'a str,
    pub endpoint_id: &'a str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AffinityEntry {
    pub token_id: String,
    pub record: AffinityRecord,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CacheKeyParts {
    token_id: String,
    api_format: String,
    model_id: String,
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

    pub async fn list(&self) -> Result<Vec<AffinityEntry>, LlmProxyError> {
        let mut connection = self.connection.clone();
        let keys = scan_keys(&mut connection, &self.cache_pattern()).await?;
        let mut entries = Vec::with_capacity(keys.len());
        for key in keys {
            let value: String = connection.get(&key).await.map_err(redis_error)?;
            let record = serde_json::from_str(&value).map_err(|error| LlmProxyError::Infrastructure(error.to_string()))?;
            let parts = parse_cache_key(&self.key_prefix, &key)?;
            entries.push(AffinityEntry {
                token_id: parts.token_id,
                record,
            });
        }
        Ok(entries)
    }

    pub async fn clear_all(&self) -> Result<u64, LlmProxyError> {
        let mut connection = self.connection.clone();
        let keys = scan_keys(&mut connection, &self.cache_pattern()).await?;
        if keys.is_empty() {
            return Ok(0);
        }
        connection.del(keys).await.map_err(redis_error)
    }

    fn cache_key(&self, token_id: &str, model_id: &str, api_format: &str) -> String {
        format!("{}:llm_proxy:cache_affinity:{token_id}:{api_format}:{model_id}", self.key_prefix)
    }

    fn cache_pattern(&self) -> String {
        format!("{}:llm_proxy:cache_affinity:*", self.key_prefix)
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

async fn scan_keys(connection: &mut redis::aio::ConnectionManager, pattern: &str) -> Result<Vec<String>, LlmProxyError> {
    let iter: AsyncIter<String> = connection.scan_match(pattern).await.map_err(redis_error)?;
    iter.try_collect().await.map_err(redis_error)
}

fn parse_cache_key(key_prefix: &str, key: &str) -> Result<CacheKeyParts, LlmProxyError> {
    let prefix = format!("{key_prefix}:llm_proxy:cache_affinity:");
    let suffix = key
        .strip_prefix(&prefix)
        .ok_or_else(|| LlmProxyError::Infrastructure(format!("invalid cache affinity key: {key}")))?;
    let mut parts = suffix.splitn(2, ':');
    let token_id = required_cache_key_part(parts.next(), key)?;
    let tail = required_cache_key_part(parts.next(), key)?;
    let (api_format, model_id) = parse_api_format_and_model(&tail, key)?;
    Ok(CacheKeyParts {
        token_id,
        api_format,
        model_id,
    })
}

fn parse_api_format_and_model(value: &str, key: &str) -> Result<(String, String), LlmProxyError> {
    for format in CANONICAL_COLON_FORMATS {
        let prefix = format!("{format}:");
        if let Some(model_id) = value.strip_prefix(&prefix) {
            return Ok(((*format).to_owned(), required_cache_key_part(Some(model_id), key)?));
        }
    }
    let mut parts = value.splitn(2, ':');
    let api_format = required_cache_key_part(parts.next(), key)?;
    let model_id = required_cache_key_part(parts.next(), key)?;
    Ok((api_format, model_id))
}

const CANONICAL_COLON_FORMATS: &[&str] = &[
    "openai:chat",
    "openai:cli",
    "openai:compact",
    "claude:chat",
    "claude:cli",
    "gemini:chat",
    "gemini:cli",
];

fn required_cache_key_part(value: Option<&str>, key: &str) -> Result<String, LlmProxyError> {
    let part = value
        .filter(|item| !item.is_empty())
        .ok_or_else(|| LlmProxyError::Infrastructure(format!("invalid cache affinity key: {key}")))?;
    Ok(part.to_owned())
}

fn redis_error(error: redis::RedisError) -> LlmProxyError {
    LlmProxyError::Infrastructure(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{AffinityRecord, ClearAffinityInput, parse_cache_key};

    #[test]
    fn parse_cache_key_reads_affinity_segments() {
        let parts = parse_cache_key("hook", "hook:llm_proxy:cache_affinity:token-1:openai:chat:model:demo").unwrap();

        assert_eq!(parts.token_id, "token-1");
        assert_eq!(parts.api_format, "openai:chat");
        assert_eq!(parts.model_id, "model:demo");
    }

    #[test]
    fn parse_cache_key_rejects_invalid_prefix() {
        let error = parse_cache_key("hook", "bad:cache:key").unwrap_err();

        assert!(error.to_string().contains("invalid cache affinity key"));
    }

    #[test]
    fn affinity_delete_target_matches_endpoint_only() {
        let record = AffinityRecord {
            provider_id: "provider-1".into(),
            endpoint_id: "endpoint-1".into(),
            key_id: "key-1".into(),
            api_format: "openai:chat".into(),
            model_id: "model-1".into(),
            created_at: 0,
            expire_at: 60,
            request_count: 3,
        };
        let target = ClearAffinityInput {
            token_id: "token-1",
            model_id: "model-1",
            api_format: "openai:chat",
            endpoint_id: "endpoint-1",
        };

        assert_eq!(record.endpoint_id, target.endpoint_id);
        assert_ne!(record.endpoint_id, "endpoint-2");
    }
}

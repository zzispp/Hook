use std::collections::HashSet;

use redis::AsyncCommands;
use storage::{Database, provider::ProviderCooldownRecordInput};
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};
use types::provider::{ProviderCooldown, ProviderCooldownPolicy, ProviderCooldownRule};

use super::{LlmProxyCache, redis_error};
use crate::llm_proxy::{LlmProxyError, candidate::ProxyCandidate};

#[derive(Clone, Debug)]
pub struct ProviderCooldownFailureInput<'a> {
    pub request_id: &'a str,
    pub candidate: &'a ProxyCandidate,
    pub retry_index: i32,
    pub status_code: i32,
    pub error_type: &'a str,
    pub error_message: &'a str,
    pub error_code: Option<&'a str>,
    pub error_param: Option<&'a str>,
}

impl LlmProxyCache {
    pub async fn clear_provider_cooldown(&self, provider_id: &str) -> Result<(), LlmProxyError> {
        let mut connection = self.connection.clone();
        let _: i32 = connection.del(self.provider_cooldown_key(provider_id)).await.map_err(redis_error)?;
        Ok(())
    }

    pub async fn cooled_provider_ids(&self, provider_ids: &[String]) -> Result<HashSet<String>, LlmProxyError> {
        if provider_ids.is_empty() {
            return Ok(HashSet::new());
        }
        let keys = provider_ids.iter().map(|id| self.provider_cooldown_key(id)).collect::<Vec<_>>();
        let mut connection = self.connection.clone();
        let values: Vec<Option<String>> = redis::cmd("MGET").arg(keys).query_async(&mut connection).await.map_err(redis_error)?;
        Ok(provider_ids.iter().zip(values).filter_map(|(id, value)| value.map(|_| id.clone())).collect())
    }

    pub async fn restore_provider_cooldowns(&self) -> Result<(), LlmProxyError> {
        let cooldowns = storage::provider::ProviderStore::new(self.database.clone())
            .active_provider_cooldowns_for_restore()
            .await?;
        for cooldown in cooldowns {
            self.restore_provider_cooldown(cooldown).await?;
        }
        Ok(())
    }

    pub async fn record_provider_status_failure(&self, input: ProviderCooldownFailureInput<'_>) -> Result<bool, LlmProxyError> {
        let snapshot = self.scheduling_snapshot().await?;
        let Some(rule) = matching_rule(&snapshot.provider_cooldown_policy, input.status_code) else {
            return Ok(false);
        };
        let now = OffsetDateTime::now_utc();
        let observed = self
            .record_failure_event(
                &input.candidate.trace.provider_id,
                input.status_code,
                snapshot.provider_cooldown_policy.window_seconds,
                now,
            )
            .await?;
        if observed < rule.failure_count {
            return Ok(false);
        }
        let cooldown_until = now + Duration::seconds(rule.cooldown_seconds);
        self.write_provider_cooldown(&input.candidate.trace.provider_id, rule.cooldown_seconds).await?;
        upsert_cooldown_record(
            &self.database,
            input,
            rule,
            snapshot.provider_cooldown_policy.window_seconds,
            observed,
            now,
            cooldown_until,
        )
        .await?;
        Ok(true)
    }

    async fn restore_provider_cooldown(&self, cooldown: ProviderCooldown) -> Result<(), LlmProxyError> {
        let now = OffsetDateTime::now_utc();
        let until = parse_timestamp(&cooldown.cooldown_until)?;
        let Some(seconds) = ttl_seconds(until, now) else {
            return Ok(());
        };
        self.write_provider_cooldown(&cooldown.provider_id, seconds).await
    }

    async fn record_failure_event(&self, provider_id: &str, status_code: i32, window_seconds: i64, now: OffsetDateTime) -> Result<i64, LlmProxyError> {
        let key = self.provider_cooldown_failure_key(provider_id, status_code);
        let min_score = now.unix_timestamp() - window_seconds + 1;
        let member = format!("{}:{}", now.unix_timestamp_nanos(), uuid::Uuid::now_v7());
        let mut connection = self.connection.clone();
        let (observed,): (i64,) = redis::pipe()
            .cmd("ZADD")
            .arg(&key)
            .arg(now.unix_timestamp())
            .arg(member)
            .ignore()
            .cmd("ZREMRANGEBYSCORE")
            .arg(&key)
            .arg("-inf")
            .arg(min_score - 1)
            .ignore()
            .cmd("EXPIRE")
            .arg(&key)
            .arg(window_seconds * 2)
            .ignore()
            .cmd("ZCARD")
            .arg(&key)
            .query_async(&mut connection)
            .await
            .map_err(redis_error)?;
        Ok(observed)
    }

    async fn write_provider_cooldown(&self, provider_id: &str, seconds: i64) -> Result<(), LlmProxyError> {
        let ttl = u64::try_from(seconds).map_err(|_| LlmProxyError::Infrastructure("provider cooldown ttl exceeds Redis range".into()))?;
        let mut connection = self.connection.clone();
        let _: () = connection
            .set_ex(self.provider_cooldown_key(provider_id), "1", ttl)
            .await
            .map_err(redis_error)?;
        Ok(())
    }

    fn provider_cooldown_key(&self, provider_id: &str) -> String {
        format!("{}:llm_proxy:provider_cooldown:{provider_id}", self.key_prefix)
    }

    fn provider_cooldown_failure_key(&self, provider_id: &str, status_code: i32) -> String {
        format!("{}:llm_proxy:provider_cooldown_failures:{provider_id}:{status_code}", self.key_prefix)
    }
}

fn matching_rule(policy: &ProviderCooldownPolicy, status_code: i32) -> Option<&ProviderCooldownRule> {
    if policy.window_seconds <= 0 {
        return None;
    }
    policy.rules.iter().find(|rule| rule.status_code == status_code)
}

async fn upsert_cooldown_record(
    database: &Database,
    input: ProviderCooldownFailureInput<'_>,
    rule: &ProviderCooldownRule,
    window_seconds: i64,
    observed: i64,
    triggered_at: OffsetDateTime,
    cooldown_until: OffsetDateTime,
) -> Result<(), LlmProxyError> {
    let store = storage::provider::ProviderStore::new(database.clone());
    let record = record_input(input, rule, window_seconds, observed, triggered_at, cooldown_until);
    store.upsert_provider_cooldown(record.clone()).await?;
    store.create_provider_cooldown_event(record).await?;
    Ok(())
}

fn record_input(
    input: ProviderCooldownFailureInput<'_>,
    rule: &ProviderCooldownRule,
    window_seconds: i64,
    observed: i64,
    triggered_at: OffsetDateTime,
    cooldown_until: OffsetDateTime,
) -> ProviderCooldownRecordInput {
    let trace = &input.candidate.trace;
    ProviderCooldownRecordInput {
        provider_id: trace.provider_id.clone(),
        provider_name_snapshot: trace.provider_name_snapshot.clone(),
        status_code: input.status_code,
        observed_count: observed,
        threshold_count: rule.failure_count,
        window_seconds,
        cooldown_seconds: rule.cooldown_seconds,
        triggered_at,
        cooldown_until,
        request_id: input.request_id.to_owned(),
        candidate_index: trace.candidate_index,
        retry_index: input.retry_index,
        endpoint_id: Some(trace.endpoint_id.clone()),
        endpoint_name_snapshot: Some(trace.endpoint_name_snapshot.clone()),
        key_id: Some(trace.key_id.clone()),
        key_name_snapshot: Some(trace.key_name_snapshot.clone()),
        error_type: Some(input.error_type.to_owned()),
        error_message: Some(input.error_message.to_owned()),
        error_code: input.error_code.map(str::to_owned),
        error_param: input.error_param.map(str::to_owned),
    }
}

fn parse_timestamp(value: &str) -> Result<OffsetDateTime, LlmProxyError> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|error| LlmProxyError::Infrastructure(format!("invalid provider cooldown timestamp: {error}")))
}

fn ttl_seconds(until: OffsetDateTime, now: OffsetDateTime) -> Option<i64> {
    let seconds = (until - now).whole_seconds();
    (seconds > 0).then_some(seconds)
}

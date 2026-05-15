mod codec;
mod keys;

use std::{collections::HashMap, time::Duration};

use redis::AsyncCommands;
use storage::{
    api_token::{ApiTokenStore, ApiTokenUsageRecord},
    model::{GlobalModelUsageRecord, ModelStore},
    usage_flush::UsageFlushStore,
};
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

use self::codec::{decode_model_usage_batch, decode_token_usage_batch, token_cost_units};
use super::{LlmProxyCache, LlmProxyError};

const USAGE_FLUSH_INTERVAL: Duration = Duration::from_secs(1);
const USAGE_FLUSH_LOCK_SECONDS: u64 = 30;
const TOKEN_ENQUEUE_LUA: &str = r#"redis.call('HINCRBY', KEYS[1], ARGV[1], ARGV[2]) redis.call('HINCRBY', KEYS[2], ARGV[1], ARGV[3]) local current = redis.call('HGET', KEYS[3], ARGV[1]) if (not current) or (ARGV[4] > current) then redis.call('HSET', KEYS[3], ARGV[1], ARGV[4]) end return 1"#;
const TOKEN_MOVE_LUA: &str = r#"redis.call('DEL', KEYS[4], KEYS[5], KEYS[6], KEYS[7]) local moved = 0 if redis.call('EXISTS', KEYS[1]) == 1 then redis.call('RENAME', KEYS[1], KEYS[4]) moved = 1 end if redis.call('EXISTS', KEYS[2]) == 1 then redis.call('RENAME', KEYS[2], KEYS[5]) moved = 1 end if redis.call('EXISTS', KEYS[3]) == 1 then redis.call('RENAME', KEYS[3], KEYS[6]) moved = 1 end if moved == 1 then redis.call('SET', KEYS[7], ARGV[1]) end return moved"#;
const MODEL_MOVE_LUA: &str = r#"redis.call('DEL', KEYS[2], KEYS[3]) if redis.call('EXISTS', KEYS[1]) == 1 then redis.call('RENAME', KEYS[1], KEYS[2]) redis.call('SET', KEYS[3], ARGV[1]) return 1 end return 0"#;
const LOCK_RELEASE_LUA: &str = r#"if redis.call('GET', KEYS[1]) == ARGV[1] then return redis.call('DEL', KEYS[1]) end return 0"#;

#[derive(Default)]
struct FlushReport {
    token_records: usize,
    model_records: usize,
}

impl FlushReport {
    fn is_empty(&self) -> bool {
        self.token_records == 0 && self.model_records == 0
    }

    fn add(&mut self, other: Self) {
        self.token_records += other.token_records;
        self.model_records += other.model_records;
    }
}

struct ProcessingBatch<T> {
    id: String,
    records: Vec<T>,
}

pub(super) fn spawn_usage_flush_task(cache: LlmProxyCache) {
    tokio::spawn(async move {
        flush_loop(cache).await;
    });
}

impl LlmProxyCache {
    pub async fn enqueue_token_usage_persist(&self, record: &ApiTokenUsageRecord) -> Result<(), LlmProxyError> {
        let used_at = record.used_at.format(&Rfc3339).map_err(timestamp_format_error)?;
        let mut connection = self.connection.clone();
        let _: i32 = redis::cmd("EVAL")
            .arg(TOKEN_ENQUEUE_LUA)
            .arg(3)
            .arg(self.pending_token_cost_key())
            .arg(self.pending_token_count_key())
            .arg(self.pending_token_last_used_at_key())
            .arg(record.token_id.as_str())
            .arg(token_cost_units(record.cost)?)
            .arg(record.request_count)
            .arg(used_at)
            .query_async(&mut connection)
            .await
            .map_err(redis_error)?;
        Ok(())
    }

    pub async fn enqueue_model_usage_persist(&self, record: &GlobalModelUsageRecord) -> Result<(), LlmProxyError> {
        let mut connection = self.connection.clone();
        let _: i64 = connection
            .hincr(self.pending_model_count_key(), record.model_id.as_str(), record.count)
            .await
            .map_err(redis_error)?;
        Ok(())
    }

    async fn flush_usage_once(&self) -> Result<FlushReport, LlmProxyError> {
        let Some(token) = self.try_usage_flush_lock().await? else {
            return Ok(FlushReport::default());
        };
        let result = self.flush_usage_batches().await;
        self.release_flush_result(&token, result).await
    }

    async fn flush_usage_batches(&self) -> Result<FlushReport, LlmProxyError> {
        let mut report = self.flush_processing_usage().await?;
        self.drain_pending_usage().await?;
        report.add(self.flush_processing_usage().await?);
        Ok(report)
    }

    async fn flush_processing_usage(&self) -> Result<FlushReport, LlmProxyError> {
        let token = self.flush_token_usage_batch().await;
        let model = self.flush_model_usage_batch().await;
        merge_flush_outcome(token, model)
    }

    async fn flush_token_usage_batch(&self) -> Result<usize, LlmProxyError> {
        let Some(batch) = self.read_token_usage_batch().await? else {
            return Ok(0);
        };
        let store = ApiTokenStore::new(self.database.clone());
        store.record_usage_batch_once(&batch.id, &batch.records).await?;
        self.clear_token_processing_usage().await?;
        self.delete_usage_flush_batch(&batch.id).await?;
        Ok(batch.records.len())
    }

    async fn flush_model_usage_batch(&self) -> Result<usize, LlmProxyError> {
        let Some(batch) = self.read_model_usage_batch().await? else {
            return Ok(0);
        };
        let store = ModelStore::new(self.database.clone());
        store.record_usage_batch_once(&batch.id, &batch.records).await?;
        self.clear_model_processing_usage().await?;
        self.delete_usage_flush_batch(&batch.id).await?;
        Ok(batch.records.len())
    }

    async fn drain_pending_usage(&self) -> Result<(), LlmProxyError> {
        self.move_token_usage(self.pending_token_keys(), self.processing_token_keys()).await?;
        self.move_model_usage(self.pending_model_count_key(), self.processing_model_count_key()).await
    }

    async fn try_usage_flush_lock(&self) -> Result<Option<String>, LlmProxyError> {
        let token = Uuid::now_v7().to_string();
        let mut connection = self.connection.clone();
        let result: Option<String> = redis::cmd("SET")
            .arg(self.usage_flush_lock_key())
            .arg(token.as_str())
            .arg("NX")
            .arg("EX")
            .arg(USAGE_FLUSH_LOCK_SECONDS)
            .query_async(&mut connection)
            .await
            .map_err(redis_error)?;
        Ok(result.map(|_| token))
    }

    async fn release_flush_result<T>(&self, token: &str, result: Result<T, LlmProxyError>) -> Result<T, LlmProxyError> {
        let release = self.release_usage_flush_lock(token).await;
        match (result, release) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), Ok(())) => Err(error),
            (Ok(_), Err(error)) => Err(error),
            (Err(error), Err(release_error)) => Err(LlmProxyError::Infrastructure(format!(
                "{error}; additionally failed to release usage flush lock: {release_error}"
            ))),
        }
    }

    async fn release_usage_flush_lock(&self, token: &str) -> Result<(), LlmProxyError> {
        let mut connection = self.connection.clone();
        let _: i32 = redis::cmd("EVAL")
            .arg(LOCK_RELEASE_LUA)
            .arg(1)
            .arg(self.usage_flush_lock_key())
            .arg(token)
            .query_async(&mut connection)
            .await
            .map_err(redis_error)?;
        Ok(())
    }

    async fn read_token_usage_batch(&self) -> Result<Option<ProcessingBatch<ApiTokenUsageRecord>>, LlmProxyError> {
        let mut connection = self.connection.clone();
        let id: Option<String> = connection.get(self.processing_token_batch_id_key()).await.map_err(redis_error)?;
        let cost: HashMap<String, String> = connection.hgetall(self.processing_token_cost_key()).await.map_err(redis_error)?;
        let count: HashMap<String, String> = connection.hgetall(self.processing_token_count_key()).await.map_err(redis_error)?;
        let last_used_at: HashMap<String, String> = connection.hgetall(self.processing_token_last_used_at_key()).await.map_err(redis_error)?;
        processing_batch(id, decode_token_usage_batch(cost, count, last_used_at)?, "token")
    }

    async fn read_model_usage_batch(&self) -> Result<Option<ProcessingBatch<GlobalModelUsageRecord>>, LlmProxyError> {
        let mut connection = self.connection.clone();
        let id: Option<String> = connection.get(self.processing_model_batch_id_key()).await.map_err(redis_error)?;
        let counts: HashMap<String, String> = connection.hgetall(self.processing_model_count_key()).await.map_err(redis_error)?;
        processing_batch(id, decode_model_usage_batch(counts)?, "model")
    }

    async fn clear_token_processing_usage(&self) -> Result<(), LlmProxyError> {
        let mut connection = self.connection.clone();
        let _: i32 = connection
            .del(&[
                self.processing_token_cost_key(),
                self.processing_token_count_key(),
                self.processing_token_last_used_at_key(),
                self.processing_token_batch_id_key(),
            ])
            .await
            .map_err(redis_error)?;
        Ok(())
    }

    async fn clear_model_processing_usage(&self) -> Result<(), LlmProxyError> {
        let mut connection = self.connection.clone();
        let _: i32 = connection
            .del(&[self.processing_model_count_key(), self.processing_model_batch_id_key()])
            .await
            .map_err(redis_error)?;
        Ok(())
    }

    async fn move_token_usage(&self, source: [String; 3], target: [String; 3]) -> Result<(), LlmProxyError> {
        let mut connection = self.connection.clone();
        let batch_id = Uuid::now_v7().to_string();
        let _: i32 = redis::cmd("EVAL")
            .arg(TOKEN_MOVE_LUA)
            .arg(7)
            .arg(source[0].as_str())
            .arg(source[1].as_str())
            .arg(source[2].as_str())
            .arg(target[0].as_str())
            .arg(target[1].as_str())
            .arg(target[2].as_str())
            .arg(self.processing_token_batch_id_key())
            .arg(batch_id)
            .query_async(&mut connection)
            .await
            .map_err(redis_error)?;
        Ok(())
    }

    async fn move_model_usage(&self, source: String, target: String) -> Result<(), LlmProxyError> {
        let mut connection = self.connection.clone();
        let batch_id = Uuid::now_v7().to_string();
        let _: i32 = redis::cmd("EVAL")
            .arg(MODEL_MOVE_LUA)
            .arg(3)
            .arg(source)
            .arg(target)
            .arg(self.processing_model_batch_id_key())
            .arg(batch_id)
            .query_async(&mut connection)
            .await
            .map_err(redis_error)?;
        Ok(())
    }

    async fn delete_usage_flush_batch(&self, batch_id: &str) -> Result<(), LlmProxyError> {
        UsageFlushStore::new(self.database.clone()).delete_batch(batch_id).await?;
        Ok(())
    }
}

async fn flush_loop(cache: LlmProxyCache) {
    let mut interval = tokio::time::interval(USAGE_FLUSH_INTERVAL);
    loop {
        interval.tick().await;
        match cache.flush_usage_once().await {
            Ok(report) if !report.is_empty() => hook_tracing::info_with_fields!(
                "llm proxy usage flush completed",
                token_records = report.token_records,
                model_records = report.model_records,
            ),
            Ok(_) => {}
            Err(error) => hook_tracing::error("llm proxy usage flush failed", &error),
        }
    }
}

fn merge_flush_outcome(token: Result<usize, LlmProxyError>, model: Result<usize, LlmProxyError>) -> Result<FlushReport, LlmProxyError> {
    match (token, model) {
        (Ok(token_records), Ok(model_records)) => Ok(FlushReport { token_records, model_records }),
        (Err(error), Ok(_)) | (Ok(_), Err(error)) => Err(error),
        (Err(token_error), Err(model_error)) => Err(LlmProxyError::Infrastructure(format!(
            "token usage flush failed: {token_error}; model usage flush failed: {model_error}"
        ))),
    }
}

fn processing_batch<T>(id: Option<String>, records: Vec<T>, label: &str) -> Result<Option<ProcessingBatch<T>>, LlmProxyError> {
    if records.is_empty() && id.is_none() {
        return Ok(None);
    }
    if records.is_empty() {
        return Err(LlmProxyError::Infrastructure(format!("{label} usage processing records are missing")));
    }
    let id = id.ok_or_else(|| LlmProxyError::Infrastructure(format!("{label} usage processing batch id is missing")))?;
    Ok(Some(ProcessingBatch { id, records }))
}

fn redis_error(error: redis::RedisError) -> LlmProxyError {
    LlmProxyError::Infrastructure(format!("redis error: {error}"))
}

fn timestamp_format_error(error: time::error::Format) -> LlmProxyError {
    LlmProxyError::Infrastructure(format!("usage flush timestamp error: {error}"))
}

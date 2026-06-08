use std::time::Duration;

use time::OffsetDateTime;

use crate::{StorageError, StorageResult};

use super::{
    repository::ProviderStore, request_record_housekeeping_delete, request_record_housekeeping_payload, request_record_housekeeping_timeout::CleanupBudget,
};

#[derive(Clone, Copy, Debug)]
pub struct RequestRecordCleanupOptions {
    pub record_cutoff: OffsetDateTime,
    pub payload_cutoff: OffsetDateTime,
    pub delete_batch_size: u64,
    pub compress_batch_size: u64,
    pub max_runtime: Duration,
    pub batch_sleep: Duration,
    pub statement_timeout_seconds: i64,
    pub lock_timeout_seconds: i64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RequestRecordCleanupResult {
    pub deleted_records: u64,
    pub deleted_candidates: u64,
    pub compressed_records: u64,
    pub compressed_candidates: u64,
    pub time_budget_exhausted: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct CompressBatchResult {
    pub scanned: u64,
    pub changed: u64,
    pub time_budget_exhausted: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct CleanupBatch {
    deleted_records: u64,
    deleted_candidates: u64,
    compressed_records: u64,
    compressed_candidates: u64,
    scanned_records: u64,
    scanned_candidates: u64,
    time_budget_exhausted: bool,
}

pub async fn cleanup_request_records(store: &ProviderStore, options: RequestRecordCleanupOptions) -> StorageResult<RequestRecordCleanupResult> {
    validate_options(&options)?;
    let budget = CleanupBudget::start(options.max_runtime);
    let mut result = RequestRecordCleanupResult::default();
    loop {
        if budget.exhausted() {
            result.time_budget_exhausted = true;
            return Ok(result);
        }
        let batch = cleanup_batch(store, &options, &budget).await?;
        result.add(batch);
        if batch.time_budget_exhausted {
            result.time_budget_exhausted = true;
            return Ok(result);
        }
        if batch.is_empty() {
            return Ok(result);
        }
        if sleep_between_batches(options.batch_sleep, &budget).await {
            result.time_budget_exhausted = true;
            return Ok(result);
        }
    }
}

async fn cleanup_batch(store: &ProviderStore, options: &RequestRecordCleanupOptions, budget: &CleanupBudget) -> StorageResult<CleanupBatch> {
    let deleted_records = request_record_housekeeping_delete::delete_record_batch(store, options, budget).await?;
    let deleted_orphans = request_record_housekeeping_delete::delete_orphan_candidate_batch(store, options, budget).await?;
    let compressed_records = request_record_housekeeping_payload::compress_record_batch(store, options, budget).await?;
    let compressed_candidates = request_record_housekeeping_payload::compress_candidate_batch(store, options, budget).await?;
    Ok(CleanupBatch {
        deleted_records: deleted_records.deleted_records,
        deleted_candidates: deleted_records.deleted_candidates + deleted_orphans,
        compressed_records: compressed_records.changed,
        compressed_candidates: compressed_candidates.changed,
        scanned_records: compressed_records.scanned,
        scanned_candidates: compressed_candidates.scanned,
        time_budget_exhausted: compressed_records.time_budget_exhausted || compressed_candidates.time_budget_exhausted,
    })
}

fn validate_options(options: &RequestRecordCleanupOptions) -> StorageResult<()> {
    positive_u64("delete_batch_size", options.delete_batch_size)?;
    positive_u64("compress_batch_size", options.compress_batch_size)?;
    positive_duration("max_runtime", options.max_runtime)?;
    positive_i64("statement_timeout_seconds", options.statement_timeout_seconds)?;
    positive_i64("lock_timeout_seconds", options.lock_timeout_seconds)
}

fn positive_u64(name: &str, value: u64) -> StorageResult<()> {
    if value == 0 || value > i64::MAX as u64 {
        return Err(StorageError::Database(format!("{name} must be between 1 and {}", i64::MAX)));
    }
    Ok(())
}

fn positive_i64(name: &str, value: i64) -> StorageResult<()> {
    if value <= 0 {
        return Err(StorageError::Database(format!("{name} must be greater than 0")));
    }
    Ok(())
}

fn positive_duration(name: &str, value: Duration) -> StorageResult<()> {
    if value.is_zero() {
        return Err(StorageError::Database(format!("{name} must be greater than 0")));
    }
    Ok(())
}

async fn sleep_between_batches(duration: Duration, budget: &CleanupBudget) -> bool {
    if !duration.is_zero() {
        let Some(remaining) = budget.remaining() else {
            return true;
        };
        if remaining <= duration {
            tokio::time::sleep(remaining).await;
            return true;
        }
        tokio::time::sleep(duration).await;
    }
    false
}

impl RequestRecordCleanupResult {
    fn add(&mut self, batch: CleanupBatch) {
        self.deleted_records += batch.deleted_records;
        self.deleted_candidates += batch.deleted_candidates;
        self.compressed_records += batch.compressed_records;
        self.compressed_candidates += batch.compressed_candidates;
    }
}

impl CleanupBatch {
    fn is_empty(self) -> bool {
        self.deleted_records == 0 && self.deleted_candidates == 0 && self.scanned_records == 0 && self.scanned_candidates == 0
    }
}

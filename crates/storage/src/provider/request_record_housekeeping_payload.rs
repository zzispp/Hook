use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
    sea_query::{LockBehavior, LockType},
};

use crate::StorageResult;

use super::{
    record::{request_candidates, request_records},
    repository::ProviderStore,
    request_record_housekeeping::{CompressBatchResult, RequestRecordCleanupOptions},
    request_record_housekeeping_timeout::{CleanupBudget, apply_timeouts},
    request_record_payload_codec,
};

const COMPRESSED_MARKER_PATTERN: &str = r#"%"__hook_compressed_payload__"%"#;

pub(super) async fn compress_record_batch(
    store: &ProviderStore,
    options: &RequestRecordCleanupOptions,
    budget: &CleanupBudget,
) -> StorageResult<CompressBatchResult> {
    if budget.exhausted() {
        return Ok(exhausted_batch());
    }
    let records = summary_payload_batch(store, options, budget).await?;
    let scanned = records.len() as u64;
    let mut changed = 0;
    for record in records {
        if budget.exhausted() {
            return Ok(CompressBatchResult {
                scanned,
                changed,
                time_budget_exhausted: true,
            });
        }
        changed += u64::from(update_summary_payloads(store, options, budget, record).await?);
    }
    Ok(CompressBatchResult {
        scanned,
        changed,
        time_budget_exhausted: budget.exhausted(),
    })
}

pub(super) async fn compress_candidate_batch(
    store: &ProviderStore,
    options: &RequestRecordCleanupOptions,
    budget: &CleanupBudget,
) -> StorageResult<CompressBatchResult> {
    if budget.exhausted() {
        return Ok(exhausted_batch());
    }
    let records = candidate_payload_batch(store, options, budget).await?;
    let scanned = records.len() as u64;
    let mut changed = 0;
    for record in records {
        if budget.exhausted() {
            return Ok(CompressBatchResult {
                scanned,
                changed,
                time_budget_exhausted: true,
            });
        }
        changed += u64::from(update_candidate_payloads(store, options, budget, record).await?);
    }
    Ok(CompressBatchResult {
        scanned,
        changed,
        time_budget_exhausted: budget.exhausted(),
    })
}

async fn summary_payload_batch(
    store: &ProviderStore,
    options: &RequestRecordCleanupOptions,
    budget: &CleanupBudget,
) -> StorageResult<Vec<request_records::Model>> {
    let tx = store.connection().begin().await?;
    apply_timeouts(&tx, options, budget).await?;
    let records = request_records::Entity::find()
        .filter(request_records::Column::CreatedAt.lt(options.payload_cutoff))
        .filter(uncompressed_summary_payload_exists())
        .order_by_asc(request_records::Column::CreatedAt)
        .order_by_asc(request_records::Column::RequestId)
        .limit(options.compress_batch_size)
        .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
        .all(&tx)
        .await?;
    tx.commit().await?;
    Ok(records)
}

async fn candidate_payload_batch(
    store: &ProviderStore,
    options: &RequestRecordCleanupOptions,
    budget: &CleanupBudget,
) -> StorageResult<Vec<request_candidates::Model>> {
    let tx = store.connection().begin().await?;
    apply_timeouts(&tx, options, budget).await?;
    let records = request_candidates::Entity::find()
        .filter(request_candidates::Column::CreatedAt.lt(options.payload_cutoff))
        .filter(uncompressed_candidate_payload_exists())
        .order_by_asc(request_candidates::Column::CreatedAt)
        .order_by_asc(request_candidates::Column::Id)
        .limit(options.compress_batch_size)
        .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
        .all(&tx)
        .await?;
    tx.commit().await?;
    Ok(records)
}

async fn update_summary_payloads(
    store: &ProviderStore,
    options: &RequestRecordCleanupOptions,
    budget: &CleanupBudget,
    record: request_records::Model,
) -> StorageResult<bool> {
    let Some(active) = compressed_summary_payloads(record)? else {
        return Ok(false);
    };
    let tx = store.connection().begin().await?;
    apply_timeouts(&tx, options, budget).await?;
    active.update(&tx).await?;
    tx.commit().await?;
    Ok(true)
}

async fn update_candidate_payloads(
    store: &ProviderStore,
    options: &RequestRecordCleanupOptions,
    budget: &CleanupBudget,
    record: request_candidates::Model,
) -> StorageResult<bool> {
    let Some(active) = compressed_candidate_payloads(record)? else {
        return Ok(false);
    };
    let tx = store.connection().begin().await?;
    apply_timeouts(&tx, options, budget).await?;
    active.update(&tx).await?;
    tx.commit().await?;
    Ok(true)
}

fn compressed_summary_payloads(record: request_records::Model) -> StorageResult<Option<request_records::ActiveModel>> {
    let request_headers = request_record_payload_codec::compress_payload(record.request_headers.clone())?;
    let request_body = request_record_payload_codec::compress_payload(record.request_body.clone())?;
    let response_headers = request_record_payload_codec::compress_payload(record.client_response_headers.clone())?;
    let response_body = request_record_payload_codec::compress_payload(record.client_response_body.clone())?;
    if summary_payloads_unchanged(&record, &request_headers, &request_body, &response_headers, &response_body) {
        return Ok(None);
    }
    let mut active: request_records::ActiveModel = record.into();
    active.request_headers = Set(request_headers);
    active.request_body = Set(request_body);
    active.client_response_headers = Set(response_headers);
    active.client_response_body = Set(response_body);
    Ok(Some(active))
}

fn compressed_candidate_payloads(record: request_candidates::Model) -> StorageResult<Option<request_candidates::ActiveModel>> {
    let request_headers = request_record_payload_codec::compress_payload(record.provider_request_headers.clone())?;
    let request_body = request_record_payload_codec::compress_payload(record.provider_request_body.clone())?;
    let response_headers = request_record_payload_codec::compress_payload(record.provider_response_headers.clone())?;
    let response_body = request_record_payload_codec::compress_payload(record.provider_response_body.clone())?;
    if candidate_payloads_unchanged(&record, &request_headers, &request_body, &response_headers, &response_body) {
        return Ok(None);
    }
    let mut active: request_candidates::ActiveModel = record.into();
    active.provider_request_headers = Set(request_headers);
    active.provider_request_body = Set(request_body);
    active.provider_response_headers = Set(response_headers);
    active.provider_response_body = Set(response_body);
    Ok(Some(active))
}

fn uncompressed_summary_payload_exists() -> Condition {
    Condition::any()
        .add(uncompressed_summary_column(request_records::Column::RequestHeaders))
        .add(uncompressed_summary_column(request_records::Column::RequestBody))
        .add(uncompressed_summary_column(request_records::Column::ClientResponseHeaders))
        .add(uncompressed_summary_column(request_records::Column::ClientResponseBody))
}

fn uncompressed_candidate_payload_exists() -> Condition {
    Condition::any()
        .add(uncompressed_candidate_column(request_candidates::Column::ProviderRequestHeaders))
        .add(uncompressed_candidate_column(request_candidates::Column::ProviderRequestBody))
        .add(uncompressed_candidate_column(request_candidates::Column::ProviderResponseHeaders))
        .add(uncompressed_candidate_column(request_candidates::Column::ProviderResponseBody))
}

fn uncompressed_summary_column(column: request_records::Column) -> Condition {
    Condition::all().add(column.is_not_null()).add(column.not_like(COMPRESSED_MARKER_PATTERN))
}

fn uncompressed_candidate_column(column: request_candidates::Column) -> Condition {
    Condition::all().add(column.is_not_null()).add(column.not_like(COMPRESSED_MARKER_PATTERN))
}

fn summary_payloads_unchanged(
    record: &request_records::Model,
    request_headers: &Option<String>,
    request_body: &Option<String>,
    response_headers: &Option<String>,
    response_body: &Option<String>,
) -> bool {
    request_headers == &record.request_headers
        && request_body == &record.request_body
        && response_headers == &record.client_response_headers
        && response_body == &record.client_response_body
}

fn candidate_payloads_unchanged(
    record: &request_candidates::Model,
    request_headers: &Option<String>,
    request_body: &Option<String>,
    response_headers: &Option<String>,
    response_body: &Option<String>,
) -> bool {
    request_headers == &record.provider_request_headers
        && request_body == &record.provider_request_body
        && response_headers == &record.provider_response_headers
        && response_body == &record.provider_response_body
}

fn exhausted_batch() -> CompressBatchResult {
    CompressBatchResult {
        scanned: 0,
        changed: 0,
        time_budget_exhausted: true,
    }
}

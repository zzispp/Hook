use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set, sea_query::Expr};
use time::OffsetDateTime;

use crate::StorageResult;

use super::{
    record::{request_candidates, request_records},
    repository::ProviderStore,
    request_record_payload_codec,
};

const STATUS_FAILED: &str = "failed";
const STATUS_PENDING: &str = "pending";
const STATUS_SCHEDULED: &str = "scheduled";
const STATUS_SKIPPED: &str = "skipped";
const STATUS_STREAMING: &str = "streaming";
const BILLING_STATUS_VOID: &str = "void";
const CLIENT_STATUS_GATEWAY_TIMEOUT: i32 = 504;
const STALE_PENDING_REASON: &str = "stale_pending_request";
const STALE_STREAMING_REASON: &str = "stale_streaming_request";
const STALE_ACTIVE_CANDIDATE_STATUSES: [&str; 3] = [STATUS_SCHEDULED, STATUS_PENDING, STATUS_STREAMING];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StaleRequestRecordSweepResult {
    pub request_records: u64,
    pub request_candidates: u64,
}

pub async fn delete_request_records_before(store: &ProviderStore, cutoff: OffsetDateTime) -> StorageResult<u64> {
    let summaries = request_records::Entity::delete_many()
        .filter(request_records::Column::CreatedAt.lt(cutoff))
        .exec(store.connection())
        .await?;
    request_candidates::Entity::delete_many()
        .filter(request_candidates::Column::CreatedAt.lt(cutoff))
        .exec(store.connection())
        .await?;
    Ok(summaries.rows_affected)
}

pub async fn mark_stale_request_records_failed(
    store: &ProviderStore,
    pending_cutoff: OffsetDateTime,
    streaming_cutoff: OffsetDateTime,
) -> StorageResult<StaleRequestRecordSweepResult> {
    let now = time::OffsetDateTime::now_utc();
    let pending_result = mark_stale_requests_for_status(store, StaleStatus::Pending, pending_cutoff, now).await?;
    let streaming_result = mark_stale_requests_for_status(store, StaleStatus::Streaming, streaming_cutoff, now).await?;
    Ok(StaleRequestRecordSweepResult {
        request_records: pending_result.request_records + streaming_result.request_records,
        request_candidates: pending_result.request_candidates + streaming_result.request_candidates,
    })
}

pub async fn compress_request_record_payloads_before(store: &ProviderStore, cutoff: OffsetDateTime) -> StorageResult<u64> {
    let summaries = request_records::Entity::find()
        .filter(request_records::Column::CreatedAt.lt(cutoff))
        .filter(summary_payload_exists())
        .all(store.connection())
        .await?;
    let mut changed = 0;
    for record in summaries {
        changed += u64::from(compress_summary_payloads(store, record).await?);
    }
    let candidates = request_candidates::Entity::find()
        .filter(request_candidates::Column::CreatedAt.lt(cutoff))
        .filter(candidate_payload_exists())
        .all(store.connection())
        .await?;
    for record in candidates {
        changed += u64::from(compress_candidate_payloads(store, record).await?);
    }
    Ok(changed)
}

async fn stale_request_records_for_status(store: &ProviderStore, status: &str, cutoff: OffsetDateTime) -> StorageResult<Vec<request_records::Model>> {
    request_records::Entity::find()
        .filter(request_records::Column::Status.eq(status))
        .filter(request_records::Column::UpdatedAt.lt(cutoff))
        .all(store.connection())
        .await
        .map_err(Into::into)
}

async fn mark_stale_requests_for_status(
    store: &ProviderStore,
    stale_status: StaleStatus,
    cutoff: OffsetDateTime,
    now: OffsetDateTime,
) -> StorageResult<StaleRequestRecordSweepResult> {
    let records = stale_request_records_for_status(store, stale_status.status(), cutoff).await?;
    let request_ids = records.iter().map(|record| record.request_id.clone()).collect::<Vec<_>>();
    let candidate_count = mark_stale_request_candidates(store, &request_ids, stale_status, now).await?;
    let record_count = update_stale_request_records(store, records, stale_status, now).await?;
    Ok(StaleRequestRecordSweepResult {
        request_records: record_count,
        request_candidates: candidate_count,
    })
}

async fn mark_stale_request_candidates(store: &ProviderStore, request_ids: &[String], stale_status: StaleStatus, now: OffsetDateTime) -> StorageResult<u64> {
    if request_ids.is_empty() {
        return Ok(0);
    }
    let result = request_candidates::Entity::update_many()
        .col_expr(request_candidates::Column::Status, Expr::val(STATUS_SKIPPED))
        .col_expr(request_candidates::Column::SkipReason, Expr::val(stale_status.reason()))
        .col_expr(request_candidates::Column::FinishedAt, Expr::val(now))
        .filter(request_candidates::Column::Status.is_in(STALE_ACTIVE_CANDIDATE_STATUSES))
        .filter(request_candidates::Column::RequestId.is_in(request_ids.iter().cloned()))
        .exec(store.connection())
        .await?;
    Ok(result.rows_affected)
}

async fn update_stale_request_records(
    store: &ProviderStore,
    records: Vec<request_records::Model>,
    stale_status: StaleStatus,
    now: OffsetDateTime,
) -> StorageResult<u64> {
    let mut updated_count = 0;
    for record in records {
        update_stale_request_record(store, record, stale_status, now).await?;
        updated_count += 1;
    }
    Ok(updated_count)
}

async fn update_stale_request_record(
    store: &ProviderStore,
    record: request_records::Model,
    stale_status: StaleStatus,
    now: OffsetDateTime,
) -> StorageResult<()> {
    let old_record = record.clone();
    let mut active: request_records::ActiveModel = record.into();
    active.status = Set(STATUS_FAILED.into());
    active.billing_status = Set(BILLING_STATUS_VOID.into());
    active.client_status_code = Set(Some(CLIENT_STATUS_GATEWAY_TIMEOUT));
    active.client_error_type = Set(Some(stale_status.reason().into()));
    active.client_error_message = Set(Some(stale_status.message().into()));
    active.termination_origin = Set(Some("system".into()));
    active.termination_reason = Set(Some(stale_status.reason().into()));
    active.stream_end_reason = Set(stale_status.stream_end_reason().map(str::to_owned));
    active.finished_at = Set(Some(now));
    active.updated_at = Set(now);
    let new_record = active.update(store.connection()).await?;
    crate::dashboard::sync_user_usage_buckets(store.connection(), &old_record, &new_record).await?;
    crate::dashboard::sync_cost_analysis_buckets(store.connection(), &old_record, &new_record).await?;
    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum StaleStatus {
    Pending,
    Streaming,
}

impl StaleStatus {
    fn status(self) -> &'static str {
        match self {
            Self::Pending => STATUS_PENDING,
            Self::Streaming => STATUS_STREAMING,
        }
    }

    fn reason(self) -> &'static str {
        match self {
            Self::Pending => STALE_PENDING_REASON,
            Self::Streaming => STALE_STREAMING_REASON,
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::Pending => "request stayed pending past the stale timeout",
            Self::Streaming => "request stayed streaming past the stale timeout",
        }
    }

    fn stream_end_reason(self) -> Option<&'static str> {
        match self {
            Self::Pending => None,
            Self::Streaming => Some(STALE_STREAMING_REASON),
        }
    }
}

fn summary_payload_exists() -> Condition {
    Condition::any()
        .add(request_records::Column::RequestHeaders.is_not_null())
        .add(request_records::Column::RequestBody.is_not_null())
        .add(request_records::Column::ClientResponseHeaders.is_not_null())
        .add(request_records::Column::ClientResponseBody.is_not_null())
}

fn candidate_payload_exists() -> Condition {
    Condition::any()
        .add(request_candidates::Column::ProviderRequestHeaders.is_not_null())
        .add(request_candidates::Column::ProviderRequestBody.is_not_null())
        .add(request_candidates::Column::ProviderResponseHeaders.is_not_null())
        .add(request_candidates::Column::ProviderResponseBody.is_not_null())
}

async fn compress_summary_payloads(store: &ProviderStore, record: request_records::Model) -> StorageResult<bool> {
    let request_headers = request_record_payload_codec::compress_payload(record.request_headers.clone())?;
    let request_body = request_record_payload_codec::compress_payload(record.request_body.clone())?;
    let client_response_headers = request_record_payload_codec::compress_payload(record.client_response_headers.clone())?;
    let client_response_body = request_record_payload_codec::compress_payload(record.client_response_body.clone())?;
    if request_headers == record.request_headers
        && request_body == record.request_body
        && client_response_headers == record.client_response_headers
        && client_response_body == record.client_response_body
    {
        return Ok(false);
    }
    let mut active: request_records::ActiveModel = record.into();
    active.request_headers = Set(request_headers);
    active.request_body = Set(request_body);
    active.client_response_headers = Set(client_response_headers);
    active.client_response_body = Set(client_response_body);
    active.update(store.connection()).await?;
    Ok(true)
}

async fn compress_candidate_payloads(store: &ProviderStore, record: request_candidates::Model) -> StorageResult<bool> {
    let provider_request_headers = request_record_payload_codec::compress_payload(record.provider_request_headers.clone())?;
    let provider_request_body = request_record_payload_codec::compress_payload(record.provider_request_body.clone())?;
    let provider_response_headers = request_record_payload_codec::compress_payload(record.provider_response_headers.clone())?;
    let provider_response_body = request_record_payload_codec::compress_payload(record.provider_response_body.clone())?;
    if provider_request_headers == record.provider_request_headers
        && provider_request_body == record.provider_request_body
        && provider_response_headers == record.provider_response_headers
        && provider_response_body == record.provider_response_body
    {
        return Ok(false);
    }
    let mut active: request_candidates::ActiveModel = record.into();
    active.provider_request_headers = Set(provider_request_headers);
    active.provider_request_body = Set(provider_request_body);
    active.provider_response_headers = Set(provider_response_headers);
    active.provider_response_body = Set(provider_response_body);
    active.update(store.connection()).await?;
    Ok(true)
}

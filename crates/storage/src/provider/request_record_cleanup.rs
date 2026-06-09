use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, sea_query::Expr};
use time::OffsetDateTime;

use crate::StorageResult;

use super::{
    record::{request_candidates, request_records},
    repository::ProviderStore,
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
    if result.rows_affected > 0 {
        for request_id in request_ids {
            super::request_record_partition_write::sync_request_candidates_for_request(store, request_id).await?;
        }
    }
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
    super::request_record_partition_write::sync_request_record(store, &new_record.request_id).await?;
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

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, sea_query::Expr};
use time::OffsetDateTime;

use crate::StorageResult;

use super::{
    StaleRequestSweepReport,
    record::{request_candidates, request_records},
    repository::ProviderStore,
};

const STALE_STATUS_CODE: i32 = 504;

pub async fn sweep_stale_request_records(
    store: &ProviderStore,
    pending_cutoff: OffsetDateTime,
    streaming_cutoff: OffsetDateTime,
) -> StorageResult<StaleRequestSweepReport> {
    let records = request_records::Entity::find()
        .filter(request_records::Column::FinishedAt.is_null())
        .filter(request_records::Column::Status.is_in(["pending", "streaming"]))
        .all(store.connection())
        .await?;
    let mut report = StaleRequestSweepReport::default();
    for record in records {
        let Some(kind) = stale_kind(&record, pending_cutoff, streaming_cutoff) else {
            continue;
        };
        sweep_request_record(store, record, &kind).await?;
        report.failed_candidates += fail_active_candidates(store, &kind).await?;
        report.skipped_candidates += skip_scheduled_candidates(store, &kind).await?;
        match kind {
            StaleKind::Pending { .. } => report.pending_records += 1,
            StaleKind::Streaming { .. } => report.streaming_records += 1,
        }
    }
    Ok(report)
}

#[derive(Clone)]
enum StaleKind {
    Pending { request_id: String },
    Streaming { request_id: String },
}

impl StaleKind {
    fn request_id(&self) -> &str {
        match self {
            Self::Pending { request_id } | Self::Streaming { request_id } => request_id,
        }
    }

    fn error_type(&self) -> &'static str {
        match self {
            Self::Pending { .. } => "stale_pending_request",
            Self::Streaming { .. } => "stale_streaming_request",
        }
    }

    fn error_message(&self) -> &'static str {
        match self {
            Self::Pending { .. } => "request remained pending beyond stale sweep threshold",
            Self::Streaming { .. } => "request remained streaming beyond stale sweep threshold",
        }
    }

    fn skip_reason(&self) -> &'static str {
        match self {
            Self::Pending { .. } => "stale_pending_request",
            Self::Streaming { .. } => "stale_streaming_request",
        }
    }

    fn termination_reason(&self) -> &'static str {
        match self {
            Self::Pending { .. } => "pending_timeout",
            Self::Streaming { .. } => "streaming_timeout",
        }
    }

    fn stream_end_reason(&self) -> Option<&'static str> {
        match self {
            Self::Pending { .. } => None,
            Self::Streaming { .. } => Some("stale_streaming_timeout"),
        }
    }
}

fn stale_kind(
    record: &request_records::Model,
    pending_cutoff: OffsetDateTime,
    streaming_cutoff: OffsetDateTime,
) -> Option<StaleKind> {
    let anchor = record.started_at.unwrap_or(record.created_at);
    match record.status.as_str() {
        "pending" if anchor < pending_cutoff => Some(StaleKind::Pending {
            request_id: record.request_id.clone(),
        }),
        "streaming" if anchor < streaming_cutoff => Some(StaleKind::Streaming {
            request_id: record.request_id.clone(),
        }),
        _ => None,
    }
}

async fn sweep_request_record(store: &ProviderStore, record: request_records::Model, kind: &StaleKind) -> StorageResult<()> {
    let now = OffsetDateTime::now_utc();
    let mut active: request_records::ActiveModel = record.into();
    active.status = Set("failed".into());
    active.billing_status = Set("void".into());
    active.client_status_code = Set(Some(STALE_STATUS_CODE));
    active.client_error_type = Set(Some(kind.error_type().into()));
    active.client_error_message = Set(Some(kind.error_message().into()));
    active.termination_origin = Set(Some("server".into()));
    active.termination_reason = Set(Some(kind.termination_reason().into()));
    active.stream_end_reason = Set(kind.stream_end_reason().map(str::to_owned));
    active.finished_at = Set(Some(now));
    active.updated_at = Set(now);
    active.update(store.connection()).await?;
    Ok(())
}

async fn fail_active_candidates(store: &ProviderStore, kind: &StaleKind) -> StorageResult<u64> {
    let result = request_candidates::Entity::update_many()
        .col_expr(request_candidates::Column::Status, Expr::val("failed"))
        .col_expr(request_candidates::Column::SkipReason, null_text())
        .col_expr(request_candidates::Column::StatusCode, Expr::val(STALE_STATUS_CODE))
        .col_expr(request_candidates::Column::ErrorType, Expr::val(kind.error_type()))
        .col_expr(request_candidates::Column::ErrorMessage, Expr::val(kind.error_message()))
        .col_expr(request_candidates::Column::ErrorCode, null_text())
        .col_expr(request_candidates::Column::ErrorParam, null_text())
        .col_expr(request_candidates::Column::FinishedAt, Expr::val(OffsetDateTime::now_utc()))
        .filter(request_candidates::Column::RequestId.eq(kind.request_id()))
        .filter(request_candidates::Column::FinishedAt.is_null())
        .filter(request_candidates::Column::Status.is_in(["pending", "streaming"]))
        .exec(store.connection())
        .await?;
    Ok(result.rows_affected)
}

async fn skip_scheduled_candidates(store: &ProviderStore, kind: &StaleKind) -> StorageResult<u64> {
    let result = request_candidates::Entity::update_many()
        .col_expr(request_candidates::Column::Status, Expr::val("skipped"))
        .col_expr(request_candidates::Column::SkipReason, Expr::val(kind.skip_reason()))
        .col_expr(request_candidates::Column::FinishedAt, Expr::val(OffsetDateTime::now_utc()))
        .filter(request_candidates::Column::RequestId.eq(kind.request_id()))
        .filter(request_candidates::Column::Status.eq("scheduled"))
        .exec(store.connection())
        .await?;
    Ok(result.rows_affected)
}

fn null_text() -> Expr {
    Expr::value(Option::<String>::None)
}

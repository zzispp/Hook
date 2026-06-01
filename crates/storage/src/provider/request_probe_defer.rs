use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait, sea_query::Expr};

use crate::{StorageError, StorageResult};

use super::{
    record::{request_candidates, request_records},
    repository::ProviderStore,
};

pub(super) async fn mark_model_status_probe_deferred(store: &ProviderStore, request_id: &str, skip_reason: &str) -> StorageResult<()> {
    let now = time::OffsetDateTime::now_utc();
    let tx = store.connection().begin().await?;
    request_candidates::Entity::update_many()
        .col_expr(request_candidates::Column::Status, Expr::val("skipped"))
        .col_expr(request_candidates::Column::SkipReason, Expr::val(skip_reason))
        .col_expr(request_candidates::Column::FinishedAt, Expr::val(now))
        .filter(request_candidates::Column::RequestId.eq(request_id))
        .filter(request_candidates::Column::Status.eq("scheduled"))
        .exec(&tx)
        .await?;
    let summary = request_records::Entity::update_many()
        .col_expr(request_records::Column::Status, Expr::val("skipped"))
        .col_expr(request_records::Column::BillingStatus, Expr::val("void"))
        .col_expr(request_records::Column::FinishedAt, Expr::val(now))
        .col_expr(request_records::Column::UpdatedAt, Expr::val(now))
        .filter(request_records::Column::RequestId.eq(request_id))
        .filter(request_records::Column::Status.eq("pending"))
        .exec(&tx)
        .await?;
    if summary.rows_affected != 1 {
        return Err(StorageError::Database(format!(
            "model status probe deferred request summary transition affected {} rows for {request_id}",
            summary.rows_affected
        )));
    }
    tx.commit().await?;
    Ok(())
}

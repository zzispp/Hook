use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, sea_query::Expr};
use time::OffsetDateTime;

use crate::StorageResult;

use super::{
    record::{request_candidates, request_records},
    repository::ProviderStore,
};

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

pub async fn clear_request_record_payloads_before(store: &ProviderStore, cutoff: OffsetDateTime) -> StorageResult<u64> {
    let result = request_candidates::Entity::update_many()
        .col_expr(request_candidates::Column::RequestHeaders, null_text())
        .col_expr(request_candidates::Column::RequestBody, null_text())
        .col_expr(request_candidates::Column::ResponseBody, null_text())
        .filter(request_candidates::Column::CreatedAt.lt(cutoff))
        .filter(payload_exists())
        .exec(store.connection())
        .await?;
    Ok(result.rows_affected)
}

fn payload_exists() -> Condition {
    Condition::any()
        .add(request_candidates::Column::RequestHeaders.is_not_null())
        .add(request_candidates::Column::RequestBody.is_not_null())
        .add(request_candidates::Column::ResponseBody.is_not_null())
}

fn null_text() -> Expr {
    Expr::value(Option::<String>::None)
}

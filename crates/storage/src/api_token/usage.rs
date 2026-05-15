use rust_decimal::Decimal;
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, TransactionTrait,
    sea_query::{Expr, ExprTrait},
};

use crate::{
    StorageError, StorageResult,
    usage_flush::{batch_exists, insert_batch, token_usage_flush_batch},
};

use super::{ApiTokenUsageRecord, record::api_tokens};

pub(super) async fn record_usage<C>(connection: &C, input: &ApiTokenUsageRecord) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let result = api_tokens::Entity::update_many()
        .col_expr(api_tokens::Column::UsedQuota, used_quota_expr(input.cost))
        .col_expr(api_tokens::Column::RequestCount, request_count_expr(input.request_count))
        .col_expr(api_tokens::Column::LastUsedAt, Expr::val(input.used_at))
        .col_expr(api_tokens::Column::UpdatedAt, Expr::val(input.used_at))
        .filter(api_tokens::Column::Id.eq(input.token_id.as_str()))
        .exec(connection)
        .await?;
    ensure_usage_recorded(result.rows_affected)
}

pub(super) async fn record_usage_batch(connection: &sea_orm::DatabaseConnection, inputs: &[ApiTokenUsageRecord]) -> StorageResult<()> {
    if inputs.is_empty() {
        return Ok(());
    }
    let tx = connection.begin().await?;
    for input in inputs {
        record_usage(&tx, input).await?;
    }
    tx.commit().await?;
    Ok(())
}

pub(super) async fn record_usage_batch_once(connection: &sea_orm::DatabaseConnection, batch_id: &str, inputs: &[ApiTokenUsageRecord]) -> StorageResult<bool> {
    if inputs.is_empty() {
        return Ok(false);
    }
    let tx = connection.begin().await?;
    if batch_exists(&tx, batch_id).await? {
        tx.commit().await?;
        return Ok(false);
    }
    for input in inputs {
        record_usage(&tx, input).await?;
    }
    insert_batch(&tx, token_usage_flush_batch(batch_id, inputs.len())?).await?;
    tx.commit().await?;
    Ok(true)
}

fn used_quota_expr(cost: Decimal) -> Expr {
    Expr::col(api_tokens::Column::UsedQuota).add(Expr::val(cost))
}

fn request_count_expr(request_count: i64) -> Expr {
    Expr::col(api_tokens::Column::RequestCount).add(Expr::val(request_count))
}

fn ensure_usage_recorded(rows_affected: u64) -> StorageResult<()> {
    if rows_affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

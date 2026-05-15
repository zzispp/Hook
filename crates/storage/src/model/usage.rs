use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, TransactionTrait,
    sea_query::{Expr, ExprTrait},
};

use crate::{
    StorageError, StorageResult,
    usage_flush::{batch_exists, insert_batch, model_usage_flush_batch},
};

use super::{GlobalModelUsageRecord, record::global_models};

pub(super) async fn record_usage<C>(connection: &C, input: &GlobalModelUsageRecord) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let result = global_models::Entity::update_many()
        .col_expr(global_models::Column::UsageCount, usage_count_expr(input.count))
        .filter(global_models::Column::Id.eq(input.model_id.as_str()))
        .exec(connection)
        .await?;
    ensure_usage_recorded(result.rows_affected)
}

pub(super) async fn record_usage_batch(connection: &sea_orm::DatabaseConnection, inputs: &[GlobalModelUsageRecord]) -> StorageResult<()> {
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

pub(super) async fn record_usage_batch_once(
    connection: &sea_orm::DatabaseConnection,
    batch_id: &str,
    inputs: &[GlobalModelUsageRecord],
) -> StorageResult<bool> {
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
    insert_batch(&tx, model_usage_flush_batch(batch_id, inputs.len())?).await?;
    tx.commit().await?;
    Ok(true)
}

fn usage_count_expr(count: i64) -> Expr {
    Expr::col(global_models::Column::UsageCount).add(Expr::val(count))
}

fn ensure_usage_recorded(rows_affected: u64) -> StorageResult<()> {
    if rows_affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

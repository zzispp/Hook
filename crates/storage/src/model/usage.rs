use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, TransactionTrait,
    sea_query::{Expr, ExprTrait},
};

use crate::{
    StorageError, StorageResult,
    usage_flush::{UsageFlushApplyReport, batch_exists, insert_batch, model_usage_flush_batch},
};

use super::{GlobalModelUsageRecord, record::global_models};

pub(super) async fn record_usage<C>(connection: &C, input: &GlobalModelUsageRecord) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    ensure_usage_recorded(apply_usage(connection, input).await?)
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
) -> StorageResult<UsageFlushApplyReport> {
    if inputs.is_empty() {
        return Ok(UsageFlushApplyReport::empty());
    }
    let tx = connection.begin().await?;
    if batch_exists(&tx, batch_id).await? {
        tx.commit().await?;
        return Ok(UsageFlushApplyReport::already_applied());
    }
    let report = record_flush_inputs(&tx, inputs).await?;
    insert_batch(&tx, model_usage_flush_batch(batch_id, inputs.len())?).await?;
    tx.commit().await?;
    Ok(report)
}

async fn record_flush_inputs<C>(connection: &C, inputs: &[GlobalModelUsageRecord]) -> StorageResult<UsageFlushApplyReport>
where
    C: ConnectionTrait,
{
    let mut applied_records = 0;
    let mut skipped_missing_resource_ids = Vec::new();
    for input in inputs {
        if apply_usage(connection, input).await? {
            applied_records += 1;
        } else {
            skipped_missing_resource_ids.push(input.model_id.clone());
        }
    }
    Ok(UsageFlushApplyReport::applied(applied_records, skipped_missing_resource_ids))
}

async fn apply_usage<C>(connection: &C, input: &GlobalModelUsageRecord) -> StorageResult<bool>
where
    C: ConnectionTrait,
{
    let result = global_models::Entity::update_many()
        .col_expr(global_models::Column::UsageCount, usage_count_expr(input.count))
        .filter(global_models::Column::Id.eq(input.model_id.as_str()))
        .exec(connection)
        .await?;
    Ok(result.rows_affected > 0)
}

fn usage_count_expr(count: i64) -> Expr {
    Expr::col(global_models::Column::UsageCount).add(Expr::val(count))
}

fn ensure_usage_recorded(recorded: bool) -> StorageResult<()> {
    if !recorded {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

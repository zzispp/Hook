use std::collections::HashSet;

use sea_orm::{
    ColumnTrait, ConnectionTrait, DbBackend, EntityTrait, QueryFilter, Statement, TransactionTrait, Value,
    sea_query::{Expr, ExprTrait},
};

use crate::{
    StorageError, StorageResult,
    usage_flush::{UsageFlushApplyReport, batch_exists, insert_batch, model_usage_flush_batch},
};

use super::{GlobalModelUsageRecord, GlobalModelUserUsageRecord, record::global_models};

pub(super) async fn record_usage<C>(connection: &C, input: &GlobalModelUsageRecord) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    ensure_usage_recorded(apply_usage(connection, input).await?)?;
    apply_user_usage_for_global_record(connection, input).await
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
    user_inputs: &[GlobalModelUserUsageRecord],
) -> StorageResult<UsageFlushApplyReport> {
    if inputs.is_empty() && user_inputs.is_empty() {
        return Ok(UsageFlushApplyReport::empty());
    }
    let tx = connection.begin().await?;
    if batch_exists(&tx, batch_id).await? {
        tx.commit().await?;
        return Ok(UsageFlushApplyReport::already_applied());
    }
    ensure_user_records_have_platform_records(inputs, user_inputs)?;
    let report = record_flush_inputs(&tx, inputs, user_inputs).await?;
    insert_batch(&tx, model_usage_flush_batch(batch_id, inputs.len() + user_inputs.len())?).await?;
    tx.commit().await?;
    Ok(report)
}

fn ensure_user_records_have_platform_records(inputs: &[GlobalModelUsageRecord], user_inputs: &[GlobalModelUserUsageRecord]) -> StorageResult<()> {
    let model_ids = inputs.iter().map(|input| input.model_id.as_str()).collect::<HashSet<_>>();
    let missing = user_inputs.iter().find(|input| !model_ids.contains(input.model_id.as_str()));
    if let Some(input) = missing {
        return Err(StorageError::Database(format!(
            "user model usage record missing platform model usage for model {}",
            input.model_id
        )));
    }
    Ok(())
}

async fn record_flush_inputs<C>(
    connection: &C,
    inputs: &[GlobalModelUsageRecord],
    user_inputs: &[GlobalModelUserUsageRecord],
) -> StorageResult<UsageFlushApplyReport>
where
    C: ConnectionTrait,
{
    let mut applied_records = 0;
    let mut skipped_missing_resource_ids = Vec::new();
    let mut applied_model_ids = HashSet::new();
    for input in inputs {
        if apply_usage(connection, input).await? {
            applied_records += 1;
            applied_model_ids.insert(input.model_id.clone());
        } else {
            skipped_missing_resource_ids.push(input.model_id.clone());
        }
    }
    for input in user_inputs {
        if applied_model_ids.contains(&input.model_id) {
            apply_user_usage(connection, input).await?;
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
    if result.rows_affected == 0 {
        return Ok(false);
    }
    Ok(true)
}

fn usage_count_expr(count: i64) -> Expr {
    Expr::col(global_models::Column::UsageCount).add(Expr::val(count))
}

async fn apply_user_usage<C>(connection: &C, input: &GlobalModelUserUsageRecord) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    connection
        .execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            user_usage_upsert_sql(),
            [
                Value::from(input.user_id.clone()),
                Value::from(input.model_id.clone()),
                Value::from(input.count),
            ],
        ))
        .await?;
    Ok(())
}

async fn apply_user_usage_for_global_record<C>(connection: &C, input: &GlobalModelUsageRecord) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let Some(user_id) = input.user_id.as_ref() else {
        return Ok(());
    };
    apply_user_usage(
        connection,
        &GlobalModelUserUsageRecord {
            user_id: user_id.clone(),
            model_id: input.model_id.clone(),
            count: input.count,
        },
    )
    .await
}

fn user_usage_upsert_sql() -> &'static str {
    "INSERT INTO global_model_user_usage_counts \
     (user_id, global_model_id, usage_count, created_at, updated_at) \
     VALUES ($1, $2, $3, NOW(), NOW()) \
     ON CONFLICT (user_id, global_model_id) DO UPDATE SET \
     usage_count = global_model_user_usage_counts.usage_count + EXCLUDED.usage_count, \
     updated_at = EXCLUDED.updated_at"
}

fn ensure_usage_recorded(recorded: bool) -> StorageResult<()> {
    if !recorded {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

use sea_orm::{ActiveModelTrait, DbBackend, EntityTrait, FromQueryResult, Set, Statement, TransactionTrait};

use crate::{StorageError, StorageResult};

use super::{ModelStatusDueRecord, ModelStatusStore, entities::checks, query::CheckRow};

const LOCK_SECONDS: i64 = 120;
pub(super) const DUE_CHECKS_SQL: &str = "WITH due AS (SELECT id FROM model_status_checks WHERE enabled = TRUE AND next_due_at <= $1 AND (locked_until IS NULL OR locked_until <= $1) ORDER BY next_due_at ASC LIMIT {limit} FOR UPDATE SKIP LOCKED), \
             updated AS (UPDATE model_status_checks c SET locked_until = $2, updated_at = $1 FROM due WHERE c.id = due.id RETURNING c.*) \
             SELECT c.id, c.name, c.global_model_id, g.name AS model_name, c.api_format, c.api_token_id, t.name AS api_token_name, c.interval_seconds, c.enabled, c.next_due_at, c.last_status, c.last_checked_at, c.last_latency_ms, c.last_message, c.created_at, c.updated_at \
             FROM updated c JOIN global_models g ON g.id = c.global_model_id JOIN api_tokens t ON t.id = c.api_token_id";

pub(super) async fn due_checks(store: &ModelStatusStore, limit: u64, now: time::OffsetDateTime) -> StorageResult<Vec<ModelStatusDueRecord>> {
    let rows = claim_due_rows(store, limit, now).await?;
    let mut output = Vec::with_capacity(rows.len());
    for row in rows {
        let token = store.independent_token(&row.api_token_id).await?.ok_or(StorageError::NotFound)?;
        output.push(ModelStatusDueRecord {
            check_id: row.id,
            model_name: row.model_name,
            api_format: row.api_format,
            interval_seconds: row.interval_seconds,
            token,
        });
    }
    Ok(output)
}

pub(super) async fn defer_check(store: &ModelStatusStore, check_id: &str, next_due_at: time::OffsetDateTime) -> StorageResult<()> {
    let check = checks::Entity::find_by_id(check_id.to_owned())
        .one(store.connection())
        .await?
        .ok_or(StorageError::NotFound)?;
    let mut active: checks::ActiveModel = check.into();
    active.locked_until = Set(None);
    active.next_due_at = Set(next_due_at);
    active.updated_at = Set(time::OffsetDateTime::now_utc());
    active.update(store.connection()).await?;
    Ok(())
}

async fn claim_due_rows(store: &ModelStatusStore, limit: u64, now: time::OffsetDateTime) -> StorageResult<Vec<CheckRow>> {
    let lock_until = now + time::Duration::seconds(LOCK_SECONDS);
    let sql = DUE_CHECKS_SQL.replace("{limit}", &limit.to_string());
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, sql, vec![now.into(), lock_until.into()]);
    let tx = store.connection().begin().await?;
    let rows = CheckRow::find_by_statement(statement).all(&tx).await?;
    tx.commit().await?;
    Ok(rows)
}

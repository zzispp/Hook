use sea_orm::{DbBackend, FromQueryResult, Statement, TransactionTrait};

use crate::{StorageError, StorageResult};

use super::{
    repository::ProviderStore,
    request_record_housekeeping::RequestRecordCleanupOptions,
    request_record_housekeeping_timeout::{CleanupBudget, apply_timeouts},
};

const SELECT_RECORD_BATCH_SQL: &str =
    "SELECT request_id FROM request_records WHERE created_at < $1 ORDER BY created_at ASC, request_id ASC LIMIT $2 FOR UPDATE SKIP LOCKED";
const DELETE_ORPHAN_CANDIDATES_SQL: &str = "WITH orphan_candidates AS (SELECT c.id FROM request_candidates c WHERE c.created_at < $1 AND NOT EXISTS (SELECT 1 FROM request_records r WHERE r.request_id = c.request_id) ORDER BY c.created_at ASC, c.id ASC LIMIT $2 FOR UPDATE SKIP LOCKED), \
deleted_candidates AS (DELETE FROM request_candidates c USING orphan_candidates o WHERE c.id = o.id RETURNING c.id) \
SELECT COUNT(*) AS deleted_candidates FROM deleted_candidates";
const DELETE_EXPIRED_ROUTING_DECISIONS_SQL: &str = "WITH expired_decisions AS (SELECT request_id FROM routing_decision_samples WHERE created_at < $1 ORDER BY created_at ASC, request_id ASC LIMIT $2 FOR UPDATE SKIP LOCKED), \
deleted_decisions AS (DELETE FROM routing_decision_samples d USING expired_decisions e WHERE d.request_id = e.request_id RETURNING d.request_id) \
SELECT COUNT(*) AS deleted_routing_decisions FROM deleted_decisions";

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct DeleteRecordBatch {
    pub(super) deleted_records: u64,
    pub(super) deleted_candidates: u64,
}

pub(super) async fn delete_record_batch(
    store: &ProviderStore,
    options: &RequestRecordCleanupOptions,
    budget: &CleanupBudget,
) -> StorageResult<DeleteRecordBatch> {
    if budget.exhausted() {
        return Ok(DeleteRecordBatch::default());
    }
    let tx = store.connection().begin().await?;
    apply_timeouts(&tx, options).await?;
    let request_ids = RequestIdRow::find_by_statement(select_record_batch_statement(options))
        .all(&tx)
        .await?
        .into_iter()
        .map(|row| row.request_id)
        .collect::<Vec<_>>();
    if request_ids.is_empty() {
        tx.commit().await?;
        return Ok(DeleteRecordBatch::default());
    }
    let deleted_candidates = delete_record_candidates(&tx, &request_ids).await?;
    let deleted_records = delete_records(&tx, &request_ids).await?;
    tx.commit().await?;
    Ok(DeleteRecordBatch {
        deleted_records,
        deleted_candidates,
    })
}

pub(super) async fn delete_orphan_candidate_batch(store: &ProviderStore, options: &RequestRecordCleanupOptions, budget: &CleanupBudget) -> StorageResult<u64> {
    if budget.exhausted() {
        return Ok(0);
    }
    let tx = store.connection().begin().await?;
    apply_timeouts(&tx, options).await?;
    let row = DeleteOrphanCandidateBatchRow::find_by_statement(delete_orphan_candidates_statement(options))
        .one(&tx)
        .await?
        .ok_or_else(|| StorageError::Database("delete orphan request candidates batch returned no rows".into()))?;
    tx.commit().await?;
    u64::try_from(row.deleted_candidates).map_err(count_overflow)
}

pub(super) async fn delete_expired_routing_decision_batch(
    store: &ProviderStore,
    options: &RequestRecordCleanupOptions,
    budget: &CleanupBudget,
) -> StorageResult<u64> {
    if budget.exhausted() {
        return Ok(0);
    }
    let tx = store.connection().begin().await?;
    apply_timeouts(&tx, options).await?;
    let row = DeleteRoutingDecisionBatchRow::find_by_statement(delete_expired_routing_decisions_statement(options))
        .one(&tx)
        .await?
        .ok_or_else(|| StorageError::Database("delete expired routing decision samples batch returned no rows".into()))?;
    tx.commit().await?;
    u64::try_from(row.deleted_routing_decisions).map_err(count_overflow)
}

async fn delete_record_candidates(tx: &sea_orm::DatabaseTransaction, request_ids: &[String]) -> StorageResult<u64> {
    let row = DeleteCandidatesRow::find_by_statement(delete_candidates_statement(request_ids))
        .one(tx)
        .await?
        .ok_or_else(|| StorageError::Database("delete request candidates batch returned no rows".into()))?;
    u64::try_from(row.deleted_candidates).map_err(count_overflow)
}

async fn delete_records(tx: &sea_orm::DatabaseTransaction, request_ids: &[String]) -> StorageResult<u64> {
    let row = DeleteRecordsRow::find_by_statement(delete_selected_records_statement(request_ids))
        .one(tx)
        .await?
        .ok_or_else(|| StorageError::Database("delete request records batch returned no rows".into()))?;
    u64::try_from(row.deleted_records).map_err(count_overflow)
}

fn select_record_batch_statement(options: &RequestRecordCleanupOptions) -> Statement {
    Statement::from_sql_and_values(
        DbBackend::Postgres,
        SELECT_RECORD_BATCH_SQL,
        vec![options.record_cutoff.into(), batch_limit(options.delete_batch_size).into()],
    )
}

fn delete_candidates_statement(request_ids: &[String]) -> Statement {
    Statement::from_sql_and_values(
        DbBackend::Postgres,
        format!(
            "WITH deleted_candidates AS (DELETE FROM request_candidates WHERE request_id IN ({}) RETURNING id) SELECT COUNT(*) AS deleted_candidates FROM deleted_candidates",
            placeholders(request_ids.len())
        ),
        request_id_values(request_ids),
    )
}

fn delete_selected_records_statement(request_ids: &[String]) -> Statement {
    Statement::from_sql_and_values(
        DbBackend::Postgres,
        format!(
            "WITH deleted_records AS (DELETE FROM request_records WHERE request_id IN ({}) RETURNING request_id) SELECT COUNT(*) AS deleted_records FROM deleted_records",
            placeholders(request_ids.len())
        ),
        request_id_values(request_ids),
    )
}

fn delete_orphan_candidates_statement(options: &RequestRecordCleanupOptions) -> Statement {
    Statement::from_sql_and_values(
        DbBackend::Postgres,
        DELETE_ORPHAN_CANDIDATES_SQL,
        vec![options.record_cutoff.into(), batch_limit(options.delete_batch_size).into()],
    )
}

fn delete_expired_routing_decisions_statement(options: &RequestRecordCleanupOptions) -> Statement {
    Statement::from_sql_and_values(
        DbBackend::Postgres,
        DELETE_EXPIRED_ROUTING_DECISIONS_SQL,
        vec![options.record_cutoff.into(), batch_limit(options.delete_batch_size).into()],
    )
}

fn batch_limit(value: u64) -> i64 {
    value.try_into().expect("request record cleanup batch size must fit into i64")
}

fn placeholders(count: usize) -> String {
    (1..=count).map(|index| format!("${index}")).collect::<Vec<_>>().join(", ")
}

fn request_id_values(request_ids: &[String]) -> Vec<sea_orm::Value> {
    request_ids.iter().cloned().map(Into::into).collect()
}

fn count_overflow(error: std::num::TryFromIntError) -> StorageError {
    StorageError::Database(format!("request record cleanup count conversion failed: {error}"))
}

#[derive(Debug, FromQueryResult)]
struct RequestIdRow {
    request_id: String,
}

#[derive(Debug, FromQueryResult)]
struct DeleteCandidatesRow {
    deleted_candidates: i64,
}

#[derive(Debug, FromQueryResult)]
struct DeleteRecordsRow {
    deleted_records: i64,
}

#[derive(Debug, FromQueryResult)]
struct DeleteOrphanCandidateBatchRow {
    deleted_candidates: i64,
}

#[derive(Debug, FromQueryResult)]
struct DeleteRoutingDecisionBatchRow {
    deleted_routing_decisions: i64,
}

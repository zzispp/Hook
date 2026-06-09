use sea_orm::{ColumnTrait, DbBackend, EntityTrait, FromQueryResult, QueryFilter, Statement, TransactionTrait, sea_query::Expr};

use crate::StorageResult;

use super::{
    record::{request_candidates, request_records},
    repository::ProviderStore,
    request_record_housekeeping::{CompressBatchResult, RequestRecordCleanupOptions},
    request_record_housekeeping_timeout::{CleanupBudget, apply_timeouts},
    request_record_payload_codec,
};

pub(super) async fn compress_record_batch(
    store: &ProviderStore,
    options: &RequestRecordCleanupOptions,
    budget: &CleanupBudget,
) -> StorageResult<CompressBatchResult> {
    if budget.exhausted() {
        return Ok(exhausted_batch());
    }
    let records = summary_payload_batch(store, options, budget).await?;
    let scanned = records.len() as u64;
    let mut changed = 0;
    for record in records {
        if budget.exhausted() {
            return Ok(CompressBatchResult {
                scanned,
                changed,
                time_budget_exhausted: true,
            });
        }
        changed += u64::from(update_summary_payloads(store, options, budget, record).await?);
    }
    Ok(CompressBatchResult {
        scanned,
        changed,
        time_budget_exhausted: budget.exhausted(),
    })
}

pub(super) async fn compress_candidate_batch(
    store: &ProviderStore,
    options: &RequestRecordCleanupOptions,
    budget: &CleanupBudget,
) -> StorageResult<CompressBatchResult> {
    if budget.exhausted() {
        return Ok(exhausted_batch());
    }
    let records = candidate_payload_batch(store, options, budget).await?;
    let scanned = records.len() as u64;
    let mut changed = 0;
    for record in records {
        if budget.exhausted() {
            return Ok(CompressBatchResult {
                scanned,
                changed,
                time_budget_exhausted: true,
            });
        }
        changed += u64::from(update_candidate_payloads(store, options, budget, record).await?);
    }
    Ok(CompressBatchResult {
        scanned,
        changed,
        time_budget_exhausted: budget.exhausted(),
    })
}

async fn summary_payload_batch(store: &ProviderStore, options: &RequestRecordCleanupOptions, budget: &CleanupBudget) -> StorageResult<Vec<SummaryPayloadRow>> {
    let tx = store.connection().begin().await?;
    apply_timeouts(&tx, options, budget).await?;
    let records = SummaryPayloadRow::find_by_statement(summary_payload_batch_statement(options)).all(&tx).await?;
    tx.commit().await?;
    Ok(records)
}

async fn candidate_payload_batch(
    store: &ProviderStore,
    options: &RequestRecordCleanupOptions,
    budget: &CleanupBudget,
) -> StorageResult<Vec<CandidatePayloadRow>> {
    let tx = store.connection().begin().await?;
    apply_timeouts(&tx, options, budget).await?;
    let records = CandidatePayloadRow::find_by_statement(candidate_payload_batch_statement(options))
        .all(&tx)
        .await?;
    tx.commit().await?;
    Ok(records)
}

async fn update_summary_payloads(
    store: &ProviderStore,
    options: &RequestRecordCleanupOptions,
    budget: &CleanupBudget,
    record: SummaryPayloadRow,
) -> StorageResult<bool> {
    let update = compressed_summary_payloads(record)?;
    let tx = store.connection().begin().await?;
    apply_timeouts(&tx, options, budget).await?;
    let rows_affected = update_summary_row(update, &tx).await?;
    tx.commit().await?;
    Ok(rows_affected > 0)
}

async fn update_candidate_payloads(
    store: &ProviderStore,
    options: &RequestRecordCleanupOptions,
    budget: &CleanupBudget,
    record: CandidatePayloadRow,
) -> StorageResult<bool> {
    let update = compressed_candidate_payloads(record)?;
    let tx = store.connection().begin().await?;
    apply_timeouts(&tx, options, budget).await?;
    let rows_affected = update_candidate_row(update, &tx).await?;
    tx.commit().await?;
    Ok(rows_affected > 0)
}

async fn update_summary_row(update: SummaryPayloadUpdate, tx: &sea_orm::DatabaseTransaction) -> StorageResult<u64> {
    let now = time::OffsetDateTime::now_utc();
    let mut query = request_records::Entity::update_many()
        .col_expr(request_records::Column::PayloadCompressedAt, Expr::val(Some(now)))
        .filter(request_records::Column::RequestId.eq(update.request_id))
        .filter(request_records::Column::PayloadCompressedAt.is_null());
    if update.payload_changed {
        query = query
            .col_expr(request_records::Column::RequestHeaders, Expr::val(update.request_headers))
            .col_expr(request_records::Column::RequestBody, Expr::val(update.request_body))
            .col_expr(request_records::Column::ClientResponseHeaders, Expr::val(update.client_response_headers))
            .col_expr(request_records::Column::ClientResponseBody, Expr::val(update.client_response_body));
    }
    query.exec(tx).await.map(|result| result.rows_affected).map_err(Into::into)
}

async fn update_candidate_row(update: CandidatePayloadUpdate, tx: &sea_orm::DatabaseTransaction) -> StorageResult<u64> {
    let now = time::OffsetDateTime::now_utc();
    let mut query = request_candidates::Entity::update_many()
        .col_expr(request_candidates::Column::PayloadCompressedAt, Expr::val(Some(now)))
        .filter(request_candidates::Column::Id.eq(update.id))
        .filter(request_candidates::Column::PayloadCompressedAt.is_null());
    if update.payload_changed {
        query = query
            .col_expr(request_candidates::Column::ProviderRequestHeaders, Expr::val(update.provider_request_headers))
            .col_expr(request_candidates::Column::ProviderRequestBody, Expr::val(update.provider_request_body))
            .col_expr(request_candidates::Column::ProviderResponseHeaders, Expr::val(update.provider_response_headers))
            .col_expr(request_candidates::Column::ProviderResponseBody, Expr::val(update.provider_response_body));
    }
    query.exec(tx).await.map(|result| result.rows_affected).map_err(Into::into)
}

fn compressed_summary_payloads(record: SummaryPayloadRow) -> StorageResult<SummaryPayloadUpdate> {
    let request_headers = request_record_payload_codec::compress_payload(record.request_headers.clone())?;
    let request_body = request_record_payload_codec::compress_payload(record.request_body.clone())?;
    let response_headers = request_record_payload_codec::compress_payload(record.client_response_headers.clone())?;
    let response_body = request_record_payload_codec::compress_payload(record.client_response_body.clone())?;
    let payload_changed = !summary_payloads_unchanged(&record, &request_headers, &request_body, &response_headers, &response_body);
    Ok(SummaryPayloadUpdate {
        request_id: record.request_id,
        request_headers,
        request_body,
        client_response_headers: response_headers,
        client_response_body: response_body,
        payload_changed,
    })
}

fn compressed_candidate_payloads(record: CandidatePayloadRow) -> StorageResult<CandidatePayloadUpdate> {
    let request_headers = request_record_payload_codec::compress_payload(record.provider_request_headers.clone())?;
    let request_body = request_record_payload_codec::compress_payload(record.provider_request_body.clone())?;
    let response_headers = request_record_payload_codec::compress_payload(record.provider_response_headers.clone())?;
    let response_body = request_record_payload_codec::compress_payload(record.provider_response_body.clone())?;
    let payload_changed = !candidate_payloads_unchanged(&record, &request_headers, &request_body, &response_headers, &response_body);
    Ok(CandidatePayloadUpdate {
        id: record.id,
        provider_request_headers: request_headers,
        provider_request_body: request_body,
        provider_response_headers: response_headers,
        provider_response_body: response_body,
        payload_changed,
    })
}

fn summary_payloads_unchanged(
    record: &SummaryPayloadRow,
    request_headers: &Option<String>,
    request_body: &Option<String>,
    response_headers: &Option<String>,
    response_body: &Option<String>,
) -> bool {
    request_headers == &record.request_headers
        && request_body == &record.request_body
        && response_headers == &record.client_response_headers
        && response_body == &record.client_response_body
}

fn candidate_payloads_unchanged(
    record: &CandidatePayloadRow,
    request_headers: &Option<String>,
    request_body: &Option<String>,
    response_headers: &Option<String>,
    response_body: &Option<String>,
) -> bool {
    request_headers == &record.provider_request_headers
        && request_body == &record.provider_request_body
        && response_headers == &record.provider_response_headers
        && response_body == &record.provider_response_body
}

fn exhausted_batch() -> CompressBatchResult {
    CompressBatchResult {
        scanned: 0,
        changed: 0,
        time_budget_exhausted: true,
    }
}

fn summary_payload_batch_statement(options: &RequestRecordCleanupOptions) -> Statement {
    Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT request_id, request_headers, request_body, client_response_headers, client_response_body \
         FROM request_records \
         WHERE created_at < $1 \
           AND payload_compressed_at IS NULL \
           AND (request_headers IS NOT NULL OR request_body IS NOT NULL OR client_response_headers IS NOT NULL OR client_response_body IS NOT NULL) \
         ORDER BY created_at ASC, request_id ASC \
         LIMIT $2 FOR UPDATE SKIP LOCKED",
        vec![options.payload_cutoff.into(), (options.compress_batch_size as i64).into()],
    )
}

fn candidate_payload_batch_statement(options: &RequestRecordCleanupOptions) -> Statement {
    Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT id, provider_request_headers, provider_request_body, provider_response_headers, provider_response_body \
         FROM request_candidates \
         WHERE created_at < $1 \
           AND payload_compressed_at IS NULL \
           AND (provider_request_headers IS NOT NULL OR provider_request_body IS NOT NULL OR provider_response_headers IS NOT NULL OR provider_response_body IS NOT NULL) \
         ORDER BY created_at ASC, id ASC \
         LIMIT $2 FOR UPDATE SKIP LOCKED",
        vec![options.payload_cutoff.into(), (options.compress_batch_size as i64).into()],
    )
}

#[derive(Debug, FromQueryResult)]
struct SummaryPayloadRow {
    request_id: String,
    request_headers: Option<String>,
    request_body: Option<String>,
    client_response_headers: Option<String>,
    client_response_body: Option<String>,
}

#[derive(Debug, FromQueryResult)]
struct CandidatePayloadRow {
    id: String,
    provider_request_headers: Option<String>,
    provider_request_body: Option<String>,
    provider_response_headers: Option<String>,
    provider_response_body: Option<String>,
}

struct SummaryPayloadUpdate {
    request_id: String,
    request_headers: Option<String>,
    request_body: Option<String>,
    client_response_headers: Option<String>,
    client_response_body: Option<String>,
    payload_changed: bool,
}

struct CandidatePayloadUpdate {
    id: String,
    provider_request_headers: Option<String>,
    provider_request_body: Option<String>,
    provider_response_headers: Option<String>,
    provider_response_body: Option<String>,
    payload_changed: bool,
}

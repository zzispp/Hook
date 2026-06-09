use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, Value};

use crate::{StorageError, StorageResult};

use super::{
    ProviderStore, RequestPayloadOwner, RequestPayloadPendingInput, RequestPayloadStoreInput, request_record_payload_codec, request_record_payload_data,
    request_record_payload_store::{
        KIND_CLIENT_RESPONSE_BODY, KIND_CLIENT_RESPONSE_HEADERS, KIND_PROVIDER_REQUEST_BODY, KIND_PROVIDER_REQUEST_HEADERS, KIND_PROVIDER_RESPONSE_BODY,
        KIND_PROVIDER_RESPONSE_HEADERS, KIND_REQUEST_BODY, KIND_REQUEST_HEADERS, OWNER_REQUEST_CANDIDATE, OWNER_REQUEST_RECORD,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestPayloadBackfillOptions {
    pub batch_size: u64,
    pub minimum_created_at: time::OffsetDateTime,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RequestPayloadBackfillResult {
    pub records_backfilled: u64,
    pub candidates_backfilled: u64,
}

#[derive(Debug, FromQueryResult)]
struct LegacyRecordPayloadRow {
    request_id: String,
    request_headers: Option<String>,
    request_body: Option<String>,
    client_response_headers: Option<String>,
    client_response_body: Option<String>,
}

#[derive(Debug, FromQueryResult)]
struct LegacyCandidatePayloadRow {
    id: String,
    provider_request_headers: Option<String>,
    provider_request_body: Option<String>,
    provider_response_headers: Option<String>,
    provider_response_body: Option<String>,
}

pub async fn backfill_legacy_payloads(store: &ProviderStore, options: RequestPayloadBackfillOptions) -> StorageResult<RequestPayloadBackfillResult> {
    validate_options(options)?;
    let records = backfill_records(store, options).await?;
    let candidates = backfill_candidates(store, options).await?;
    Ok(RequestPayloadBackfillResult {
        records_backfilled: records,
        candidates_backfilled: candidates,
    })
}

async fn backfill_records(store: &ProviderStore, options: RequestPayloadBackfillOptions) -> StorageResult<u64> {
    let rows = LegacyRecordPayloadRow::find_by_statement(record_batch_statement(options)?)
        .all(store.connection())
        .await?;
    for row in &rows {
        backfill_record_row(store, row).await?;
    }
    Ok(rows.len() as u64)
}

async fn backfill_candidates(store: &ProviderStore, options: RequestPayloadBackfillOptions) -> StorageResult<u64> {
    let rows = LegacyCandidatePayloadRow::find_by_statement(candidate_batch_statement(options)?)
        .all(store.connection())
        .await?;
    for row in &rows {
        backfill_candidate_row(store, row).await?;
    }
    Ok(rows.len() as u64)
}

async fn backfill_record_row(store: &ProviderStore, row: &LegacyRecordPayloadRow) -> StorageResult<()> {
    super::request_record_partition_write::sync_request_record(store, &row.request_id).await?;
    let owner = RequestPayloadOwner {
        owner_type: OWNER_REQUEST_RECORD.to_owned(),
        owner_id: row.request_id.clone(),
    };
    store_legacy_payloads(store, owner, record_payload_fields(row)).await?;
    execute_statement(store, clear_record_payload_sql(), [Value::from(row.request_id.clone())]).await
}

async fn backfill_candidate_row(store: &ProviderStore, row: &LegacyCandidatePayloadRow) -> StorageResult<()> {
    super::request_record_partition_write::sync_request_candidate(store, &row.id).await?;
    let owner = RequestPayloadOwner {
        owner_type: OWNER_REQUEST_CANDIDATE.to_owned(),
        owner_id: row.id.clone(),
    };
    store_legacy_payloads(store, owner, candidate_payload_fields(row)).await?;
    execute_statement(store, clear_candidate_payload_sql(), [Value::from(row.id.clone())]).await
}

async fn store_legacy_payloads(store: &ProviderStore, owner: RequestPayloadOwner, fields: Vec<(&'static str, Option<String>)>) -> StorageResult<()> {
    for (kind, value) in fields {
        if let Some(text) = value {
            store_legacy_payload(store, owner.clone(), kind, text).await?;
        }
    }
    Ok(())
}

async fn store_legacy_payload(store: &ProviderStore, owner: RequestPayloadOwner, kind: &'static str, text: String) -> StorageResult<()> {
    let payload = request_record_payload_codec::decode_payload(Some(text))?.ok_or_else(|| StorageError::Database("legacy payload decoded to empty".into()))?;
    let key = super::request_record_payload_store::create_pending_payload(
        store,
        RequestPayloadPendingInput {
            owner,
            kind: kind.to_owned(),
            payload: payload.clone(),
        },
    )
    .await?;
    super::request_record_payload_store::store_payload(
        store,
        RequestPayloadStoreInput {
            key,
            data: request_record_payload_data::compress_json(&payload)?,
        },
    )
    .await
}

fn record_payload_fields(row: &LegacyRecordPayloadRow) -> Vec<(&'static str, Option<String>)> {
    vec![
        (KIND_REQUEST_HEADERS, row.request_headers.clone()),
        (KIND_REQUEST_BODY, row.request_body.clone()),
        (KIND_CLIENT_RESPONSE_HEADERS, row.client_response_headers.clone()),
        (KIND_CLIENT_RESPONSE_BODY, row.client_response_body.clone()),
    ]
}

fn candidate_payload_fields(row: &LegacyCandidatePayloadRow) -> Vec<(&'static str, Option<String>)> {
    vec![
        (KIND_PROVIDER_REQUEST_HEADERS, row.provider_request_headers.clone()),
        (KIND_PROVIDER_REQUEST_BODY, row.provider_request_body.clone()),
        (KIND_PROVIDER_RESPONSE_HEADERS, row.provider_response_headers.clone()),
        (KIND_PROVIDER_RESPONSE_BODY, row.provider_response_body.clone()),
    ]
}

fn record_batch_statement(options: RequestPayloadBackfillOptions) -> StorageResult<Statement> {
    Ok(Statement::from_sql_and_values(
        DbBackend::Postgres,
        record_batch_sql(),
        vec![Value::from(options.minimum_created_at), Value::from(batch_size_i64(options.batch_size)?)],
    ))
}

fn candidate_batch_statement(options: RequestPayloadBackfillOptions) -> StorageResult<Statement> {
    Ok(Statement::from_sql_and_values(
        DbBackend::Postgres,
        candidate_batch_sql(),
        vec![Value::from(options.minimum_created_at), Value::from(batch_size_i64(options.batch_size)?)],
    ))
}

fn record_batch_sql() -> &'static str {
    "SELECT request_id, request_headers, request_body, client_response_headers, client_response_body FROM request_records \
     WHERE created_at >= $1 AND (request_headers IS NOT NULL OR request_body IS NOT NULL OR client_response_headers IS NOT NULL OR client_response_body IS NOT NULL) \
     ORDER BY created_at ASC, request_id ASC LIMIT $2"
}

fn candidate_batch_sql() -> &'static str {
    "SELECT id, provider_request_headers, provider_request_body, provider_response_headers, provider_response_body FROM request_candidates \
     WHERE created_at >= $1 AND (provider_request_headers IS NOT NULL OR provider_request_body IS NOT NULL OR provider_response_headers IS NOT NULL OR provider_response_body IS NOT NULL) \
     ORDER BY created_at ASC, id ASC LIMIT $2"
}

fn clear_record_payload_sql() -> &'static str {
    "UPDATE request_records SET request_headers = NULL, request_body = NULL, client_response_headers = NULL, client_response_body = NULL, \
     payload_compressed_at = NOW() WHERE request_id = $1"
}

fn clear_candidate_payload_sql() -> &'static str {
    "UPDATE request_candidates SET provider_request_headers = NULL, provider_request_body = NULL, provider_response_headers = NULL, provider_response_body = NULL, \
     payload_compressed_at = NOW() WHERE id = $1"
}

fn validate_options(options: RequestPayloadBackfillOptions) -> StorageResult<()> {
    if options.batch_size == 0 || options.batch_size > i64::MAX as u64 {
        return Err(StorageError::Database(format!(
            "request payload backfill batch size must be between 1 and {}",
            i64::MAX
        )));
    }
    Ok(())
}

fn batch_size_i64(batch_size: u64) -> StorageResult<i64> {
    i64::try_from(batch_size).map_err(|_| StorageError::Database("request payload backfill batch size exceeds PostgreSQL integer range".into()))
}

async fn execute_statement<const N: usize>(store: &ProviderStore, sql: &'static str, values: [Value; N]) -> StorageResult<()> {
    store
        .connection()
        .execute_raw(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{candidate_batch_sql, record_batch_sql};

    #[test]
    fn backfill_reads_legacy_payloads_in_bounded_batches() {
        assert!(record_batch_sql().contains("LIMIT $2"));
        assert!(candidate_batch_sql().contains("LIMIT $2"));
        assert!(record_batch_sql().contains("created_at >= $1"));
        assert!(candidate_batch_sql().contains("created_at >= $1"));
    }
}

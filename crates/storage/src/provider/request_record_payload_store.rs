use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, Value};
use types::provider::{RequestPayloadMeta, RequestPayloadSource, RequestPayloadStatus};

use crate::{StorageError, StorageResult};

use super::{ProviderStore, request_record_detail::format_timestamp, request_record_payload_data};

const STATUS_PENDING: &str = "pending";
const STATUS_STORED: &str = "stored";
const STATUS_FAILED: &str = "failed";
const SOURCE_PARTITIONED: &str = "partitioned";
pub const OWNER_REQUEST_RECORD: &str = "request_record";
pub const OWNER_REQUEST_CANDIDATE: &str = "request_candidate";
pub const KIND_REQUEST_HEADERS: &str = "request_headers";
pub const KIND_REQUEST_BODY: &str = "request_body";
pub const KIND_CLIENT_RESPONSE_HEADERS: &str = "client_response_headers";
pub const KIND_CLIENT_RESPONSE_BODY: &str = "client_response_body";
pub const KIND_PROVIDER_REQUEST_HEADERS: &str = "provider_request_headers";
pub const KIND_PROVIDER_REQUEST_BODY: &str = "provider_request_body";
pub const KIND_PROVIDER_RESPONSE_HEADERS: &str = "provider_response_headers";
pub const KIND_PROVIDER_RESPONSE_BODY: &str = "provider_response_body";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestPayloadOwner {
    pub owner_type: String,
    pub owner_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RequestPayloadPendingInput {
    pub owner: RequestPayloadOwner,
    pub kind: String,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestPayloadKey {
    pub created_at: time::OffsetDateTime,
    pub owner: RequestPayloadOwner,
    pub kind: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RequestPayloadStoreInput {
    pub key: RequestPayloadKey,
    pub data: RequestPayloadData,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RequestPayloadData {
    pub original_size: i64,
    pub compressed_size: i64,
    pub sha256: String,
    pub compressed_payload: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoredRequestPayload {
    pub meta: RequestPayloadMeta,
    pub payload: Option<serde_json::Value>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RequestPayloadStaleSweepResult {
    pub failed_payloads: u64,
}

#[derive(FromQueryResult)]
struct PayloadRow {
    owner_type: String,
    owner_id: String,
    payload_kind: String,
    status: String,
    source: String,
    original_size: Option<i64>,
    compressed_size: Option<i64>,
    sha256: Option<String>,
    error_message: Option<String>,
    updated_at: time::OffsetDateTime,
    compressed_payload: Option<Vec<u8>>,
}

#[derive(FromQueryResult)]
struct PayloadKeyRow {
    created_at: time::OffsetDateTime,
}

pub async fn create_pending_payload(store: &ProviderStore, input: RequestPayloadPendingInput) -> StorageResult<RequestPayloadKey> {
    let size = payload_size(&input.payload)?;
    validate_owner_type(&input.owner.owner_type)?;
    let owner = input.owner;
    let kind = input.kind;
    let statement = Statement::from_sql_and_values(
        DbBackend::Postgres,
        pending_sql(&owner.owner_type),
        [
            Value::from(owner.owner_type.clone()),
            Value::from(owner.owner_id.clone()),
            Value::from(kind.clone()),
            Value::from(size),
        ],
    );
    let row = PayloadKeyRow::find_by_statement(statement)
        .one(store.connection())
        .await?
        .ok_or(StorageError::NotFound)?;
    Ok(RequestPayloadKey {
        created_at: row.created_at,
        owner,
        kind,
    })
}

pub async fn store_payload(store: &ProviderStore, input: RequestPayloadStoreInput) -> StorageResult<()> {
    let statement = Statement::from_sql_and_values(
        DbBackend::Postgres,
        stored_sql(),
        [
            Value::from(input.key.created_at),
            Value::from(input.key.owner.owner_type),
            Value::from(input.key.owner.owner_id),
            Value::from(input.key.kind),
            Value::from(input.data.original_size),
            Value::from(input.data.compressed_size),
            Value::from(input.data.sha256),
            Value::from(input.data.compressed_payload),
        ],
    );
    let result = store.connection().execute_raw(statement).await?;
    if result.rows_affected() == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

pub async fn mark_payload_failed(store: &ProviderStore, key: RequestPayloadKey, error: String) -> StorageResult<()> {
    let statement = Statement::from_sql_and_values(
        DbBackend::Postgres,
        failed_sql(),
        [
            Value::from(key.created_at),
            Value::from(key.owner.owner_type),
            Value::from(key.owner.owner_id),
            Value::from(key.kind),
            Value::from(error),
        ],
    );
    store.connection().execute_raw(statement).await?;
    Ok(())
}

pub async fn payloads_for_owner(store: &ProviderStore, owner: &RequestPayloadOwner) -> StorageResult<Vec<StoredRequestPayload>> {
    let statement = Statement::from_sql_and_values(
        DbBackend::Postgres,
        select_payloads_sql(),
        [Value::from(owner.owner_type.clone()), Value::from(owner.owner_id.clone())],
    );
    PayloadRow::find_by_statement(statement)
        .all(store.connection())
        .await?
        .into_iter()
        .map(stored_payload)
        .collect()
}

pub async fn mark_stale_payloads_failed(store: &ProviderStore, cutoff: time::OffsetDateTime) -> StorageResult<RequestPayloadStaleSweepResult> {
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, stale_sweep_sql(), [Value::from(cutoff)]);
    let result = store.connection().execute_raw(statement).await?;
    Ok(RequestPayloadStaleSweepResult {
        failed_payloads: result.rows_affected(),
    })
}

pub fn compress_payload(value: &serde_json::Value) -> StorageResult<RequestPayloadData> {
    request_record_payload_data::compress_json(value)
}

fn validate_owner_type(owner_type: &str) -> StorageResult<()> {
    match owner_type {
        OWNER_REQUEST_RECORD | OWNER_REQUEST_CANDIDATE => Ok(()),
        _ => Err(StorageError::Database(format!("unsupported request payload owner type: {owner_type}"))),
    }
}

fn pending_sql(owner_type: &str) -> &'static str {
    match owner_type {
        OWNER_REQUEST_RECORD => pending_record_sql(),
        OWNER_REQUEST_CANDIDATE => pending_candidate_sql(),
        _ => unreachable!("request payload owner type must be validated before SQL selection"),
    }
}

fn pending_record_sql() -> &'static str {
    "WITH owner_row AS (SELECT created_at FROM request_records_partitioned WHERE request_id = $2 \
     UNION ALL SELECT created_at FROM request_records WHERE request_id = $2 AND NOT EXISTS \
     (SELECT 1 FROM request_records_partitioned WHERE request_id = $2) LIMIT 1) \
     INSERT INTO request_payloads (created_at, owner_type, owner_id, payload_kind, status, source, original_size, compressed_size, sha256, compressed_payload, error_message, updated_at) \
     SELECT created_at, $1, $2, $3, 'pending', 'partitioned', $4, NULL, NULL, NULL, NULL, NOW() FROM owner_row \
     ON CONFLICT (created_at, owner_type, owner_id, payload_kind) DO UPDATE SET status = 'pending', original_size = EXCLUDED.original_size, \
     compressed_size = NULL, sha256 = NULL, compressed_payload = NULL, error_message = NULL, updated_at = NOW() RETURNING created_at"
}

fn pending_candidate_sql() -> &'static str {
    "WITH owner_row AS (SELECT created_at FROM request_candidates_partitioned WHERE id = $2 \
     UNION ALL SELECT created_at FROM request_candidates WHERE id = $2 AND NOT EXISTS \
     (SELECT 1 FROM request_candidates_partitioned WHERE id = $2) LIMIT 1) \
     INSERT INTO request_payloads (created_at, owner_type, owner_id, payload_kind, status, source, original_size, compressed_size, sha256, compressed_payload, error_message, updated_at) \
     SELECT created_at, $1, $2, $3, 'pending', 'partitioned', $4, NULL, NULL, NULL, NULL, NOW() FROM owner_row \
     ON CONFLICT (created_at, owner_type, owner_id, payload_kind) DO UPDATE SET status = 'pending', original_size = EXCLUDED.original_size, \
     compressed_size = NULL, sha256 = NULL, compressed_payload = NULL, error_message = NULL, updated_at = NOW() RETURNING created_at"
}

fn stored_sql() -> &'static str {
    "UPDATE request_payloads SET status = 'stored', source = 'partitioned', original_size = $5, compressed_size = $6, sha256 = $7, \
     compressed_payload = $8, error_message = NULL, updated_at = NOW() WHERE created_at = $1 AND owner_type = $2 AND owner_id = $3 AND payload_kind = $4"
}

fn failed_sql() -> &'static str {
    "UPDATE request_payloads SET status = 'failed', error_message = $5, updated_at = NOW() \
     WHERE created_at = $1 AND owner_type = $2 AND owner_id = $3 AND payload_kind = $4"
}

fn select_payloads_sql() -> &'static str {
    "SELECT owner_type, owner_id, payload_kind, status, source, original_size, compressed_size, sha256, error_message, updated_at, compressed_payload \
     FROM request_payloads WHERE owner_type = $1 AND owner_id = $2 ORDER BY payload_kind ASC"
}

fn stale_sweep_sql() -> &'static str {
    "UPDATE request_payloads SET status = 'failed', error_message = 'payload writer did not store payload before stale timeout', updated_at = NOW() \
     WHERE status = 'pending' AND updated_at < $1"
}

fn stored_payload(row: PayloadRow) -> StorageResult<StoredRequestPayload> {
    Ok(StoredRequestPayload {
        meta: payload_meta(&row),
        payload: row.compressed_payload.as_deref().map(request_record_payload_data::decode_json).transpose()?,
    })
}

fn payload_meta(row: &PayloadRow) -> RequestPayloadMeta {
    RequestPayloadMeta {
        owner_type: row.owner_type.clone(),
        owner_id: row.owner_id.clone(),
        kind: row.payload_kind.clone(),
        status: payload_status(&row.status),
        source: payload_source(&row.source),
        original_size: row.original_size,
        compressed_size: row.compressed_size,
        sha256: row.sha256.clone(),
        error_message: row.error_message.clone(),
        updated_at: format_timestamp(row.updated_at),
    }
}

fn payload_status(value: &str) -> RequestPayloadStatus {
    match value {
        STATUS_PENDING => RequestPayloadStatus::Pending,
        STATUS_STORED => RequestPayloadStatus::Stored,
        STATUS_FAILED => RequestPayloadStatus::Failed,
        _ => RequestPayloadStatus::Pending,
    }
}

fn payload_source(value: &str) -> RequestPayloadSource {
    match value {
        SOURCE_PARTITIONED => RequestPayloadSource::Partitioned,
        _ => RequestPayloadSource::Legacy,
    }
}

fn payload_size(value: &serde_json::Value) -> StorageResult<i64> {
    request_record_payload_data::json_size(value)
}

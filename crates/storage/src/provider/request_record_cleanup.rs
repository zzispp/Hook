use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use time::OffsetDateTime;

use crate::StorageResult;

use super::{
    record::{request_candidates, request_records},
    repository::ProviderStore,
    request_record_payload_codec,
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

pub async fn compress_request_record_payloads_before(store: &ProviderStore, cutoff: OffsetDateTime) -> StorageResult<u64> {
    let summaries = request_records::Entity::find()
        .filter(request_records::Column::CreatedAt.lt(cutoff))
        .filter(summary_payload_exists())
        .all(store.connection())
        .await?;
    let mut changed = 0;
    for record in summaries {
        changed += u64::from(compress_summary_payloads(store, record).await?);
    }
    let candidates = request_candidates::Entity::find()
        .filter(request_candidates::Column::CreatedAt.lt(cutoff))
        .filter(candidate_payload_exists())
        .all(store.connection())
        .await?;
    for record in candidates {
        changed += u64::from(compress_candidate_payloads(store, record).await?);
    }
    Ok(changed)
}

fn summary_payload_exists() -> Condition {
    Condition::any()
        .add(request_records::Column::RequestHeaders.is_not_null())
        .add(request_records::Column::RequestBody.is_not_null())
        .add(request_records::Column::ClientResponseHeaders.is_not_null())
        .add(request_records::Column::ClientResponseBody.is_not_null())
}

fn candidate_payload_exists() -> Condition {
    Condition::any()
        .add(request_candidates::Column::ProviderRequestHeaders.is_not_null())
        .add(request_candidates::Column::ProviderRequestBody.is_not_null())
        .add(request_candidates::Column::ProviderResponseHeaders.is_not_null())
        .add(request_candidates::Column::ProviderResponseBody.is_not_null())
}

async fn compress_summary_payloads(store: &ProviderStore, record: request_records::Model) -> StorageResult<bool> {
    let request_headers = request_record_payload_codec::compress_payload(record.request_headers.clone())?;
    let request_body = request_record_payload_codec::compress_payload(record.request_body.clone())?;
    let client_response_headers = request_record_payload_codec::compress_payload(record.client_response_headers.clone())?;
    let client_response_body = request_record_payload_codec::compress_payload(record.client_response_body.clone())?;
    if request_headers == record.request_headers
        && request_body == record.request_body
        && client_response_headers == record.client_response_headers
        && client_response_body == record.client_response_body
    {
        return Ok(false);
    }
    let mut active: request_records::ActiveModel = record.into();
    active.request_headers = Set(request_headers);
    active.request_body = Set(request_body);
    active.client_response_headers = Set(client_response_headers);
    active.client_response_body = Set(client_response_body);
    active.update(store.connection()).await?;
    Ok(true)
}

async fn compress_candidate_payloads(store: &ProviderStore, record: request_candidates::Model) -> StorageResult<bool> {
    let provider_request_headers = request_record_payload_codec::compress_payload(record.provider_request_headers.clone())?;
    let provider_request_body = request_record_payload_codec::compress_payload(record.provider_request_body.clone())?;
    let provider_response_headers = request_record_payload_codec::compress_payload(record.provider_response_headers.clone())?;
    let provider_response_body = request_record_payload_codec::compress_payload(record.provider_response_body.clone())?;
    if provider_request_headers == record.provider_request_headers
        && provider_request_body == record.provider_request_body
        && provider_response_headers == record.provider_response_headers
        && provider_response_body == record.provider_response_body
    {
        return Ok(false);
    }
    let mut active: request_candidates::ActiveModel = record.into();
    active.provider_request_headers = Set(provider_request_headers);
    active.provider_request_body = Set(provider_request_body);
    active.provider_response_headers = Set(provider_response_headers);
    active.provider_response_body = Set(provider_response_body);
    active.update(store.connection()).await?;
    Ok(true)
}

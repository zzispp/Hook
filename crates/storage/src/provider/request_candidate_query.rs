use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use types::provider::RequestCandidateListRequest;

use crate::StorageResult;

use super::{RequestCandidateRecordInput, record::request_candidates, repository::ProviderStore};

pub async fn create_request_candidate(store: &ProviderStore, input: RequestCandidateRecordInput) -> StorageResult<types::provider::RequestCandidate> {
    let now = time::OffsetDateTime::now_utc();
    let record = request_candidates::ActiveModel {
        id: Set(store.next_id()),
        request_id: Set(input.request_id),
        token_id: Set(input.token_id),
        group_code: Set(input.group_code),
        global_model_id: Set(input.global_model_id),
        provider_id: Set(input.provider_id),
        endpoint_id: Set(input.endpoint_id),
        key_id: Set(input.key_id),
        client_api_format: Set(input.client_api_format),
        provider_api_format: Set(input.provider_api_format),
        needs_conversion: Set(input.needs_conversion),
        is_stream: Set(input.is_stream),
        candidate_index: Set(input.candidate_index),
        retry_index: Set(input.retry_index),
        status: Set(input.status),
        status_code: Set(input.status_code),
        latency_ms: Set(input.latency_ms),
        first_byte_time_ms: Set(input.first_byte_time_ms),
        error_type: Set(input.error_type),
        error_message: Set(input.error_message),
        created_at: Set(now),
        started_at: Set(input.started.then_some(now)),
        finished_at: Set(input.finished.then_some(now)),
    }
    .insert(store.connection())
    .await?;
    Ok(record.response())
}

pub async fn list_request_candidates(store: &ProviderStore, request: RequestCandidateListRequest) -> StorageResult<Vec<types::provider::RequestCandidate>> {
    let mut query = request_candidates::Entity::find()
        .order_by_asc(request_candidates::Column::CandidateIndex)
        .order_by_asc(request_candidates::Column::RetryIndex);
    if let Some(request_id) = request.request_id {
        query = query.filter(request_candidates::Column::RequestId.eq(request_id));
    }
    let records = query.offset(request.skip).limit(request.limit).all(store.connection()).await?;
    Ok(records.into_iter().map(|record| record.response()).collect())
}

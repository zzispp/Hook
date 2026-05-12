use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use types::provider::RequestCandidateListRequest;

use crate::{StorageError, StorageResult, json};

use super::{RequestCandidateRecordInput, RequestCandidateRecordPatch, record::request_candidates, repository::ProviderStore};

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
        request_headers: Set(json::encode_optional(&input.request_headers)?),
        request_body: Set(json::encode_optional(&input.request_body)?),
        response_body: Set(json::encode_optional(&input.response_body)?),
        candidate_index: Set(input.candidate_index),
        retry_index: Set(input.retry_index),
        status: Set(input.status),
        status_code: Set(input.status_code),
        prompt_tokens: Set(input.prompt_tokens),
        completion_tokens: Set(input.completion_tokens),
        total_tokens: Set(input.total_tokens),
        cache_creation_input_tokens: Set(input.cache_creation_input_tokens),
        cache_read_input_tokens: Set(input.cache_read_input_tokens),
        cost_currency: Set(input.cost_currency),
        token_cost: Set(input.token_cost),
        base_cost: Set(input.base_cost),
        total_cost: Set(input.total_cost),
        billing_multiplier: Set(input.billing_multiplier),
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

pub async fn update_request_candidate(store: &ProviderStore, input: RequestCandidateRecordPatch) -> StorageResult<types::provider::RequestCandidate> {
    let Some(record) = request_candidates::Entity::find()
        .filter(request_candidates::Column::RequestId.eq(&input.request_id))
        .filter(request_candidates::Column::CandidateIndex.eq(input.candidate_index))
        .filter(request_candidates::Column::RetryIndex.eq(input.retry_index))
        .one(store.connection())
        .await?
    else {
        return Err(StorageError::NotFound);
    };
    let was_started = record.started_at.is_some();
    let now = time::OffsetDateTime::now_utc();
    let mut record: request_candidates::ActiveModel = record.into();
    record.status = Set(input.status);
    record.status_code = Set(input.status_code);
    record.prompt_tokens = Set(input.prompt_tokens);
    record.completion_tokens = Set(input.completion_tokens);
    record.total_tokens = Set(input.total_tokens);
    record.cache_creation_input_tokens = Set(input.cache_creation_input_tokens);
    record.cache_read_input_tokens = Set(input.cache_read_input_tokens);
    record.cost_currency = Set(input.cost_currency);
    record.token_cost = Set(input.token_cost);
    record.base_cost = Set(input.base_cost);
    record.total_cost = Set(input.total_cost);
    record.billing_multiplier = Set(input.billing_multiplier);
    record.latency_ms = Set(input.latency_ms);
    record.first_byte_time_ms = Set(input.first_byte_time_ms);
    record.error_type = Set(input.error_type);
    record.error_message = Set(input.error_message);
    record.response_body = Set(json::encode_optional(&input.response_body)?);
    if !was_started {
        record.started_at = Set(Some(now));
    }
    if input.finished {
        record.finished_at = Set(Some(now));
    }
    Ok(record.update(store.connection()).await?.response())
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

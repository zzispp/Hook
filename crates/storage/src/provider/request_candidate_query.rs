use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, sea_query::Expr};

use crate::{StorageError, StorageResult, json};

use super::{
    RequestBillingRecordValues, RequestCandidateRecordInput, RequestCandidateRecordPatch, record::request_candidates, repository::ProviderStore,
    request_candidate_update::candidate_update, request_record_write::ensure_accounting_cost_currency, request_upstream_cost,
};

pub async fn create_request_candidate(store: &ProviderStore, input: RequestCandidateRecordInput) -> StorageResult<types::provider::RequestCandidate> {
    let now = time::OffsetDateTime::now_utc();
    let mut record = request_candidates::ActiveModel {
        id: Set(store.next_id()),
        request_id: Set(input.request_id),
        token_id: Set(input.token_id),
        group_code: Set(input.group_code),
        global_model_id: Set(input.global_model_id),
        provider_id: Set(input.provider_id),
        provider_name_snapshot: Set(input.provider_name_snapshot),
        endpoint_id: Set(input.endpoint_id),
        endpoint_name_snapshot: Set(input.endpoint_name_snapshot),
        key_id: Set(input.key_id),
        key_name_snapshot: Set(input.key_name_snapshot),
        key_preview_snapshot: Set(input.key_preview_snapshot),
        client_api_format: Set(input.client_api_format),
        provider_api_format: Set(input.provider_api_format),
        needs_conversion: Set(input.needs_conversion),
        is_stream: Set(input.is_stream),
        is_cached: Set(input.is_cached),
        provider_request_headers: Set(None),
        provider_request_body: Set(None),
        provider_response_headers: Set(None),
        provider_response_body: Set(None),
        payload_compressed_at: Set(None),
        candidate_index: Set(input.candidate_index),
        retry_index: Set(input.retry_index),
        status: Set(input.status),
        skip_reason: Set(input.skip_reason),
        status_code: Set(input.status_code),
        prompt_tokens: Set(input.prompt_tokens),
        completion_tokens: Set(input.completion_tokens),
        total_tokens: Set(input.total_tokens),
        cache_creation_input_tokens: Set(input.cache_creation_input_tokens),
        cache_read_input_tokens: Set(input.cache_read_input_tokens),
        input_text_tokens: Set(input.input_text_tokens),
        input_audio_tokens: Set(input.input_audio_tokens),
        input_image_tokens: Set(input.input_image_tokens),
        output_text_tokens: Set(input.output_text_tokens),
        output_audio_tokens: Set(input.output_audio_tokens),
        output_image_tokens: Set(input.output_image_tokens),
        reasoning_tokens: Set(input.reasoning_tokens),
        cache_creation_5m_input_tokens: Set(input.cache_creation_5m_input_tokens),
        cache_creation_1h_input_tokens: Set(input.cache_creation_1h_input_tokens),
        usage_source: Set(input.usage_source),
        usage_semantic: Set(input.usage_semantic),
        service_tier: Set(None),
        upstream_cost_mode: Set(None),
        upstream_cost_source: Set(None),
        upstream_price_per_request: Set(None),
        upstream_input_price_per_million: Set(None),
        upstream_output_price_per_million: Set(None),
        upstream_cache_creation_price_per_million: Set(None),
        upstream_cache_read_price_per_million: Set(None),
        upstream_request_cost: Set(None),
        upstream_input_cost: Set(None),
        upstream_output_cost: Set(None),
        upstream_cache_creation_cost: Set(None),
        upstream_cache_read_cost: Set(None),
        upstream_total_cost: Set(None),
        cost_currency: Set(None),
        input_cost: Set(None),
        output_cost: Set(None),
        cache_creation_cost: Set(None),
        cache_read_cost: Set(None),
        request_cost: Set(None),
        token_cost: Set(None),
        base_cost: Set(None),
        total_cost: Set(None),
        billing_multiplier: Set(None),
        input_price_per_million: Set(None),
        output_price_per_million: Set(None),
        cache_creation_price_per_million: Set(None),
        cache_read_price_per_million: Set(None),
        billing_snapshot: Set(json::encode_optional(&input.billing_snapshot)?),
        latency_ms: Set(input.latency_ms),
        first_byte_time_ms: Set(input.first_byte_time_ms),
        error_type: Set(input.error_type),
        error_message: Set(input.error_message),
        error_code: Set(input.error_code),
        error_param: Set(input.error_param),
        created_at: Set(now),
        started_at: Set(input.started.then_some(now)),
        finished_at: Set(input.finished.then_some(now)),
    };
    request_upstream_cost::apply_candidate_values(&mut record, input.upstream_cost);
    apply_billing_values(&mut record, input.billing)?;
    let record = record.insert(store.connection()).await?;
    super::request_record_partition_write::sync_request_candidate(store, &record.id).await?;
    record.response()
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
    let record_id = record.id.clone();
    let was_started = record.started_at.is_some();
    let now = time::OffsetDateTime::now_utc();
    let update = candidate_update(input, now, was_started)?;
    let result = update.filter(request_candidates::Column::Id.eq(&record_id)).exec(store.connection()).await?;
    if result.rows_affected == 0 {
        return Err(StorageError::NotFound);
    }
    let record = request_candidates::Entity::find_by_id(record_id)
        .one(store.connection())
        .await?
        .ok_or(StorageError::NotFound)?;
    super::request_record_partition_write::sync_request_candidate(store, &record.id).await?;
    record.response()
}

fn apply_billing_values(record: &mut request_candidates::ActiveModel, billing: RequestBillingRecordValues) -> StorageResult<()> {
    if let Some(service_tier) = billing.service_tier {
        record.service_tier = Set(Some(service_tier));
    }
    record.cost_currency = Set(ensure_accounting_cost_currency(billing.cost_currency)?);
    record.input_cost = Set(billing.input_cost);
    record.output_cost = Set(billing.output_cost);
    record.cache_creation_cost = Set(billing.cache_creation_cost);
    record.cache_read_cost = Set(billing.cache_read_cost);
    record.request_cost = Set(billing.request_cost);
    record.token_cost = Set(billing.token_cost);
    record.base_cost = Set(billing.base_cost);
    record.total_cost = Set(billing.total_cost);
    record.billing_multiplier = Set(billing.billing_multiplier);
    record.input_price_per_million = Set(billing.input_price_per_million);
    record.output_price_per_million = Set(billing.output_price_per_million);
    record.cache_creation_price_per_million = Set(billing.cache_creation_price_per_million);
    record.cache_read_price_per_million = Set(billing.cache_read_price_per_million);
    Ok(())
}

pub async fn mark_scheduled_request_candidates_skipped(store: &ProviderStore, request_id: &str, skip_reason: &str) -> StorageResult<u64> {
    let now = time::OffsetDateTime::now_utc();
    let result = request_candidates::Entity::update_many()
        .col_expr(request_candidates::Column::Status, Expr::val("skipped"))
        .col_expr(request_candidates::Column::SkipReason, Expr::val(skip_reason))
        .col_expr(request_candidates::Column::FinishedAt, Expr::val(now))
        .filter(request_candidates::Column::RequestId.eq(request_id))
        .filter(request_candidates::Column::Status.eq("scheduled"))
        .exec(store.connection())
        .await?;
    if result.rows_affected > 0 {
        super::request_record_partition_write::sync_request_candidates_for_request(store, request_id).await?;
    }
    Ok(result.rows_affected)
}

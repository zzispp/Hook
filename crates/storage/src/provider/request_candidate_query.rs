use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set, UpdateMany, sea_query::Expr};
use types::{model::PatchField, provider::RequestCandidateListRequest};

use crate::{StorageError, StorageResult, json};

use super::{
    RequestBillingRecordValues, RequestCandidateRecordInput, RequestCandidateRecordPatch, record::request_candidates, repository::ProviderStore,
    request_record_write::ensure_accounting_cost_currency, request_upstream_cost,
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
        provider_request_headers: Set(json::encode_optional(&input.provider_request_headers)?),
        provider_request_body: Set(json::encode_optional(&input.provider_request_body)?),
        provider_response_headers: Set(json::encode_optional(&input.provider_response_headers)?),
        provider_response_body: Set(json::encode_optional(&input.provider_response_body)?),
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
    record.response()
}

fn candidate_update(input: RequestCandidateRecordPatch, now: time::OffsetDateTime, was_started: bool) -> StorageResult<UpdateMany<request_candidates::Entity>> {
    let mut update = request_candidates::Entity::update_many()
        .col_expr(request_candidates::Column::Status, Expr::val(input.status))
        .col_expr(request_candidates::Column::SkipReason, Expr::val(input.skip_reason))
        .col_expr(request_candidates::Column::StatusCode, Expr::val(input.status_code))
        .col_expr(request_candidates::Column::PromptTokens, Expr::val(input.prompt_tokens))
        .col_expr(request_candidates::Column::CompletionTokens, Expr::val(input.completion_tokens))
        .col_expr(request_candidates::Column::TotalTokens, Expr::val(input.total_tokens))
        .col_expr(
            request_candidates::Column::CacheCreationInputTokens,
            Expr::val(input.cache_creation_input_tokens),
        )
        .col_expr(request_candidates::Column::CacheReadInputTokens, Expr::val(input.cache_read_input_tokens))
        .col_expr(request_candidates::Column::InputTextTokens, Expr::val(input.input_text_tokens))
        .col_expr(request_candidates::Column::InputAudioTokens, Expr::val(input.input_audio_tokens))
        .col_expr(request_candidates::Column::InputImageTokens, Expr::val(input.input_image_tokens))
        .col_expr(request_candidates::Column::OutputTextTokens, Expr::val(input.output_text_tokens))
        .col_expr(request_candidates::Column::OutputAudioTokens, Expr::val(input.output_audio_tokens))
        .col_expr(request_candidates::Column::OutputImageTokens, Expr::val(input.output_image_tokens))
        .col_expr(request_candidates::Column::ReasoningTokens, Expr::val(input.reasoning_tokens))
        .col_expr(
            request_candidates::Column::CacheCreation5mInputTokens,
            Expr::val(input.cache_creation_5m_input_tokens),
        )
        .col_expr(
            request_candidates::Column::CacheCreation1hInputTokens,
            Expr::val(input.cache_creation_1h_input_tokens),
        )
        .col_expr(request_candidates::Column::UsageSource, Expr::val(input.usage_source))
        .col_expr(request_candidates::Column::UsageSemantic, Expr::val(input.usage_semantic));
    update = apply_candidate_upstream_cost_patch(update, input.upstream_cost);
    update = apply_candidate_billing_values(update, input.billing)?;
    update = apply_json_expr(update, request_candidates::Column::BillingSnapshot, input.billing_snapshot)?;
    update = update
        .col_expr(request_candidates::Column::LatencyMs, Expr::val(input.latency_ms))
        .col_expr(request_candidates::Column::FirstByteTimeMs, Expr::val(input.first_byte_time_ms))
        .col_expr(request_candidates::Column::ErrorType, Expr::val(input.error_type))
        .col_expr(request_candidates::Column::ErrorMessage, Expr::val(input.error_message))
        .col_expr(request_candidates::Column::ErrorCode, Expr::val(input.error_code))
        .col_expr(request_candidates::Column::ErrorParam, Expr::val(input.error_param));
    update = apply_json_expr(update, request_candidates::Column::ProviderRequestHeaders, input.provider_request_headers)?;
    update = apply_json_expr(update, request_candidates::Column::ProviderRequestBody, input.provider_request_body)?;
    update = apply_json_expr(update, request_candidates::Column::ProviderResponseHeaders, input.provider_response_headers)?;
    update = apply_json_expr(update, request_candidates::Column::ProviderResponseBody, input.provider_response_body)?;
    if !was_started {
        update = update.col_expr(request_candidates::Column::StartedAt, Expr::val(Some(now)));
    }
    if input.finished {
        update = update.col_expr(request_candidates::Column::FinishedAt, Expr::val(Some(now)));
    }
    Ok(update)
}

fn apply_candidate_upstream_cost_patch(
    mut update: UpdateMany<request_candidates::Entity>,
    patch: super::RequestUpstreamCostRecordPatch,
) -> UpdateMany<request_candidates::Entity> {
    if !patch.upstream_cost_mode.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamCostMode,
            patch_string_value(patch.upstream_cost_mode, request_upstream_cost::mode_value),
        );
    }
    if !patch.upstream_cost_source.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamCostSource,
            patch_string_value(patch.upstream_cost_source, request_upstream_cost::source_value),
        );
    }
    if !patch.upstream_price_per_request.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamPricePerRequest,
            patch_value(patch.upstream_price_per_request),
        );
    }
    if !patch.upstream_input_price_per_million.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamInputPricePerMillion,
            patch_value(patch.upstream_input_price_per_million),
        );
    }
    if !patch.upstream_output_price_per_million.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamOutputPricePerMillion,
            patch_value(patch.upstream_output_price_per_million),
        );
    }
    if !patch.upstream_cache_creation_price_per_million.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamCacheCreationPricePerMillion,
            patch_value(patch.upstream_cache_creation_price_per_million),
        );
    }
    if !patch.upstream_cache_read_price_per_million.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamCacheReadPricePerMillion,
            patch_value(patch.upstream_cache_read_price_per_million),
        );
    }
    if !patch.upstream_request_cost.is_missing() {
        update = update.col_expr(request_candidates::Column::UpstreamRequestCost, patch_value(patch.upstream_request_cost));
    }
    if !patch.upstream_input_cost.is_missing() {
        update = update.col_expr(request_candidates::Column::UpstreamInputCost, patch_value(patch.upstream_input_cost));
    }
    if !patch.upstream_output_cost.is_missing() {
        update = update.col_expr(request_candidates::Column::UpstreamOutputCost, patch_value(patch.upstream_output_cost));
    }
    if !patch.upstream_cache_creation_cost.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamCacheCreationCost,
            patch_value(patch.upstream_cache_creation_cost),
        );
    }
    if !patch.upstream_cache_read_cost.is_missing() {
        update = update.col_expr(request_candidates::Column::UpstreamCacheReadCost, patch_value(patch.upstream_cache_read_cost));
    }
    if !patch.upstream_total_cost.is_missing() {
        update = update.col_expr(request_candidates::Column::UpstreamTotalCost, patch_value(patch.upstream_total_cost));
    }
    update
}

fn apply_candidate_billing_values(
    mut update: UpdateMany<request_candidates::Entity>,
    billing: RequestBillingRecordValues,
) -> StorageResult<UpdateMany<request_candidates::Entity>> {
    if let Some(service_tier) = billing.service_tier {
        update = update.col_expr(request_candidates::Column::ServiceTier, Expr::val(Some(service_tier)));
    }
    Ok(update
        .col_expr(
            request_candidates::Column::CostCurrency,
            Expr::val(ensure_accounting_cost_currency(billing.cost_currency)?),
        )
        .col_expr(request_candidates::Column::InputCost, Expr::val(billing.input_cost))
        .col_expr(request_candidates::Column::OutputCost, Expr::val(billing.output_cost))
        .col_expr(request_candidates::Column::CacheCreationCost, Expr::val(billing.cache_creation_cost))
        .col_expr(request_candidates::Column::CacheReadCost, Expr::val(billing.cache_read_cost))
        .col_expr(request_candidates::Column::RequestCost, Expr::val(billing.request_cost))
        .col_expr(request_candidates::Column::TokenCost, Expr::val(billing.token_cost))
        .col_expr(request_candidates::Column::BaseCost, Expr::val(billing.base_cost))
        .col_expr(request_candidates::Column::TotalCost, Expr::val(billing.total_cost))
        .col_expr(request_candidates::Column::BillingMultiplier, Expr::val(billing.billing_multiplier))
        .col_expr(request_candidates::Column::InputPricePerMillion, Expr::val(billing.input_price_per_million))
        .col_expr(request_candidates::Column::OutputPricePerMillion, Expr::val(billing.output_price_per_million))
        .col_expr(
            request_candidates::Column::CacheCreationPricePerMillion,
            Expr::val(billing.cache_creation_price_per_million),
        )
        .col_expr(
            request_candidates::Column::CacheReadPricePerMillion,
            Expr::val(billing.cache_read_price_per_million),
        ))
}

fn apply_json_expr(
    update: UpdateMany<request_candidates::Entity>,
    column: request_candidates::Column,
    patch: PatchField<serde_json::Value>,
) -> StorageResult<UpdateMany<request_candidates::Entity>> {
    Ok(match patch {
        PatchField::Value(value) => update.col_expr(column, Expr::val(Some(json::encode_required(&value)?))),
        PatchField::Null => update.col_expr(column, Expr::val(Option::<String>::None)),
        PatchField::Missing => update,
    })
}

fn patch_value<T>(patch: PatchField<T>) -> sea_orm::sea_query::SimpleExpr
where
    T: Into<sea_orm::Value> + sea_orm::sea_query::Nullable,
{
    match patch {
        PatchField::Value(value) => Expr::val(Some(value)),
        PatchField::Null | PatchField::Missing => Expr::val(Option::<T>::None),
    }
}

fn patch_string_value<T>(patch: PatchField<T>, render: fn(&T) -> &'static str) -> sea_orm::sea_query::SimpleExpr {
    match patch {
        PatchField::Value(value) => Expr::val(Some(render(&value).to_owned())),
        PatchField::Null | PatchField::Missing => Expr::val(Option::<String>::None),
    }
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
    Ok(result.rows_affected)
}

pub async fn list_request_candidates(store: &ProviderStore, request: RequestCandidateListRequest) -> StorageResult<Vec<types::provider::RequestCandidate>> {
    let mut query = request_candidates::Entity::find()
        .order_by_asc(request_candidates::Column::CandidateIndex)
        .order_by_asc(request_candidates::Column::RetryIndex);
    if let Some(request_id) = request.request_id {
        query = query.filter(request_candidates::Column::RequestId.eq(request_id));
    }
    let records = query.offset(request.skip).limit(request.limit).all(store.connection()).await?;
    records.into_iter().map(|record| record.response()).collect()
}

use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::Value;
use types::model::PatchField;

use crate::{StorageError, StorageResult, json};

use super::{
    RequestBillingRecordPatch, RequestBillingRecordValues, RequestRecordRecordInput, RequestRecordRecordPatch, record::request_records,
    repository::ProviderStore, request_upstream_cost,
};

pub async fn create_request_record(store: &ProviderStore, input: RequestRecordRecordInput) -> StorageResult<()> {
    let record = request_record_active_model(input)?.insert(store.connection()).await?;
    crate::dashboard::sync_request_metric_buckets(store.connection(), None, &record).await?;
    super::request_record_partition_write::sync_request_record(store, &record.request_id).await?;
    Ok(())
}

pub async fn update_request_record(store: &ProviderStore, input: RequestRecordRecordPatch) -> StorageResult<()> {
    let request_id = input.request_id.clone();
    let Some(record) = request_records::Entity::find_by_id(request_id.clone()).one(store.connection()).await? else {
        return Err(StorageError::NotFound);
    };
    let now = time::OffsetDateTime::now_utc();
    let was_started = record.started_at.is_some();
    let has_failover = record.has_failover;
    let has_retry = record.has_retry;
    let old_record = record.clone();
    let mut active: request_records::ActiveModel = record.into();
    apply_request_record_patch(&mut active, input, now, was_started, has_failover, has_retry)?;
    active.updated_at = Set(now);
    let result = request_records::Entity::update_many()
        .set(active)
        .filter(request_records::Column::RequestId.eq(&request_id))
        .exec(store.connection())
        .await?;
    if result.rows_affected == 0 {
        return Err(StorageError::NotFound);
    }
    let updated = request_records::Entity::find_by_id(request_id)
        .one(store.connection())
        .await?
        .ok_or(StorageError::NotFound)?;
    crate::dashboard::sync_user_usage_buckets(store.connection(), &old_record, &updated).await?;
    crate::dashboard::sync_cost_analysis_buckets(store.connection(), &old_record, &updated).await?;
    crate::dashboard::sync_request_metric_buckets(store.connection(), Some(&old_record), &updated).await?;
    super::request_record_partition_write::sync_request_record(store, &updated.request_id).await?;
    Ok(())
}

fn request_record_active_model(input: RequestRecordRecordInput) -> StorageResult<request_records::ActiveModel> {
    let now = time::OffsetDateTime::now_utc();
    let mut record = request_records::ActiveModel {
        request_id: Set(input.request_id),
        token_id: Set(input.token_id),
        user_id_snapshot: Set(input.user_id_snapshot),
        username_snapshot: Set(input.username_snapshot),
        token_name_snapshot: Set(input.token_name_snapshot),
        token_prefix_snapshot: Set(input.token_prefix_snapshot),
        group_code: Set(input.group_code),
        global_model_id: Set(input.global_model_id),
        model_name_snapshot: Set(input.model_name_snapshot),
        provider_id: Set(input.provider_id),
        provider_name_snapshot: Set(input.provider_name_snapshot),
        endpoint_id: Set(input.endpoint_id),
        key_id: Set(input.key_id),
        provider_key_name_snapshot: Set(input.provider_key_name_snapshot),
        provider_key_preview_snapshot: Set(input.provider_key_preview_snapshot),
        client_api_format: Set(input.client_api_format),
        provider_api_format: Set(input.provider_api_format),
        request_type: Set(input.request_type),
        is_stream: Set(input.is_stream),
        has_failover: Set(input.has_failover),
        has_retry: Set(input.has_retry),
        status: Set(input.status),
        billing_status: Set(input.billing_status),
        client_status_code: Set(None),
        client_error_type: Set(None),
        client_error_message: Set(None),
        termination_origin: Set(None),
        termination_reason: Set(None),
        stream_end_reason: Set(None),
        prompt_tokens: Set(None),
        completion_tokens: Set(None),
        total_tokens: Set(None),
        cache_creation_input_tokens: Set(None),
        cache_read_input_tokens: Set(None),
        input_text_tokens: Set(None),
        input_audio_tokens: Set(None),
        input_image_tokens: Set(None),
        output_text_tokens: Set(None),
        output_audio_tokens: Set(None),
        output_image_tokens: Set(None),
        reasoning_tokens: Set(None),
        cache_creation_5m_input_tokens: Set(None),
        cache_creation_1h_input_tokens: Set(None),
        usage_source: Set(None),
        usage_semantic: Set(None),
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
        input_cost: Set(None),
        output_cost: Set(None),
        cache_creation_cost: Set(None),
        cache_read_cost: Set(None),
        request_cost: Set(None),
        input_price_per_million: Set(None),
        output_price_per_million: Set(None),
        cache_creation_price_per_million: Set(None),
        cache_read_price_per_million: Set(None),
        cost_currency: Set(None),
        token_cost: Set(None),
        base_cost: Set(None),
        total_cost: Set(None),
        billing_multiplier: Set(None),
        billing_snapshot: Set(json::encode_optional(&input.billing_snapshot)?),
        response_headers_time_ms: Set(None),
        first_sse_event_time_ms: Set(None),
        first_token_time_ms: Set(None),
        first_byte_time_ms: Set(None),
        total_latency_ms: Set(None),
        candidate_count: Set(input.candidate_count),
        request_headers: Set(None),
        request_body: Set(None),
        client_response_headers: Set(None),
        client_response_body: Set(None),
        payload_compressed_at: Set(None),
        created_at: Set(now),
        started_at: Set(None),
        finished_at: Set(None),
        updated_at: Set(now),
    };
    request_upstream_cost::apply_request_values(&mut record, input.upstream_cost);
    apply_billing_values(&mut record, input.billing)?;
    Ok(record)
}

fn apply_request_record_patch(
    active: &mut request_records::ActiveModel,
    input: RequestRecordRecordPatch,
    now: time::OffsetDateTime,
    was_started: bool,
    has_failover: bool,
    has_retry: bool,
) -> StorageResult<()> {
    if let Some(provider_id) = input.provider_id {
        active.provider_id = Set(Some(provider_id));
    }
    if let Some(provider_name) = input.provider_name_snapshot {
        active.provider_name_snapshot = Set(Some(provider_name));
    }
    if let Some(endpoint_id) = input.endpoint_id {
        active.endpoint_id = Set(Some(endpoint_id));
    }
    if let Some(key_id) = input.key_id {
        active.key_id = Set(Some(key_id));
    }
    if let Some(key_name) = input.provider_key_name_snapshot {
        active.provider_key_name_snapshot = Set(Some(key_name));
    }
    if let Some(key_preview) = input.provider_key_preview_snapshot {
        active.provider_key_preview_snapshot = Set(Some(key_preview));
    }
    if let Some(provider_api_format) = input.provider_api_format {
        active.provider_api_format = Set(Some(provider_api_format));
    }
    if let Some(is_stream) = input.is_stream {
        active.is_stream = Set(is_stream);
    }
    if let Some(value) = input.has_failover {
        active.has_failover = Set(has_failover || value);
    }
    if let Some(value) = input.has_retry {
        active.has_retry = Set(has_retry || value);
    }
    active.status = Set(input.status);
    active.billing_status = Set(input.billing_status);
    apply_i32_patch(&mut active.client_status_code, input.client_status_code);
    apply_string_patch(&mut active.client_error_type, input.client_error_type);
    apply_string_patch(&mut active.client_error_message, input.client_error_message);
    apply_string_patch(&mut active.termination_origin, input.termination_origin);
    apply_string_patch(&mut active.termination_reason, input.termination_reason);
    apply_string_patch(&mut active.stream_end_reason, input.stream_end_reason);
    apply_i64_patch(&mut active.prompt_tokens, input.prompt_tokens);
    apply_i64_patch(&mut active.completion_tokens, input.completion_tokens);
    apply_i64_patch(&mut active.total_tokens, input.total_tokens);
    apply_i64_patch(&mut active.cache_creation_input_tokens, input.cache_creation_input_tokens);
    apply_i64_patch(&mut active.cache_read_input_tokens, input.cache_read_input_tokens);
    apply_i64_patch(&mut active.input_text_tokens, input.input_text_tokens);
    apply_i64_patch(&mut active.input_audio_tokens, input.input_audio_tokens);
    apply_i64_patch(&mut active.input_image_tokens, input.input_image_tokens);
    apply_i64_patch(&mut active.output_text_tokens, input.output_text_tokens);
    apply_i64_patch(&mut active.output_audio_tokens, input.output_audio_tokens);
    apply_i64_patch(&mut active.output_image_tokens, input.output_image_tokens);
    apply_i64_patch(&mut active.reasoning_tokens, input.reasoning_tokens);
    apply_i64_patch(&mut active.cache_creation_5m_input_tokens, input.cache_creation_5m_input_tokens);
    apply_i64_patch(&mut active.cache_creation_1h_input_tokens, input.cache_creation_1h_input_tokens);
    apply_string_patch(&mut active.usage_source, input.usage_source);
    apply_string_patch(&mut active.usage_semantic, input.usage_semantic);
    request_upstream_cost::apply_request_patch(active, input.upstream_cost);
    apply_billing_patch(active, input.billing)?;
    apply_json_patch(&mut active.billing_snapshot, input.billing_snapshot)?;
    apply_i64_patch(&mut active.response_headers_time_ms, input.response_headers_time_ms);
    apply_i64_patch(&mut active.first_sse_event_time_ms, input.first_sse_event_time_ms);
    apply_i64_patch(&mut active.first_token_time_ms, input.first_token_time_ms);
    apply_i64_patch(&mut active.first_byte_time_ms, input.first_byte_time_ms);
    apply_i64_patch(&mut active.total_latency_ms, input.total_latency_ms);
    apply_legacy_payload_patch(&mut active.client_response_headers, input.client_response_headers);
    apply_legacy_payload_patch(&mut active.client_response_body, input.client_response_body);
    if input.started && !was_started {
        active.started_at = Set(Some(now));
    }
    if input.finished {
        active.finished_at = Set(Some(now));
    } else {
        active.finished_at = Set(None);
    }
    Ok(())
}

pub(super) fn ensure_accounting_cost_currency(value: Option<String>) -> StorageResult<Option<String>> {
    match value {
        Some(currency) if currency == currency::ACCOUNTING_CURRENCY => Ok(Some(currency)),
        Some(_) => Err(StorageError::Conflict(format!("cost currency must be {}", currency::ACCOUNTING_CURRENCY))),
        None => Ok(None),
    }
}

fn apply_billing_values(active: &mut request_records::ActiveModel, billing: RequestBillingRecordValues) -> StorageResult<()> {
    if let Some(service_tier) = billing.service_tier {
        active.service_tier = Set(Some(service_tier));
    }
    active.cost_currency = Set(ensure_accounting_cost_currency(billing.cost_currency)?);
    active.input_cost = Set(billing.input_cost);
    active.output_cost = Set(billing.output_cost);
    active.cache_creation_cost = Set(billing.cache_creation_cost);
    active.cache_read_cost = Set(billing.cache_read_cost);
    active.request_cost = Set(billing.request_cost);
    active.token_cost = Set(billing.token_cost);
    active.base_cost = Set(billing.base_cost);
    active.total_cost = Set(billing.total_cost);
    active.billing_multiplier = Set(billing.billing_multiplier);
    active.input_price_per_million = Set(billing.input_price_per_million);
    active.output_price_per_million = Set(billing.output_price_per_million);
    active.cache_creation_price_per_million = Set(billing.cache_creation_price_per_million);
    active.cache_read_price_per_million = Set(billing.cache_read_price_per_million);
    Ok(())
}

fn apply_billing_patch(active: &mut request_records::ActiveModel, billing: RequestBillingRecordPatch) -> StorageResult<()> {
    apply_string_patch(&mut active.service_tier, billing.service_tier);
    match billing.cost_currency {
        PatchField::Value(value) => active.cost_currency = Set(ensure_accounting_cost_currency(Some(value))?),
        PatchField::Null => active.cost_currency = Set(None),
        PatchField::Missing => {}
    }
    apply_decimal_patch(&mut active.input_cost, billing.input_cost);
    apply_decimal_patch(&mut active.output_cost, billing.output_cost);
    apply_decimal_patch(&mut active.cache_creation_cost, billing.cache_creation_cost);
    apply_decimal_patch(&mut active.cache_read_cost, billing.cache_read_cost);
    apply_decimal_patch(&mut active.request_cost, billing.request_cost);
    apply_decimal_patch(&mut active.token_cost, billing.token_cost);
    apply_decimal_patch(&mut active.base_cost, billing.base_cost);
    apply_decimal_patch(&mut active.total_cost, billing.total_cost);
    apply_decimal_patch(&mut active.billing_multiplier, billing.billing_multiplier);
    apply_decimal_patch(&mut active.input_price_per_million, billing.input_price_per_million);
    apply_decimal_patch(&mut active.output_price_per_million, billing.output_price_per_million);
    apply_decimal_patch(&mut active.cache_creation_price_per_million, billing.cache_creation_price_per_million);
    apply_decimal_patch(&mut active.cache_read_price_per_million, billing.cache_read_price_per_million);
    Ok(())
}

fn apply_i32_patch(active: &mut sea_orm::ActiveValue<Option<i32>>, patch: PatchField<i32>) {
    match patch {
        PatchField::Value(value) => *active = Set(Some(value)),
        PatchField::Null => *active = Set(None),
        PatchField::Missing => {}
    }
}

fn apply_i64_patch(active: &mut sea_orm::ActiveValue<Option<i64>>, patch: PatchField<i64>) {
    match patch {
        PatchField::Value(value) => *active = Set(Some(value)),
        PatchField::Null => *active = Set(None),
        PatchField::Missing => {}
    }
}

fn apply_string_patch(active: &mut sea_orm::ActiveValue<Option<String>>, patch: PatchField<String>) {
    match patch {
        PatchField::Value(value) => *active = Set(Some(value)),
        PatchField::Null => *active = Set(None),
        PatchField::Missing => {}
    }
}

fn apply_decimal_patch(active: &mut sea_orm::ActiveValue<Option<Decimal>>, patch: PatchField<Decimal>) {
    match patch {
        PatchField::Value(value) => *active = Set(Some(value)),
        PatchField::Null => *active = Set(None),
        PatchField::Missing => {}
    }
}

fn apply_json_patch(active: &mut sea_orm::ActiveValue<Option<String>>, patch: PatchField<Value>) -> StorageResult<()> {
    match patch {
        PatchField::Value(value) => *active = Set(Some(json::encode_required(&value)?)),
        PatchField::Null => *active = Set(None),
        PatchField::Missing => {}
    }
    Ok(())
}

fn apply_legacy_payload_patch(active: &mut sea_orm::ActiveValue<Option<String>>, patch: PatchField<Value>) {
    if !patch.is_missing() {
        *active = Set(None);
    }
}

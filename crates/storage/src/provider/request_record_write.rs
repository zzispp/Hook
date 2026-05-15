use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::Value;
use types::model::PatchField;

use crate::{StorageError, StorageResult, json};

use super::{
    RequestBillingRecordPatch, RequestBillingRecordValues, RequestRecordRecordInput, RequestRecordRecordPatch, record::request_records,
    repository::ProviderStore,
};

pub async fn create_request_record(store: &ProviderStore, input: RequestRecordRecordInput) -> StorageResult<()> {
    request_record_active_model(input)?.insert(store.connection()).await?;
    Ok(())
}

pub async fn update_request_record(store: &ProviderStore, input: RequestRecordRecordPatch) -> StorageResult<()> {
    let Some(record) = request_records::Entity::find_by_id(input.request_id.clone()).one(store.connection()).await? else {
        return Err(StorageError::NotFound);
    };
    let now = time::OffsetDateTime::now_utc();
    let was_started = record.started_at.is_some();
    let has_failover = record.has_failover;
    let has_retry = record.has_retry;
    let mut active: request_records::ActiveModel = record.into();
    apply_request_record_patch(&mut active, input, now, was_started, has_failover, has_retry)?;
    active.updated_at = Set(now);
    active.update(store.connection()).await?;
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
        service_tier: Set(None),
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
        first_byte_time_ms: Set(None),
        total_latency_ms: Set(None),
        candidate_count: Set(input.candidate_count),
        request_headers: Set(json::encode_optional(&input.request_headers)?),
        request_body: Set(json::encode_optional(&input.request_body)?),
        client_response_headers: Set(None),
        client_response_body: Set(None),
        created_at: Set(now),
        started_at: Set(None),
        finished_at: Set(None),
        updated_at: Set(now),
    };
    apply_billing_values(&mut record, input.billing);
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
    apply_billing_patch(active, input.billing);
    apply_i64_patch(&mut active.first_byte_time_ms, input.first_byte_time_ms);
    apply_i64_patch(&mut active.total_latency_ms, input.total_latency_ms);
    apply_json_patch(&mut active.client_response_headers, input.client_response_headers)?;
    apply_json_patch(&mut active.client_response_body, input.client_response_body)?;
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

fn apply_billing_values(active: &mut request_records::ActiveModel, billing: RequestBillingRecordValues) {
    if let Some(service_tier) = billing.service_tier {
        active.service_tier = Set(Some(service_tier));
    }
    active.cost_currency = Set(billing.cost_currency);
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
}

fn apply_billing_patch(active: &mut request_records::ActiveModel, billing: RequestBillingRecordPatch) {
    apply_string_patch(&mut active.service_tier, billing.service_tier);
    apply_string_patch(&mut active.cost_currency, billing.cost_currency);
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

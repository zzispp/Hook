use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::Value;
use types::model::PatchField;

use crate::{StorageError, StorageResult, json};

use super::{RequestRecordRecordInput, RequestRecordRecordPatch, record::request_records, repository::ProviderStore};

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
    Ok(request_records::ActiveModel {
        request_id: Set(input.request_id),
        token_id: Set(input.token_id),
        group_code: Set(input.group_code),
        global_model_id: Set(input.global_model_id),
        provider_id: Set(input.provider_id),
        endpoint_id: Set(input.endpoint_id),
        key_id: Set(input.key_id),
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
    })
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
    if let Some(endpoint_id) = input.endpoint_id {
        active.endpoint_id = Set(Some(endpoint_id));
    }
    if let Some(key_id) = input.key_id {
        active.key_id = Set(Some(key_id));
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
    apply_string_patch(&mut active.cost_currency, input.cost_currency);
    apply_decimal_patch(&mut active.token_cost, input.token_cost);
    apply_decimal_patch(&mut active.base_cost, input.base_cost);
    apply_decimal_patch(&mut active.total_cost, input.total_cost);
    apply_decimal_patch(&mut active.billing_multiplier, input.billing_multiplier);
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

use std::collections::HashSet;

use sea_orm::Set;
use serde::Serialize;
use types::provider::{ProviderListRequest, ProviderModelBinding};

use crate::{StorageResult, json};

use super::{
    ProviderApiKeyRecordPatch, ProviderEndpointRecord, ProviderEndpointRecordPatch, ProviderModelRecord, ProviderModelRecordPatch, ProviderRecord,
    ProviderRecordInput, ProviderRecordPatch, record::provider_api_keys::ActiveModel as ProviderApiKeyActiveModel,
    record::provider_endpoints::ActiveModel as ProviderEndpointActiveModel, record::providers::ActiveModel as ProviderActiveModel,
};

pub fn provider_active_model(id: String, input: ProviderRecordInput) -> ProviderActiveModel {
    let now = time::OffsetDateTime::now_utc();
    ProviderActiveModel {
        id: Set(id),
        name: Set(input.name),
        provider_type: Set(input.provider_type),
        max_retries: Set(input.max_retries),
        request_timeout_seconds: Set(input.request_timeout_seconds),
        stream_first_byte_timeout_seconds: Set(input.stream_first_byte_timeout_seconds),
        priority: Set(input.priority),
        keep_priority_on_conversion: Set(input.keep_priority_on_conversion),
        enable_format_conversion: Set(input.enable_format_conversion),
        is_active: Set(input.is_active),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

pub fn apply_provider_patch(active: &mut ProviderActiveModel, input: ProviderRecordPatch) {
    if let Some(name) = input.name {
        active.name = Set(name);
    }
    if let Some(provider_type) = input.provider_type {
        active.provider_type = Set(provider_type);
    }
    apply_i32_patch(&mut active.max_retries, input.max_retries);
    apply_f64_patch(&mut active.request_timeout_seconds, input.request_timeout_seconds);
    apply_f64_patch(&mut active.stream_first_byte_timeout_seconds, input.stream_first_byte_timeout_seconds);
    if let Some(priority) = input.priority {
        active.priority = Set(priority);
    }
    if let Some(value) = input.keep_priority_on_conversion {
        active.keep_priority_on_conversion = Set(value);
    }
    if let Some(value) = input.enable_format_conversion {
        active.enable_format_conversion = Set(value);
    }
    if let Some(value) = input.is_active {
        active.is_active = Set(value);
    }
}

pub fn apply_endpoint_patch(active: &mut ProviderEndpointActiveModel, input: ProviderEndpointRecordPatch) -> StorageResult<()> {
    if let Some(api_format) = input.api_format {
        active.api_format = Set(api_format);
    }
    if let Some(base_url) = input.base_url {
        active.base_url = Set(base_url);
    }
    apply_string_patch(&mut active.custom_path, input.custom_path);
    apply_i32_patch(&mut active.max_retries, input.max_retries);
    if let Some(value) = input.is_active {
        active.is_active = Set(value);
    }
    apply_json_patch(&mut active.format_acceptance_config, input.format_acceptance_config)?;
    apply_json_patch(&mut active.header_rules, input.header_rules)?;
    apply_json_patch(&mut active.body_rules, input.body_rules)?;
    Ok(())
}

pub struct ProviderFilterIds {
    pub api_format: Option<HashSet<String>>,
    pub model: Option<HashSet<String>>,
}

pub fn filter_provider_records(records: Vec<ProviderRecord>, request: &ProviderListRequest, ids: ProviderFilterIds) -> Vec<ProviderRecord> {
    let search = request.search.as_ref().map(|value| value.to_ascii_lowercase());
    records
        .into_iter()
        .filter(|record| provider_is_allowed(record, request, search.as_deref(), &ids))
        .collect()
}

pub fn provider_model_response(record: ProviderModelRecord) -> StorageResult<ProviderModelBinding> {
    Ok(ProviderModelBinding {
        id: record.id,
        provider_id: record.provider_id,
        global_model_id: record.global_model_id,
        provider_model_name: record.provider_model_name,
        provider_model_mapping: json::decode_optional(record.provider_model_mappings)?,
        is_active: record.is_active,
        price_per_request: record.price_per_request,
        tiered_pricing: json::decode_optional(record.tiered_pricing)?,
        config: json::decode_optional(record.config)?,
        created_at: record.created_at.to_string(),
        updated_at: record.updated_at.to_string(),
    })
}

pub fn apply_provider_model_patch(active: &mut super::record::provider_models::ActiveModel, input: ProviderModelRecordPatch) -> StorageResult<()> {
    if let Some(name) = input.provider_model_name {
        active.provider_model_name = Set(name);
    }
    if let Some(value) = input.is_active {
        active.is_active = Set(value);
    }
    apply_encoded_patch(&mut active.provider_model_mappings, input.provider_model_mapping)?;
    apply_json_patch(&mut active.config, input.config)?;
    Ok(())
}

pub fn apply_provider_api_key_patch(active: &mut ProviderApiKeyActiveModel, input: ProviderApiKeyRecordPatch) -> StorageResult<()> {
    if let Some(name) = input.name {
        active.name = Set(name);
    }
    if let Some(api_formats) = input.api_formats {
        active.api_formats = Set(json::encode_required(&api_formats)?);
    }
    if let Some(allowed_model_ids) = input.allowed_model_ids {
        active.allowed_model_ids = Set(json::encode_required(&allowed_model_ids)?);
    }
    if let Some(encrypted_api_key) = input.encrypted_api_key {
        active.encrypted_api_key = Set(encrypted_api_key);
    }
    apply_string_patch(&mut active.note, input.note);
    if let Some(value) = input.internal_priority {
        active.internal_priority = Set(value);
    }
    apply_i32_patch(&mut active.rpm_limit, input.rpm_limit);
    if let Some(value) = input.cache_ttl_minutes {
        active.cache_ttl_minutes = Set(value);
    }
    if let Some(value) = input.max_probe_interval_minutes {
        active.max_probe_interval_minutes = Set(value);
    }
    if let Some(value) = input.time_range_enabled {
        active.time_range_enabled = Set(value);
    }
    apply_string_patch(&mut active.time_range_start, input.time_range_start);
    apply_string_patch(&mut active.time_range_end, input.time_range_end);
    if let Some(value) = input.is_active {
        active.is_active = Set(value);
    }
    Ok(())
}

pub fn endpoint_belongs_to_provider(record: &ProviderEndpointRecord, provider_id: &str) -> bool {
    record.provider_id == provider_id
}

fn provider_matches(record: &ProviderRecord, query: &str) -> bool {
    record.name.to_ascii_lowercase().contains(query) || record.provider_type.to_ascii_lowercase().contains(query)
}

fn provider_is_allowed(record: &ProviderRecord, request: &ProviderListRequest, search: Option<&str>, ids: &ProviderFilterIds) -> bool {
    request.is_active.is_none_or(|value| record.is_active == value)
        && search.is_none_or(|query| provider_matches(record, query))
        && provider_id_matches(&record.id, &ids.api_format)
        && provider_id_matches(&record.id, &ids.model)
}

fn provider_id_matches(provider_id: &str, allowed_ids: &Option<HashSet<String>>) -> bool {
    allowed_ids.as_ref().is_none_or(|ids| ids.contains(provider_id))
}

fn apply_i32_patch(active: &mut sea_orm::ActiveValue<Option<i32>>, patch: types::model::PatchField<i32>) {
    match patch {
        types::model::PatchField::Value(value) => *active = Set(Some(value)),
        types::model::PatchField::Null => *active = Set(None),
        types::model::PatchField::Missing => {}
    }
}

fn apply_string_patch(active: &mut sea_orm::ActiveValue<Option<String>>, patch: types::model::PatchField<String>) {
    match patch {
        types::model::PatchField::Value(value) => *active = Set(Some(value)),
        types::model::PatchField::Null => *active = Set(None),
        types::model::PatchField::Missing => {}
    }
}

fn apply_json_patch(active: &mut sea_orm::ActiveValue<Option<String>>, patch: types::model::PatchField<serde_json::Value>) -> StorageResult<()> {
    match patch {
        types::model::PatchField::Value(value) => *active = Set(Some(json::encode_required(&value)?)),
        types::model::PatchField::Null => *active = Set(None),
        types::model::PatchField::Missing => {}
    }
    Ok(())
}

fn apply_encoded_patch<T>(active: &mut sea_orm::ActiveValue<Option<String>>, patch: types::model::PatchField<T>) -> StorageResult<()>
where
    T: Serialize,
{
    match patch {
        types::model::PatchField::Value(value) => *active = Set(Some(json::encode_required(&value)?)),
        types::model::PatchField::Null => *active = Set(None),
        types::model::PatchField::Missing => {}
    }
    Ok(())
}

fn apply_f64_patch(active: &mut sea_orm::ActiveValue<Option<f64>>, patch: types::model::PatchField<f64>) {
    match patch {
        types::model::PatchField::Value(value) => *active = Set(Some(value)),
        types::model::PatchField::Null => *active = Set(None),
        types::model::PatchField::Missing => {}
    }
}

use std::collections::HashSet;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use types::provider::{ProviderListRequest, ProviderModelBinding};

use crate::{StorageResult, json};

use super::{
    ProviderEndpointRecord, ProviderEndpointRecordPatch, ProviderModelRecord, ProviderRecord, ProviderRecordInput, ProviderRecordPatch,
    record::providers::ActiveModel as ProviderActiveModel,
    record::{provider_api_keys, provider_endpoints::ActiveModel as ProviderEndpointActiveModel},
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
        price_per_request: record.price_per_request,
        tiered_pricing: json::decode_optional(record.tiered_pricing)?,
        config: json::decode_optional(record.config)?,
        created_at: record.created_at.to_string(),
        updated_at: record.updated_at.to_string(),
    })
}

pub fn endpoint_belongs_to_provider(record: &ProviderEndpointRecord, provider_id: &str) -> bool {
    record.provider_id == provider_id
}

pub async fn remove_api_format_from_keys(provider_id: &str, api_format: &str, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    let records = provider_api_keys::Entity::find()
        .filter(provider_api_keys::Column::ProviderId.eq(provider_id))
        .all(tx)
        .await?;
    for record in records {
        let Some(api_formats) = api_formats_without(record.api_formats.clone(), api_format)? else {
            continue;
        };
        let mut active: provider_api_keys::ActiveModel = record.into();
        active.api_formats = Set(json::encode_optional(&Some(api_formats))?);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(tx).await?;
    }
    Ok(())
}

fn provider_matches(record: &ProviderRecord, query: &str) -> bool {
    record.name.to_ascii_lowercase().contains(query) || record.provider_type.to_ascii_lowercase().contains(query)
}

fn api_formats_without(value: Option<String>, api_format: &str) -> StorageResult<Option<Vec<String>>> {
    let Some(original) = json::decode_optional::<Vec<String>>(value)? else {
        return Ok(None);
    };
    let original_len = original.len();
    let filtered: Vec<String> = original.into_iter().filter(|value| value != api_format).collect();
    Ok((filtered.len() != original_len).then_some(filtered))
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

fn apply_f64_patch(active: &mut sea_orm::ActiveValue<Option<f64>>, patch: types::model::PatchField<f64>) {
    match patch {
        types::model::PatchField::Value(value) => *active = Set(Some(value)),
        types::model::PatchField::Null => *active = Set(None),
        types::model::PatchField::Missing => {}
    }
}

#[cfg(test)]
mod tests {
    use super::api_formats_without;

    #[test]
    fn api_formats_without_keeps_unrestricted_keys() {
        let result = api_formats_without(None, "openai_chat").unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn api_formats_without_removes_existing_format() {
        let input = Some(r#"["openai_chat","openai_cli"]"#.to_owned());

        let result = api_formats_without(input, "openai_chat").unwrap();

        assert_eq!(result, Some(vec!["openai_cli".to_owned()]));
    }

    #[test]
    fn api_formats_without_returns_none_when_format_is_absent() {
        let input = Some(r#"["openai_chat"]"#.to_owned());

        let result = api_formats_without(input, "openai_cli").unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn api_formats_without_returns_empty_list_when_last_format_is_removed() {
        let input = Some(r#"["openai_chat"]"#.to_owned());

        let result = api_formats_without(input, "openai_chat").unwrap();

        assert_eq!(result, Some(Vec::<String>::new()));
    }
}

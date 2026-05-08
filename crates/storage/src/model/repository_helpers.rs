use sea_orm::Set;
use types::model::{ModelCapabilities, ModelCatalogProviderDetail, PatchField};

use crate::{StorageResult, json};

use super::{GlobalModelRecord, GlobalModelRecordPatch, ModelRecord, record::global_models::ActiveModel as GlobalModelActiveModel};

pub(super) fn apply_global_model_patch(active: &mut GlobalModelActiveModel, input: GlobalModelRecordPatch) -> StorageResult<()> {
    if let Some(display_name) = input.display_name {
        active.display_name = Set(display_name);
    }
    if let Some(is_active) = input.is_active {
        active.is_active = Set(is_active);
    }
    apply_patch_fields(
        active,
        input.default_price_per_request,
        input.default_tiered_pricing,
        input.supported_capabilities,
        input.config,
    )
}

pub(super) fn capabilities(record: &GlobalModelRecord, providers: &[ModelCatalogProviderDetail]) -> StorageResult<ModelCapabilities> {
    let config = record.config()?;
    Ok(ModelCapabilities {
        supports_vision: config_bool(config.as_ref(), "vision") || providers.iter().any(|provider| provider.supports_vision == Some(true)),
        supports_function_calling: config_bool(config.as_ref(), "function_calling")
            || providers.iter().any(|provider| provider.supports_function_calling == Some(true)),
        supports_streaming: config_bool_default(config.as_ref(), "streaming", true)
            || providers.iter().any(|provider| provider.supports_streaming == Some(true)),
    })
}

pub(super) fn description(config: Option<&serde_json::Value>) -> Option<String> {
    config
        .and_then(|value| value.get("description"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
}

pub(super) fn record_matches(record: &GlobalModelRecord, active: Option<bool>, search: Option<&str>) -> bool {
    let active_matches = active.is_none_or(|expected| record.is_active == expected);
    active_matches && search.is_none_or(|query| model_matches_search(record, query))
}

pub(super) fn unique_provider_count(mut records: Vec<ModelRecord>) -> u64 {
    records.sort_by(|left, right| left.provider_id.cmp(&right.provider_id));
    records.dedup_by(|left, right| left.provider_id == right.provider_id);
    records.len() as u64
}

fn apply_patch_fields(
    active: &mut GlobalModelActiveModel,
    price_per_request: PatchField<rust_decimal::Decimal>,
    tiered_pricing: Option<types::model::TieredPricingConfig>,
    supported_capabilities: PatchField<Vec<String>>,
    config: PatchField<serde_json::Value>,
) -> StorageResult<()> {
    apply_price_patch(active, price_per_request);
    if let Some(pricing) = tiered_pricing {
        active.default_tiered_pricing = Set(json::encode_required(&pricing)?);
    }
    apply_json_patch(&mut active.supported_capabilities, supported_capabilities)?;
    apply_json_patch(&mut active.config, config)
}

fn apply_price_patch(active: &mut GlobalModelActiveModel, patch: PatchField<rust_decimal::Decimal>) {
    match patch {
        PatchField::Value(value) => active.default_price_per_request = Set(Some(value)),
        PatchField::Null => active.default_price_per_request = Set(None),
        PatchField::Missing => {}
    }
}

fn apply_json_patch<T>(field: &mut sea_orm::ActiveValue<Option<String>>, patch: PatchField<T>) -> StorageResult<()>
where
    T: serde::Serialize,
{
    match patch {
        PatchField::Value(value) => *field = Set(Some(json::encode_required(&value)?)),
        PatchField::Null => *field = Set(None),
        PatchField::Missing => {}
    }
    Ok(())
}

fn config_bool(config: Option<&serde_json::Value>, key: &str) -> bool {
    config_bool_default(config, key, false)
}

fn config_bool_default(config: Option<&serde_json::Value>, key: &str, default: bool) -> bool {
    config.and_then(|value| value.get(key)).and_then(serde_json::Value::as_bool).unwrap_or(default)
}

fn model_matches_search(record: &GlobalModelRecord, query: &str) -> bool {
    record.name.to_ascii_lowercase().contains(query) || record.display_name.to_ascii_lowercase().contains(query)
}

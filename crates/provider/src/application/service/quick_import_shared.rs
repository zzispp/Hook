use std::collections::BTreeMap;

use rust_decimal::Decimal;
use types::{
    model::GlobalModelResponse,
    provider::{ProviderCreate, ProviderQuickImportProviderConfig, ProviderQuickImportSourceConfig, ProviderQuickImportSourceKind},
};

use crate::application::{ProviderError, ProviderResult};

const PROVIDER_TYPE_CUSTOM: &str = "custom";
const DEFAULT_MAX_RETRIES: i32 = 2;
const DEFAULT_REQUEST_TIMEOUT_SECONDS: f64 = 300.0;
const DEFAULT_STREAM_FIRST_BYTE_TIMEOUT_SECONDS: f64 = 60.0;
const DEFAULT_STREAM_IDLE_TIMEOUT_SECONDS: f64 = 300.0;
const DEFAULT_PRIORITY: i32 = 100;

pub fn provider_create(name: &str, config: &ProviderQuickImportProviderConfig) -> ProviderCreate {
    ProviderCreate {
        name: name.trim().to_owned(),
        provider_type: PROVIDER_TYPE_CUSTOM.into(),
        max_retries: Some(config.max_retries.unwrap_or(DEFAULT_MAX_RETRIES)),
        request_timeout_seconds: Some(config.request_timeout_seconds.unwrap_or(DEFAULT_REQUEST_TIMEOUT_SECONDS)),
        stream_first_byte_timeout_seconds: Some(config.stream_first_byte_timeout_seconds.unwrap_or(DEFAULT_STREAM_FIRST_BYTE_TIMEOUT_SECONDS)),
        stream_idle_timeout_seconds: Some(config.stream_idle_timeout_seconds.unwrap_or(DEFAULT_STREAM_IDLE_TIMEOUT_SECONDS)),
        priority: Some(config.priority.unwrap_or(DEFAULT_PRIORITY)),
        keep_priority_on_conversion: Some(config.keep_priority_on_conversion.unwrap_or(false)),
        enable_format_conversion: Some(config.enable_format_conversion.unwrap_or(true)),
        is_active: Some(config.is_active.unwrap_or(true)),
    }
}

pub fn validate_common(
    source_kind: ProviderQuickImportSourceKind,
    source: &ProviderQuickImportSourceConfig,
    provider_name: &str,
    recharge_multiplier: Decimal,
) -> ProviderResult<()> {
    if source_kind != source.kind() {
        return Err(ProviderError::InvalidInput("source_kind does not match source.kind".into()));
    }
    if provider_name.trim().is_empty() {
        return Err(ProviderError::InvalidInput("provider_name cannot be blank".into()));
    }
    if recharge_multiplier <= Decimal::ZERO {
        return Err(ProviderError::InvalidInput("recharge_multiplier must be greater than 0".into()));
    }
    validate_source(source)
}

pub fn source_base_url(source: &ProviderQuickImportSourceConfig) -> String {
    let ProviderQuickImportSourceConfig::Newapi(config) = source;
    config.base_url.trim().trim_end_matches('/').to_owned()
}

pub fn globals_by_name(models: &[GlobalModelResponse]) -> BTreeMap<String, &GlobalModelResponse> {
    models.iter().map(|model| (model.name.clone(), model)).collect()
}

pub fn globals_by_id(models: &[GlobalModelResponse]) -> BTreeMap<String, &GlobalModelResponse> {
    models.iter().map(|model| (model.id.clone(), model)).collect()
}

pub fn global_model<'a>(models: &'a BTreeMap<String, &'a GlobalModelResponse>, id: &str) -> ProviderResult<&'a GlobalModelResponse> {
    models
        .get(id)
        .copied()
        .ok_or_else(|| ProviderError::InvalidInput(format!("global model does not exist or is inactive: {id}")))
}

fn validate_source(source: &ProviderQuickImportSourceConfig) -> ProviderResult<()> {
    let ProviderQuickImportSourceConfig::Newapi(config) = source;
    if config.base_url.trim().is_empty() || config.system_access_token.trim().is_empty() || config.user_id.trim().is_empty() {
        return Err(ProviderError::InvalidInput("newapi source fields cannot be blank".into()));
    }
    Ok(())
}

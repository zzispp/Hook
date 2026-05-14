use types::{
    model::PatchField,
    provider::{
        ProviderApiKeyCreate, ProviderApiKeyUpdate, ProviderCreate, ProviderEndpointCreate, ProviderEndpointUpdate, ProviderListRequest,
        ProviderModelBindingCreate, ProviderModelBindingUpdate, ProviderModelMapping, ProviderUpdate,
    },
};

use super::{ProviderError, ProviderResult};

const MAX_LIST_LIMIT: u64 = 1000;
const MAX_NAME_LENGTH: usize = 100;
const MAX_TYPE_LENGTH: usize = 50;
const MAX_API_FORMAT_LENGTH: usize = 50;
const MAX_URL_LENGTH: usize = 500;
const MAX_MODEL_NAME_LENGTH: usize = 200;
const REASONING_EFFORTS: [&str; 4] = ["minimal", "low", "medium", "high"];
const PROVIDER_TYPES: [&str; 1] = ["custom"];

pub fn sanitize_create(input: ProviderCreate) -> ProviderCreate {
    ProviderCreate {
        name: input.name.trim().to_owned(),
        provider_type: input.provider_type.trim().to_owned(),
        ..input
    }
}

pub fn sanitize_list_request(input: ProviderListRequest) -> ProviderListRequest {
    ProviderListRequest {
        search: input.search.and_then(trim_optional),
        api_format: input.api_format.and_then(trim_optional).map(|value| value.to_ascii_lowercase()),
        model_id: input.model_id.and_then(trim_optional),
        ..input
    }
}

pub fn sanitize_update(input: ProviderUpdate) -> ProviderUpdate {
    ProviderUpdate {
        name: input.name.map(|value| value.trim().to_owned()),
        provider_type: input.provider_type.map(|value| value.trim().to_owned()),
        ..input
    }
}

pub fn sanitize_endpoint(input: ProviderEndpointCreate) -> ProviderEndpointCreate {
    ProviderEndpointCreate {
        api_format: input.api_format.trim().to_ascii_lowercase(),
        base_url: input.base_url.trim().to_owned(),
        custom_path: input.custom_path.and_then(trim_optional),
        ..input
    }
}

pub fn sanitize_endpoint_update(input: ProviderEndpointUpdate) -> ProviderEndpointUpdate {
    ProviderEndpointUpdate {
        api_format: input.api_format.map(|value| value.trim().to_ascii_lowercase()),
        base_url: input.base_url.map(|value| value.trim().to_owned()),
        custom_path: trim_patch(input.custom_path),
        ..input
    }
}

pub fn sanitize_api_key(input: ProviderApiKeyCreate) -> ProviderApiKeyCreate {
    ProviderApiKeyCreate {
        name: input.name.trim().to_owned(),
        api_key: input.api_key.trim().to_owned(),
        note: input.note.and_then(trim_optional),
        ..input
    }
}

pub fn sanitize_api_key_update(input: ProviderApiKeyUpdate) -> ProviderApiKeyUpdate {
    ProviderApiKeyUpdate {
        name: input.name.map(|value| value.trim().to_owned()),
        api_key: input.api_key.map(|value| value.trim().to_owned()),
        note: trim_patch(input.note),
        time_range_start: trim_patch(input.time_range_start),
        time_range_end: trim_patch(input.time_range_end),
        ..input
    }
}

pub fn sanitize_model_binding(input: ProviderModelBindingCreate) -> ProviderModelBindingCreate {
    ProviderModelBindingCreate {
        global_model_id: input.global_model_id.trim().to_owned(),
        provider_model_name: input.provider_model_name.trim().to_owned(),
        provider_model_mapping: sanitize_provider_model_mapping(input.provider_model_mapping),
        ..input
    }
}

pub fn sanitize_model_binding_update(input: ProviderModelBindingUpdate) -> ProviderModelBindingUpdate {
    ProviderModelBindingUpdate {
        provider_model_name: input.provider_model_name.map(|value| value.trim().to_owned()),
        provider_model_mapping: sanitize_provider_model_mapping_patch(input.provider_model_mapping),
        ..input
    }
}

pub fn validate_create(input: &ProviderCreate) -> ProviderResult<()> {
    validate_text("name", &input.name, MAX_NAME_LENGTH)?;
    validate_provider_type(&input.provider_type)
}

pub fn validate_update(input: &ProviderUpdate) -> ProviderResult<()> {
    if input.is_empty() {
        return Err(ProviderError::InvalidInput("update payload is empty".into()));
    }
    if let Some(name) = input.name.as_deref() {
        validate_text("name", name, MAX_NAME_LENGTH)?;
    }
    if let Some(provider_type) = input.provider_type.as_deref() {
        validate_provider_type(provider_type)?;
    }
    Ok(())
}

pub fn validate_list_request(request: &ProviderListRequest) -> ProviderResult<()> {
    if request.limit == 0 || request.limit > MAX_LIST_LIMIT {
        return Err(ProviderError::InvalidInput(format!("limit must be between 1 and {MAX_LIST_LIMIT}")));
    }
    Ok(())
}

pub fn validate_endpoint(input: &ProviderEndpointCreate) -> ProviderResult<()> {
    validate_text("api_format", &input.api_format, MAX_API_FORMAT_LENGTH)?;
    validate_text("base_url", &input.base_url, MAX_URL_LENGTH)
}

pub fn validate_endpoint_update(input: &ProviderEndpointUpdate) -> ProviderResult<()> {
    if endpoint_update_is_empty(input) {
        return Err(ProviderError::InvalidInput("endpoint update payload is empty".into()));
    }
    if let Some(api_format) = input.api_format.as_deref() {
        validate_text("api_format", api_format, MAX_API_FORMAT_LENGTH)?;
    }
    if let Some(base_url) = input.base_url.as_deref() {
        validate_text("base_url", base_url, MAX_URL_LENGTH)?;
    }
    Ok(())
}

pub fn validate_api_key(input: &ProviderApiKeyCreate) -> ProviderResult<()> {
    validate_text("name", &input.name, MAX_NAME_LENGTH)?;
    if input.api_key.is_empty() {
        return Err(ProviderError::InvalidInput("api_key cannot be blank".into()));
    }
    Ok(())
}

pub fn validate_api_key_update(input: &ProviderApiKeyUpdate) -> ProviderResult<()> {
    if api_key_update_is_empty(input) {
        return Err(ProviderError::InvalidInput("api key update payload is empty".into()));
    }
    if let Some(name) = input.name.as_deref() {
        validate_text("name", name, MAX_NAME_LENGTH)?;
    }
    if input.api_key.as_deref().is_some_and(str::is_empty) {
        return Err(ProviderError::InvalidInput("api_key cannot be blank".into()));
    }
    Ok(())
}

pub fn validate_model_binding(input: &ProviderModelBindingCreate) -> ProviderResult<()> {
    validate_text("global_model_id", &input.global_model_id, MAX_NAME_LENGTH)?;
    validate_text("provider_model_name", &input.provider_model_name, MAX_MODEL_NAME_LENGTH)?;
    validate_provider_model_mapping(input.provider_model_mapping.as_ref())
}

pub fn validate_model_binding_update(input: &ProviderModelBindingUpdate) -> ProviderResult<()> {
    if model_binding_update_is_empty(input) {
        return Err(ProviderError::InvalidInput("model binding update payload is empty".into()));
    }
    if let Some(name) = input.provider_model_name.as_deref() {
        validate_text("provider_model_name", name, MAX_MODEL_NAME_LENGTH)?;
    }
    if let PatchField::Value(mapping) = &input.provider_model_mapping {
        validate_provider_model_mapping(Some(mapping))?;
    }
    Ok(())
}

fn validate_text(field: &str, value: &str, max_length: usize) -> ProviderResult<()> {
    if value.is_empty() || value.len() > max_length {
        return Err(ProviderError::InvalidInput(format!("{field} length must be between 1 and {max_length}")));
    }
    Ok(())
}

fn validate_provider_type(value: &str) -> ProviderResult<()> {
    validate_text("provider_type", value, MAX_TYPE_LENGTH)?;
    if PROVIDER_TYPES.contains(&value) {
        return Ok(());
    }
    Err(ProviderError::InvalidInput(format!(
        "provider_type must be one of {}",
        PROVIDER_TYPES.join(", ")
    )))
}

fn trim_optional(value: String) -> Option<String> {
    let value = value.trim().to_owned();
    if value.is_empty() { None } else { Some(value) }
}

fn trim_patch(value: PatchField<String>) -> PatchField<String> {
    match value {
        PatchField::Value(value) => trim_optional(value).map(PatchField::Value).unwrap_or(PatchField::Null),
        other => other,
    }
}

fn sanitize_provider_model_mapping_patch(mapping: PatchField<ProviderModelMapping>) -> PatchField<ProviderModelMapping> {
    match mapping {
        PatchField::Value(value) => sanitize_provider_model_mapping(Some(value)).map(PatchField::Value).unwrap_or(PatchField::Null),
        other => other,
    }
}

fn sanitize_provider_model_mapping(mapping: Option<ProviderModelMapping>) -> Option<ProviderModelMapping> {
    let mapping = mapping?;
    let name = mapping.name.trim().to_owned();
    let reasoning_effort = mapping.reasoning_effort.and_then(trim_optional).map(|value| value.to_ascii_lowercase());
    if name.is_empty() {
        return None;
    }
    Some(ProviderModelMapping { name, reasoning_effort })
}

fn validate_provider_model_mapping(mapping: Option<&ProviderModelMapping>) -> ProviderResult<()> {
    let Some(mapping) = mapping else {
        return Ok(());
    };
    validate_text("provider_model_mapping.name", &mapping.name, MAX_MODEL_NAME_LENGTH)?;
    if let Some(reasoning_effort) = mapping.reasoning_effort.as_deref() {
        validate_reasoning_effort(reasoning_effort)?;
    }
    Ok(())
}

fn validate_reasoning_effort(value: &str) -> ProviderResult<()> {
    if REASONING_EFFORTS.contains(&value) {
        return Ok(());
    }
    Err(ProviderError::InvalidInput(format!(
        "provider_model_mapping.reasoning_effort must be one of {}",
        REASONING_EFFORTS.join(", ")
    )))
}

fn endpoint_update_is_empty(input: &ProviderEndpointUpdate) -> bool {
    input.api_format.is_none()
        && input.base_url.is_none()
        && input.custom_path.is_missing()
        && input.max_retries.is_missing()
        && input.is_active.is_none()
        && input.format_acceptance_config.is_missing()
        && input.header_rules.is_missing()
        && input.body_rules.is_missing()
}

fn model_binding_update_is_empty(input: &ProviderModelBindingUpdate) -> bool {
    input.provider_model_name.is_none() && input.is_active.is_none() && input.provider_model_mapping.is_missing() && input.config.is_missing()
}

fn api_key_update_is_empty(input: &ProviderApiKeyUpdate) -> bool {
    input.name.is_none()
        && input.api_key.is_none()
        && input.note.is_missing()
        && input.internal_priority.is_none()
        && input.rpm_limit.is_missing()
        && input.cache_ttl_minutes.is_none()
        && input.max_probe_interval_minutes.is_none()
        && input.time_range_enabled.is_none()
        && input.time_range_start.is_missing()
        && input.time_range_end.is_missing()
        && input.is_active.is_none()
}

use types::{
    model::PatchField,
    provider::{
        ProviderApiKeyCreate, ProviderCreate, ProviderEndpointCreate, ProviderEndpointUpdate, ProviderListRequest, ProviderModelBindingCreate, ProviderUpdate,
    },
};

use super::{ProviderError, ProviderResult};

const MAX_LIST_LIMIT: u64 = 1000;
const MAX_NAME_LENGTH: usize = 100;
const MAX_TYPE_LENGTH: usize = 50;
const MAX_API_FORMAT_LENGTH: usize = 50;
const MAX_URL_LENGTH: usize = 500;
const MAX_MODEL_NAME_LENGTH: usize = 200;
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
        api_formats: input.api_formats.map(normalize_api_formats),
        ..input
    }
}

pub fn sanitize_model_binding(input: ProviderModelBindingCreate) -> ProviderModelBindingCreate {
    ProviderModelBindingCreate {
        global_model_id: input.global_model_id.trim().to_owned(),
        provider_model_name: input.provider_model_name.trim().to_owned(),
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

pub fn validate_model_binding(input: &ProviderModelBindingCreate) -> ProviderResult<()> {
    validate_text("global_model_id", &input.global_model_id, MAX_NAME_LENGTH)?;
    validate_text("provider_model_name", &input.provider_model_name, MAX_MODEL_NAME_LENGTH)
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

fn normalize_api_formats(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect()
}

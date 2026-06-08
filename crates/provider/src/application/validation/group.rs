use types::{
    model::PatchField,
    provider::{ProviderGroupCreate, ProviderGroupListRequest, ProviderGroupUpdate, ProviderKeyGroupCreate, ProviderKeyGroupUpdate},
};

use super::{MAX_DESCRIPTION_LENGTH, MAX_LIST_LIMIT, MAX_NAME_LENGTH, trim_optional, trim_patch, validate_text};
use crate::application::{ProviderError, ProviderResult};

pub fn sanitize_provider_group(input: ProviderGroupCreate) -> ProviderGroupCreate {
    ProviderGroupCreate {
        name: input.name.trim().to_owned(),
        description: input.description.and_then(trim_optional),
        provider_ids: normalize_ids(input.provider_ids),
        ..input
    }
}

pub fn sanitize_provider_group_update(input: ProviderGroupUpdate) -> ProviderGroupUpdate {
    ProviderGroupUpdate {
        name: input.name.map(|value| value.trim().to_owned()),
        description: trim_patch(input.description),
        provider_ids: sanitize_id_patch(input.provider_ids),
        ..input
    }
}

pub fn sanitize_provider_key_group(input: ProviderKeyGroupCreate) -> ProviderKeyGroupCreate {
    ProviderKeyGroupCreate {
        name: input.name.trim().to_owned(),
        description: input.description.and_then(trim_optional),
        provider_key_ids: normalize_ids(input.provider_key_ids),
        ..input
    }
}

pub fn sanitize_provider_key_group_update(input: ProviderKeyGroupUpdate) -> ProviderKeyGroupUpdate {
    ProviderKeyGroupUpdate {
        name: input.name.map(|value| value.trim().to_owned()),
        description: trim_patch(input.description),
        provider_key_ids: sanitize_id_patch(input.provider_key_ids),
        ..input
    }
}

pub fn sanitize_provider_group_list_request(input: ProviderGroupListRequest) -> ProviderGroupListRequest {
    ProviderGroupListRequest {
        search: input.search.and_then(trim_optional),
        ..input
    }
}

pub fn validate_provider_group(input: &ProviderGroupCreate) -> ProviderResult<()> {
    validate_text("name", &input.name, MAX_NAME_LENGTH)?;
    validate_description(input.description.as_deref())?;
    validate_ids("provider_ids", &input.provider_ids)
}

pub fn validate_provider_group_update(input: &ProviderGroupUpdate) -> ProviderResult<()> {
    if input.is_empty() {
        return Err(ProviderError::InvalidInput("update payload is empty".into()));
    }
    if let Some(name) = input.name.as_deref() {
        validate_text("name", name, MAX_NAME_LENGTH)?;
    }
    validate_description_patch(&input.description)?;
    validate_id_patch("provider_ids", &input.provider_ids)
}

pub fn validate_provider_key_group(input: &ProviderKeyGroupCreate) -> ProviderResult<()> {
    validate_text("name", &input.name, MAX_NAME_LENGTH)?;
    validate_description(input.description.as_deref())?;
    validate_ids("provider_key_ids", &input.provider_key_ids)
}

pub fn validate_provider_key_group_update(input: &ProviderKeyGroupUpdate) -> ProviderResult<()> {
    if input.is_empty() {
        return Err(ProviderError::InvalidInput("update payload is empty".into()));
    }
    if let Some(name) = input.name.as_deref() {
        validate_text("name", name, MAX_NAME_LENGTH)?;
    }
    validate_description_patch(&input.description)?;
    validate_id_patch("provider_key_ids", &input.provider_key_ids)
}

pub fn validate_provider_group_list_request(request: &ProviderGroupListRequest) -> ProviderResult<()> {
    if request.limit == 0 || request.limit > MAX_LIST_LIMIT {
        return Err(ProviderError::InvalidInput(format!("limit must be between 1 and {MAX_LIST_LIMIT}")));
    }
    Ok(())
}

fn validate_description(value: Option<&str>) -> ProviderResult<()> {
    if value.is_some_and(|text| text.len() > MAX_DESCRIPTION_LENGTH) {
        return Err(ProviderError::InvalidInput(format!(
            "description length must be at most {MAX_DESCRIPTION_LENGTH}"
        )));
    }
    Ok(())
}

fn validate_description_patch(patch: &PatchField<String>) -> ProviderResult<()> {
    match patch {
        PatchField::Value(value) => validate_description(Some(value)),
        PatchField::Null | PatchField::Missing => Ok(()),
    }
}

fn validate_id_patch(field: &str, patch: &PatchField<Vec<String>>) -> ProviderResult<()> {
    match patch {
        PatchField::Value(value) => validate_ids(field, value),
        PatchField::Null | PatchField::Missing => Ok(()),
    }
}

fn validate_ids(field: &str, values: &[String]) -> ProviderResult<()> {
    if values.iter().any(|value| value.trim().is_empty()) {
        return Err(ProviderError::InvalidInput(format!("{field} cannot contain blank values")));
    }
    Ok(())
}

fn sanitize_id_patch(patch: PatchField<Vec<String>>) -> PatchField<Vec<String>> {
    match patch {
        PatchField::Value(value) => PatchField::Value(normalize_ids(value)),
        PatchField::Null => PatchField::Value(Vec::new()),
        PatchField::Missing => PatchField::Missing,
    }
}

fn normalize_ids(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

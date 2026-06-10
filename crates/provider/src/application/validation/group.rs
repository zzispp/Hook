use types::{
    model::PatchField,
    provider::{
        ProviderGroupCreate, ProviderGroupListRequest, ProviderGroupMemberInput, ProviderGroupUpdate, ProviderKeyGroupCreate, ProviderKeyGroupMemberInput,
        ProviderKeyGroupUpdate,
    },
};

use super::{MAX_DESCRIPTION_LENGTH, MAX_LIST_LIMIT, MAX_NAME_LENGTH, trim_optional, trim_patch, validate_text};
use crate::application::{ProviderError, ProviderResult};

pub fn sanitize_provider_group(input: ProviderGroupCreate) -> ProviderGroupCreate {
    ProviderGroupCreate {
        name: input.name.trim().to_owned(),
        description: input.description.and_then(trim_optional),
        provider_members: sanitize_provider_members(input.provider_members),
        ..input
    }
}

pub fn sanitize_provider_group_update(input: ProviderGroupUpdate) -> ProviderGroupUpdate {
    ProviderGroupUpdate {
        name: input.name.map(|value| value.trim().to_owned()),
        description: trim_patch(input.description),
        provider_members: sanitize_provider_member_patch(input.provider_members),
        ..input
    }
}

pub fn sanitize_provider_key_group(input: ProviderKeyGroupCreate) -> ProviderKeyGroupCreate {
    ProviderKeyGroupCreate {
        name: input.name.trim().to_owned(),
        description: input.description.and_then(trim_optional),
        provider_key_members: sanitize_provider_key_members(input.provider_key_members),
        ..input
    }
}

pub fn sanitize_provider_key_group_update(input: ProviderKeyGroupUpdate) -> ProviderKeyGroupUpdate {
    ProviderKeyGroupUpdate {
        name: input.name.map(|value| value.trim().to_owned()),
        description: trim_patch(input.description),
        provider_key_members: sanitize_provider_key_member_patch(input.provider_key_members),
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
    validate_provider_members(&input.provider_members)
}

pub fn validate_provider_group_update(input: &ProviderGroupUpdate) -> ProviderResult<()> {
    if input.is_empty() {
        return Err(ProviderError::InvalidInput("update payload is empty".into()));
    }
    if let Some(name) = input.name.as_deref() {
        validate_text("name", name, MAX_NAME_LENGTH)?;
    }
    validate_description_patch(&input.description)?;
    validate_provider_member_patch(&input.provider_members)
}

pub fn validate_provider_key_group(input: &ProviderKeyGroupCreate) -> ProviderResult<()> {
    validate_text("name", &input.name, MAX_NAME_LENGTH)?;
    validate_description(input.description.as_deref())?;
    validate_provider_key_members(&input.provider_key_members)
}

pub fn validate_provider_key_group_update(input: &ProviderKeyGroupUpdate) -> ProviderResult<()> {
    if input.is_empty() {
        return Err(ProviderError::InvalidInput("update payload is empty".into()));
    }
    if let Some(name) = input.name.as_deref() {
        validate_text("name", name, MAX_NAME_LENGTH)?;
    }
    validate_description_patch(&input.description)?;
    validate_provider_key_member_patch(&input.provider_key_members)
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

fn validate_provider_member_patch(patch: &PatchField<Vec<ProviderGroupMemberInput>>) -> ProviderResult<()> {
    match patch {
        PatchField::Value(value) => validate_provider_members(value),
        PatchField::Null | PatchField::Missing => Ok(()),
    }
}

fn validate_provider_key_member_patch(patch: &PatchField<Vec<ProviderKeyGroupMemberInput>>) -> ProviderResult<()> {
    match patch {
        PatchField::Value(value) => validate_provider_key_members(value),
        PatchField::Null | PatchField::Missing => Ok(()),
    }
}

fn validate_provider_members(values: &[ProviderGroupMemberInput]) -> ProviderResult<()> {
    let ids = values.iter().map(|member| member.provider_id.as_str()).collect::<Vec<_>>();
    validate_member_ids("provider_members", &ids)
}

fn validate_provider_key_members(values: &[ProviderKeyGroupMemberInput]) -> ProviderResult<()> {
    let ids = values.iter().map(|member| member.provider_key_id.as_str()).collect::<Vec<_>>();
    validate_member_ids("provider_key_members", &ids)
}

fn validate_member_ids(field: &str, values: &[&str]) -> ProviderResult<()> {
    let mut seen = std::collections::BTreeSet::new();
    for value in values {
        if value.trim().is_empty() {
            return Err(ProviderError::InvalidInput(format!("{field} cannot contain blank values")));
        }
        if !seen.insert(*value) {
            return Err(ProviderError::InvalidInput(format!("{field} cannot contain duplicate ids")));
        }
    }
    Ok(())
}

fn sanitize_provider_member_patch(patch: PatchField<Vec<ProviderGroupMemberInput>>) -> PatchField<Vec<ProviderGroupMemberInput>> {
    match patch {
        PatchField::Value(value) => PatchField::Value(sanitize_provider_members(value)),
        PatchField::Null => PatchField::Value(Vec::new()),
        PatchField::Missing => PatchField::Missing,
    }
}

fn sanitize_provider_key_member_patch(patch: PatchField<Vec<ProviderKeyGroupMemberInput>>) -> PatchField<Vec<ProviderKeyGroupMemberInput>> {
    match patch {
        PatchField::Value(value) => PatchField::Value(sanitize_provider_key_members(value)),
        PatchField::Null => PatchField::Value(Vec::new()),
        PatchField::Missing => PatchField::Missing,
    }
}

fn sanitize_provider_members(values: Vec<ProviderGroupMemberInput>) -> Vec<ProviderGroupMemberInput> {
    values
        .into_iter()
        .map(|member| ProviderGroupMemberInput {
            provider_id: member.provider_id.trim().to_owned(),
            priority: member.priority,
        })
        .filter(|member| !member.provider_id.is_empty())
        .collect()
}

fn sanitize_provider_key_members(values: Vec<ProviderKeyGroupMemberInput>) -> Vec<ProviderKeyGroupMemberInput> {
    values
        .into_iter()
        .map(|member| ProviderKeyGroupMemberInput {
            provider_key_id: member.provider_key_id.trim().to_owned(),
            priority: member.priority,
        })
        .filter(|member| !member.provider_key_id.is_empty())
        .collect()
}

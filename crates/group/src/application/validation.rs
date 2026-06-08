use std::collections::BTreeSet;

use rust_decimal::Decimal;
use types::group::{BillingGroupCreate, BillingGroupListRequest, BillingGroupUpdate};

use super::{GroupError, GroupResult};

const MAX_CODE_LENGTH: usize = 64;
const MAX_NAME_LENGTH: usize = 100;
const MAX_DESCRIPTION_LENGTH: usize = 500;
const MAX_LIST_LIMIT: u64 = 1000;

pub fn sanitize_create(input: BillingGroupCreate) -> BillingGroupCreate {
    BillingGroupCreate {
        code: input.code.trim().to_owned(),
        name: input.name.trim().to_owned(),
        description: input.description.and_then(trim_optional),
        allowed_model_ids: normalize_ids(input.allowed_model_ids),
        allowed_provider_group_ids: normalize_ids(input.allowed_provider_group_ids),
        allowed_provider_key_group_ids: normalize_ids(input.allowed_provider_key_group_ids),
        ..input
    }
}

pub fn sanitize_update(input: BillingGroupUpdate) -> BillingGroupUpdate {
    BillingGroupUpdate {
        name: input.name.map(|value| value.trim().to_owned()),
        description: sanitize_patch_string(input.description),
        allowed_model_ids: sanitize_id_patch(input.allowed_model_ids),
        allowed_provider_group_ids: sanitize_id_patch(input.allowed_provider_group_ids),
        allowed_provider_key_group_ids: sanitize_id_patch(input.allowed_provider_key_group_ids),
        ..input
    }
}

pub fn validate_create(input: &BillingGroupCreate) -> GroupResult<()> {
    validate_code(&input.code)?;
    validate_name("name", &input.name)?;
    validate_description(input.description.as_deref())?;
    validate_multiplier(input.billing_multiplier)?;
    validate_ids("allowed_model_ids", &input.allowed_model_ids)?;
    validate_ids("allowed_provider_group_ids", &input.allowed_provider_group_ids)?;
    validate_ids("allowed_provider_key_group_ids", &input.allowed_provider_key_group_ids)?;
    validate_access_mode(&input.allowed_provider_group_ids, &input.allowed_provider_key_group_ids)
}

pub fn validate_update(input: &BillingGroupUpdate) -> GroupResult<()> {
    if input.is_empty() {
        return Err(GroupError::InvalidInput("update payload is empty".into()));
    }
    if let Some(name) = input.name.as_deref() {
        validate_name("name", name)?;
    }
    validate_description_patch(&input.description)?;
    if let Some(multiplier) = input.billing_multiplier {
        validate_multiplier(multiplier)?;
    }
    validate_id_patch("allowed_model_ids", &input.allowed_model_ids)?;
    validate_id_patch("allowed_provider_group_ids", &input.allowed_provider_group_ids)?;
    validate_id_patch("allowed_provider_key_group_ids", &input.allowed_provider_key_group_ids)?;
    Ok(())
}

pub fn validate_list_request(request: &BillingGroupListRequest) -> GroupResult<()> {
    if request.limit == 0 || request.limit > MAX_LIST_LIMIT {
        return Err(GroupError::InvalidInput(format!("limit must be between 1 and {MAX_LIST_LIMIT}")));
    }
    Ok(())
}

fn validate_code(value: &str) -> GroupResult<()> {
    validate_name("code", value)?;
    if value.len() > MAX_CODE_LENGTH {
        return Err(GroupError::InvalidInput(format!("code length must be between 1 and {MAX_CODE_LENGTH}")));
    }
    if !value.chars().all(|item| item.is_ascii_alphanumeric() || item == '_' || item == '-') {
        return Err(GroupError::InvalidInput(
            "code can only contain letters, numbers, underscores, and hyphens".into(),
        ));
    }
    Ok(())
}

fn validate_name(field: &str, value: &str) -> GroupResult<()> {
    if value.is_empty() || value.len() > MAX_NAME_LENGTH {
        return Err(GroupError::InvalidInput(format!("{field} length must be between 1 and {MAX_NAME_LENGTH}")));
    }
    Ok(())
}

fn validate_description(value: Option<&str>) -> GroupResult<()> {
    if value.is_some_and(|text| text.len() > MAX_DESCRIPTION_LENGTH) {
        return Err(GroupError::InvalidInput(format!("description length must be at most {MAX_DESCRIPTION_LENGTH}")));
    }
    Ok(())
}

fn validate_description_patch(patch: &types::model::PatchField<String>) -> GroupResult<()> {
    match patch {
        types::model::PatchField::Value(value) => validate_description(Some(value)),
        types::model::PatchField::Null | types::model::PatchField::Missing => Ok(()),
    }
}

fn validate_multiplier(value: Decimal) -> GroupResult<()> {
    if value <= Decimal::ZERO {
        return Err(GroupError::InvalidInput("billing_multiplier must be greater than 0".into()));
    }
    Ok(())
}

fn validate_id_patch(field: &str, patch: &types::model::PatchField<Vec<String>>) -> GroupResult<()> {
    match patch {
        types::model::PatchField::Value(value) => validate_ids(field, value),
        types::model::PatchField::Null | types::model::PatchField::Missing => Ok(()),
    }
}

fn validate_ids(field: &str, values: &[String]) -> GroupResult<()> {
    if values.iter().any(|value| value.trim().is_empty()) {
        return Err(GroupError::InvalidInput(format!("{field} cannot contain blank values")));
    }
    Ok(())
}

pub fn validate_access_mode(provider_group_ids: &[String], key_group_ids: &[String]) -> GroupResult<()> {
    if !provider_group_ids.is_empty() && !key_group_ids.is_empty() {
        return Err(GroupError::InvalidInput(
            "allowed_provider_group_ids and allowed_provider_key_group_ids cannot both be non-empty".into(),
        ));
    }
    Ok(())
}

fn trim_optional(value: String) -> Option<String> {
    let value = value.trim().to_owned();
    if value.is_empty() { None } else { Some(value) }
}

fn sanitize_patch_string(patch: types::model::PatchField<String>) -> types::model::PatchField<String> {
    match patch {
        types::model::PatchField::Value(value) => trim_optional(value).map_or(types::model::PatchField::Null, types::model::PatchField::Value),
        types::model::PatchField::Null => types::model::PatchField::Null,
        types::model::PatchField::Missing => types::model::PatchField::Missing,
    }
}

fn sanitize_id_patch(patch: types::model::PatchField<Vec<String>>) -> types::model::PatchField<Vec<String>> {
    match patch {
        types::model::PatchField::Value(value) => types::model::PatchField::Value(normalize_ids(value)),
        types::model::PatchField::Null => types::model::PatchField::Value(Vec::new()),
        types::model::PatchField::Missing => types::model::PatchField::Missing,
    }
}

fn normalize_ids(values: Vec<String>) -> Vec<String> {
    let mut set = BTreeSet::new();
    values
        .into_iter()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .for_each(|value| {
            set.insert(value);
        });
    set.into_iter().collect()
}

use std::collections::BTreeSet;

use constants::billing::DEFAULT_SYSTEM_GROUP_CODE;
use rust_decimal::Decimal;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use types::{
    api_token::{AdminApiTokenCreate, ApiToken, ApiTokenCreate, ApiTokenUpdate, ModelAccessMode},
    model::PatchField,
};

use super::{ApiTokenError, ApiTokenResult};

const MAX_NAME_LENGTH: usize = 100;
const MAX_LIST_LIMIT: u64 = 1000;

pub fn sanitize_create(input: ApiTokenCreate) -> ApiTokenCreate {
    ApiTokenCreate {
        name: input.name.trim().to_owned(),
        group_code: sanitize_optional_group_code(input.group_code),
        allowed_model_ids: normalize_model_ids(input.allowed_model_ids),
        ..input
    }
}

pub fn sanitize_admin_create(input: AdminApiTokenCreate) -> AdminApiTokenCreate {
    AdminApiTokenCreate {
        name: input.name.trim().to_owned(),
        user_id: input.user_id.map(|value| value.trim().to_owned()).filter(|value| !value.is_empty()),
        group_code: sanitize_optional_group_code(input.group_code),
        allowed_model_ids: normalize_model_ids(input.allowed_model_ids),
        ..input
    }
}

pub fn sanitize_update(input: ApiTokenUpdate) -> ApiTokenUpdate {
    ApiTokenUpdate {
        name: input.name.map(|value| value.trim().to_owned()),
        group_code: input.group_code.map(|value| value.trim().to_owned()),
        allowed_model_ids: sanitize_model_patch(input.allowed_model_ids),
        ..input
    }
}

pub fn validate_list_request(request: &types::api_token::ApiTokenListRequest) -> ApiTokenResult<()> {
    if request.limit == 0 || request.limit > MAX_LIST_LIMIT {
        return Err(ApiTokenError::InvalidInput(format!("limit must be between 1 and {MAX_LIST_LIMIT}")));
    }
    Ok(())
}

pub fn validate_create(input: &ApiTokenCreate) -> ApiTokenResult<ValidatedCreate> {
    validate_name(&input.name)?;
    let group_code = resolve_group_code(input.group_code.as_deref())?;
    validate_optional_positive("rate_limit_rpm", input.rate_limit_rpm)?;
    validate_optional_decimal("quota_limit", input.quota_limit)?;
    let expires_at = parse_optional_time(input.expires_at.as_deref())?;
    let mode = input.model_access_mode.unwrap_or(ModelAccessMode::All);
    validate_model_policy(mode, &input.allowed_model_ids)?;
    Ok(ValidatedCreate {
        group_code,
        expires_at,
        model_access_mode: mode,
        allowed_model_ids: input.allowed_model_ids.clone(),
    })
}

pub fn validate_admin_create(input: &AdminApiTokenCreate) -> ApiTokenResult<ValidatedCreate> {
    validate_name(&input.name)?;
    let group_code = resolve_group_code(input.group_code.as_deref())?;
    validate_optional_positive("rate_limit_rpm", input.rate_limit_rpm)?;
    validate_optional_decimal("quota_limit", input.quota_limit)?;
    let expires_at = parse_optional_time(input.expires_at.as_deref())?;
    let mode = input.model_access_mode.unwrap_or(ModelAccessMode::All);
    validate_model_policy(mode, &input.allowed_model_ids)?;
    Ok(ValidatedCreate {
        group_code,
        expires_at,
        model_access_mode: mode,
        allowed_model_ids: input.allowed_model_ids.clone(),
    })
}

pub fn validate_update(current: &ApiToken, input: &ApiTokenUpdate) -> ApiTokenResult<ValidatedUpdate> {
    if input.is_empty() {
        return Err(ApiTokenError::InvalidInput("update payload is empty".into()));
    }
    validate_update_fields(input)?;
    let expires_at = parse_time_patch(&input.expires_at)?;
    let mode = input.model_access_mode.unwrap_or(current.model_access_mode);
    let allowed_model_ids = effective_model_ids(current, input, mode)?;
    validate_model_policy(mode, &allowed_model_ids)?;
    Ok(ValidatedUpdate {
        expires_at,
        model_access_mode: mode,
        allowed_model_ids,
    })
}

pub fn model_ids_for_update(_current: &ApiToken, validated: &ValidatedUpdate, input: &ApiTokenUpdate) -> PatchField<Vec<String>> {
    if input.model_access_mode == Some(ModelAccessMode::All) {
        return PatchField::Value(Vec::new());
    }
    match &input.allowed_model_ids {
        PatchField::Value(_) => PatchField::Value(validated.allowed_model_ids.clone()),
        PatchField::Missing | PatchField::Null => PatchField::Missing,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidatedCreate {
    pub group_code: String,
    pub expires_at: Option<OffsetDateTime>,
    pub model_access_mode: ModelAccessMode,
    pub allowed_model_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidatedUpdate {
    pub expires_at: PatchField<OffsetDateTime>,
    pub model_access_mode: ModelAccessMode,
    pub allowed_model_ids: Vec<String>,
}

fn validate_update_fields(input: &ApiTokenUpdate) -> ApiTokenResult<()> {
    if let Some(name) = input.name.as_deref() {
        validate_name(name)?;
    }
    if let Some(code) = input.group_code.as_deref() {
        validate_group_code(code)?;
    }
    validate_optional_positive_patch("rate_limit_rpm", &input.rate_limit_rpm)?;
    validate_optional_decimal_patch("quota_limit", &input.quota_limit)
}

fn validate_name(value: &str) -> ApiTokenResult<()> {
    if value.is_empty() || value.len() > MAX_NAME_LENGTH {
        return Err(ApiTokenError::InvalidInput(format!("name length must be between 1 and {MAX_NAME_LENGTH}")));
    }
    Ok(())
}

fn validate_group_code(value: &str) -> ApiTokenResult<()> {
    if value.is_empty() {
        return Err(ApiTokenError::InvalidInput("group_code cannot be blank".into()));
    }
    Ok(())
}

fn resolve_group_code(value: Option<&str>) -> ApiTokenResult<String> {
    let Some(code) = value else {
        return Ok(DEFAULT_SYSTEM_GROUP_CODE.to_owned());
    };
    validate_group_code(code)?;
    Ok(code.to_owned())
}

fn validate_model_policy(mode: ModelAccessMode, model_ids: &[String]) -> ApiTokenResult<()> {
    match mode {
        ModelAccessMode::All if !model_ids.is_empty() => Err(ApiTokenError::InvalidInput(
            "allowed_model_ids must be empty when model_access_mode is all".into(),
        )),
        ModelAccessMode::Limited if model_ids.is_empty() => Err(ApiTokenError::InvalidInput(
            "allowed_model_ids cannot be empty when model_access_mode is limited".into(),
        )),
        _ => Ok(()),
    }
}

fn validate_optional_positive(field: &str, value: Option<i64>) -> ApiTokenResult<()> {
    if value.is_some_and(|item| item < 0) {
        return Err(ApiTokenError::InvalidInput(format!("{field} must be greater than or equal to 0")));
    }
    Ok(())
}

fn validate_optional_positive_patch(field: &str, patch: &PatchField<i64>) -> ApiTokenResult<()> {
    match patch {
        PatchField::Value(value) => validate_optional_positive(field, Some(*value)),
        PatchField::Null | PatchField::Missing => Ok(()),
    }
}

fn validate_optional_decimal(field: &str, value: Option<Decimal>) -> ApiTokenResult<()> {
    if value.is_some_and(|item| item <= Decimal::ZERO) {
        return Err(ApiTokenError::InvalidInput(format!("{field} must be greater than 0")));
    }
    Ok(())
}

fn validate_optional_decimal_patch(field: &str, patch: &PatchField<Decimal>) -> ApiTokenResult<()> {
    match patch {
        PatchField::Value(value) => validate_optional_decimal(field, Some(*value)),
        PatchField::Null | PatchField::Missing => Ok(()),
    }
}

fn parse_optional_time(value: Option<&str>) -> ApiTokenResult<Option<OffsetDateTime>> {
    value.map(parse_time).transpose()
}

fn parse_time_patch(patch: &PatchField<String>) -> ApiTokenResult<PatchField<OffsetDateTime>> {
    match patch {
        PatchField::Value(value) => parse_time(value).map(PatchField::Value),
        PatchField::Null => Ok(PatchField::Null),
        PatchField::Missing => Ok(PatchField::Missing),
    }
}

fn parse_time(value: &str) -> ApiTokenResult<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|error| ApiTokenError::InvalidInput(format!("invalid RFC3339 time: {error}")))
}

fn effective_model_ids(current: &ApiToken, input: &ApiTokenUpdate, mode: ModelAccessMode) -> ApiTokenResult<Vec<String>> {
    if mode == ModelAccessMode::All {
        return Ok(Vec::new());
    }
    match &input.allowed_model_ids {
        PatchField::Value(value) => Ok(value.clone()),
        PatchField::Missing => Ok(current.allowed_model_ids.clone()),
        PatchField::Null => Err(ApiTokenError::InvalidInput("allowed_model_ids cannot be null".into())),
    }
}

fn normalize_model_ids(values: Vec<String>) -> Vec<String> {
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

fn sanitize_optional_group_code(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_owned()).filter(|item| !item.is_empty())
}

fn sanitize_model_patch(patch: PatchField<Vec<String>>) -> PatchField<Vec<String>> {
    match patch {
        PatchField::Value(value) => PatchField::Value(normalize_model_ids(value)),
        PatchField::Null => PatchField::Null,
        PatchField::Missing => PatchField::Missing,
    }
}

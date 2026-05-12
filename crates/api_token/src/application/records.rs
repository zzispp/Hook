use types::api_token::{AdminApiTokenCreate, ApiTokenCreate, ApiTokenType, ApiTokenUpdate};

use crate::application::{
    ApiTokenCreateRecord, ApiTokenError, ApiTokenResult, ApiTokenUpdateRecord,
    token::GeneratedToken,
    validation::{ValidatedCreate, ValidatedUpdate, model_ids_for_update},
};

pub(super) fn user_create_record(user_id: &str, input: ApiTokenCreate, validated: ValidatedCreate, generated: &GeneratedToken) -> ApiTokenCreateRecord {
    let options = CreateRecordOptions::from_parts(input.rate_limit_rpm, input.quota_limit, validated, generated);
    create_record(Some(user_id.into()), ApiTokenType::User, input.name, options)
}

pub(super) fn admin_create_record(
    owner_id: Option<String>,
    input: AdminApiTokenCreate,
    validated: ValidatedCreate,
    generated: &GeneratedToken,
) -> ApiTokenCreateRecord {
    let token_type = input.token_type;
    let options = CreateRecordOptions::from_parts(input.rate_limit_rpm, input.quota_limit, validated, generated);
    create_record(owner_id, token_type, input.name, options)
}

pub(super) fn update_record(current: types::api_token::ApiToken, input: ApiTokenUpdate, validated: ValidatedUpdate) -> ApiTokenUpdateRecord {
    let allowed_model_ids = model_ids_for_update(&current, &validated, &input);
    ApiTokenUpdateRecord {
        name: input.name,
        group_code: input.group_code,
        expires_at: validated.expires_at,
        model_access_mode: input.model_access_mode,
        allowed_model_ids,
        rate_limit_rpm: input.rate_limit_rpm,
        quota_limit: input.quota_limit,
        is_active: input.is_active,
    }
}

pub(super) fn admin_owner_id(input: &AdminApiTokenCreate) -> ApiTokenResult<Option<String>> {
    match input.token_type {
        ApiTokenType::Independent => Ok(input.user_id.clone()),
        ApiTokenType::User => input
            .user_id
            .clone()
            .map(Some)
            .ok_or_else(|| ApiTokenError::InvalidInput("user_id is required for user token".into())),
    }
}

fn create_record(user_id: Option<String>, token_type: ApiTokenType, name: String, options: CreateRecordOptions) -> ApiTokenCreateRecord {
    ApiTokenCreateRecord {
        user_id,
        token_type,
        name,
        token_value: options.token_value,
        token_hash: options.token_hash,
        token_prefix: options.token_prefix,
        group_code: options.group_code,
        expires_at: options.expires_at,
        model_access_mode: options.model_access_mode,
        allowed_model_ids: options.allowed_model_ids,
        rate_limit_rpm: options.rate_limit_rpm,
        quota_limit: options.quota_limit,
    }
}

struct CreateRecordOptions {
    token_value: String,
    token_hash: String,
    token_prefix: String,
    group_code: String,
    expires_at: Option<time::OffsetDateTime>,
    model_access_mode: types::api_token::ModelAccessMode,
    allowed_model_ids: Vec<String>,
    rate_limit_rpm: Option<i64>,
    quota_limit: Option<rust_decimal::Decimal>,
}

impl CreateRecordOptions {
    fn from_parts(rate_limit_rpm: Option<i64>, quota_limit: Option<rust_decimal::Decimal>, validated: ValidatedCreate, generated: &GeneratedToken) -> Self {
        Self {
            token_value: generated.value.clone(),
            token_hash: generated.hash.clone(),
            token_prefix: generated.prefix.clone(),
            group_code: validated.group_code,
            expires_at: validated.expires_at,
            model_access_mode: validated.model_access_mode,
            allowed_model_ids: validated.allowed_model_ids,
            rate_limit_rpm: Some(rate_limit_rpm.unwrap_or(0)),
            quota_limit,
        }
    }
}

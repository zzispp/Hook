use rust_decimal::Decimal;
use types::{
    api_token::{AdminApiTokenCreate, ApiToken, ApiTokenCreate, ApiTokenType, ApiTokenUpdate, ModelAccessMode},
    model::PatchField,
};

use super::validation::{sanitize_admin_create, sanitize_create, sanitize_update, validate_admin_create, validate_create, validate_update};

const PAST_EXPIRES_AT: &str = "2000-01-01T00:00:00Z";

#[test]
fn create_without_group_uses_system_group() {
    let input = sanitize_create(ApiTokenCreate {
        name: "billing token".into(),
        group_code: None,
        expires_at: None,
        model_access_mode: Some(ModelAccessMode::All),
        allowed_model_ids: Vec::new(),
        rate_limit_rpm: Some(0),
        quota_limit: None,
    });

    let validated = validate_create(&input).expect("token create should be valid");

    assert_eq!(validated.group_code, constants::billing::DEFAULT_SYSTEM_GROUP_CODE);
}

#[test]
fn create_rejects_past_expires_at() {
    let input = sanitize_create(ApiTokenCreate {
        expires_at: Some(PAST_EXPIRES_AT.into()),
        ..create_input()
    });

    let result = validate_create(&input);

    assert!(result.is_err_and(|error| error.to_string().contains("expires_at must be in the future")));
}

#[test]
fn admin_create_rejects_past_expires_at() {
    let input = sanitize_admin_create(AdminApiTokenCreate {
        expires_at: Some(PAST_EXPIRES_AT.into()),
        ..admin_create_input()
    });

    let result = validate_admin_create(&input);

    assert!(result.is_err_and(|error| error.to_string().contains("expires_at must be in the future")));
}

#[test]
fn update_rejects_past_expires_at() {
    let current = api_token();
    let input = sanitize_update(ApiTokenUpdate {
        expires_at: PatchField::Value(PAST_EXPIRES_AT.into()),
        ..ApiTokenUpdate::default()
    });

    let result = validate_update(&current, &input);

    assert!(result.is_err_and(|error| error.to_string().contains("expires_at must be in the future")));
}

fn create_input() -> ApiTokenCreate {
    ApiTokenCreate {
        name: "billing token".into(),
        group_code: None,
        expires_at: None,
        model_access_mode: Some(ModelAccessMode::All),
        allowed_model_ids: Vec::new(),
        rate_limit_rpm: Some(0),
        quota_limit: None,
    }
}

fn admin_create_input() -> AdminApiTokenCreate {
    AdminApiTokenCreate {
        name: "billing token".into(),
        token_type: ApiTokenType::Independent,
        user_id: Some("admin-1".into()),
        group_code: None,
        expires_at: None,
        model_access_mode: Some(ModelAccessMode::All),
        allowed_model_ids: Vec::new(),
        rate_limit_rpm: Some(0),
        quota_limit: None,
    }
}

fn api_token() -> ApiToken {
    ApiToken {
        id: "token-1".into(),
        user_id: Some("user-1".into()),
        token_type: ApiTokenType::User,
        name: "billing token".into(),
        token_value: "sk-test".into(),
        token_hash: "hash".into(),
        token_prefix: "sk".into(),
        group_code: constants::billing::DEFAULT_SYSTEM_GROUP_CODE.into(),
        expires_at: None,
        model_access_mode: ModelAccessMode::All,
        allowed_model_ids: Vec::new(),
        rate_limit_rpm: Some(0),
        quota_limit: None,
        used_quota: Decimal::ZERO,
        request_count: 0,
        is_active: true,
        last_used_at: None,
        created_at: "2026-05-11T00:00:00Z".into(),
        updated_at: "2026-05-11T00:00:00Z".into(),
    }
}

#[test]
fn create_with_blank_group_uses_system_group() {
    let input = sanitize_create(ApiTokenCreate {
        name: "billing token".into(),
        group_code: Some("  ".into()),
        expires_at: None,
        model_access_mode: Some(ModelAccessMode::All),
        allowed_model_ids: Vec::new(),
        rate_limit_rpm: Some(0),
        quota_limit: None,
    });

    let validated = validate_create(&input).expect("token create should be valid");

    assert_eq!(validated.group_code, constants::billing::DEFAULT_SYSTEM_GROUP_CODE);
}

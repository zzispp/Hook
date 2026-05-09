use types::api_token::{ApiTokenCreate, ModelAccessMode};

use super::validation::{sanitize_create, validate_create};

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

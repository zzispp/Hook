use std::collections::BTreeSet;

use constants::auth::{PASSWORD_MAX_LENGTH, PASSWORD_MIN_LENGTH, USERNAME_MAX_LENGTH, USERNAME_MIN_LENGTH};
use constants::pagination::{MAX_PAGE_SIZE, MIN_PAGE_NUMBER, MIN_PAGE_SIZE};
use types::{
    pagination::PageRequest,
    user::{Credentials, NewUser, ReplaceUser, USER_QUOTA_MODE_UNLIMITED, USER_QUOTA_MODE_WALLET},
};

use crate::application::{AppError, AppResult};

pub(super) fn validate_credentials(input: &Credentials) -> AppResult<()> {
    reject_blank("identifier", &input.identifier)?;
    validate_password(&input.password)
}

pub(super) fn validate_new_user(input: &NewUser) -> AppResult<()> {
    validate_username(&input.username)?;
    validate_password(&input.password)?;
    reject_blank("email", &input.email)?;
    reject_blank("role", &input.role)?;
    validate_ids("allowed_model_ids", &input.allowed_model_ids)?;
    validate_ids("allowed_provider_ids", &input.allowed_provider_ids)?;
    validate_rate_limit(input.rate_limit_rpm)?;
    validate_quota_mode(&input.quota_mode)
}

pub(super) fn validate_replace_user(input: &ReplaceUser) -> AppResult<()> {
    validate_username(&input.username)?;
    if let Some(password) = &input.password {
        validate_password(password)?;
    }
    reject_blank("email", &input.email)?;
    reject_blank("role", &input.role)?;
    validate_ids("allowed_model_ids", &input.allowed_model_ids)?;
    validate_ids("allowed_provider_ids", &input.allowed_provider_ids)?;
    validate_rate_limit(input.rate_limit_rpm)?;
    validate_quota_mode(&input.quota_mode)
}

pub(super) fn validate_page(page: PageRequest) -> AppResult<()> {
    if page.page < MIN_PAGE_NUMBER {
        return Err(AppError::InvalidInput("page must be greater than 0".into()));
    }
    if page.page_size < MIN_PAGE_SIZE {
        return Err(AppError::InvalidInput("page_size must be greater than 0".into()));
    }
    if page.page_size > MAX_PAGE_SIZE {
        return Err(AppError::InvalidInput(format!("page_size must be less than or equal to {MAX_PAGE_SIZE}")));
    }
    Ok(())
}

pub(super) fn sanitize_credentials(input: Credentials) -> Credentials {
    Credentials {
        identifier: input.identifier.trim().into(),
        password: input.password.trim().into(),
    }
}

pub(super) fn sanitize_new_user(input: NewUser) -> NewUser {
    NewUser {
        username: input.username.trim().into(),
        password: input.password.trim().into(),
        email: input.email.trim().into(),
        role: input.role,
        is_active: input.is_active,
        allowed_model_ids: normalize_ids(input.allowed_model_ids),
        allowed_provider_ids: normalize_ids(input.allowed_provider_ids),
        rate_limit_rpm: input.rate_limit_rpm,
        quota_mode: input.quota_mode,
    }
}

pub(super) fn sanitize_replace_user(input: ReplaceUser) -> ReplaceUser {
    ReplaceUser {
        username: input.username.trim().into(),
        password: nonblank_password(input.password),
        email: input.email.trim().into(),
        role: input.role,
        is_active: input.is_active,
        allowed_model_ids: normalize_ids(input.allowed_model_ids),
        allowed_provider_ids: normalize_ids(input.allowed_provider_ids),
        rate_limit_rpm: input.rate_limit_rpm,
        quota_mode: input.quota_mode,
    }
}

fn validate_username(username: &str) -> AppResult<()> {
    reject_length("username", username, USERNAME_MIN_LENGTH, USERNAME_MAX_LENGTH)?;
    if !username.chars().all(is_username_character) {
        return Err(AppError::InvalidInput(
            "username can only contain letters, numbers, underscores, and hyphens".into(),
        ));
    }
    if !has_alphanumeric_edges(username) {
        return Err(AppError::InvalidInput("username must start and end with a letter or number".into()));
    }
    Ok(())
}

fn nonblank_password(password: Option<String>) -> Option<String> {
    let password = password?.trim().to_owned();
    if password.is_empty() {
        return None;
    }
    Some(password)
}

pub(super) fn validate_password(password: &str) -> AppResult<()> {
    reject_length("password", password, PASSWORD_MIN_LENGTH, PASSWORD_MAX_LENGTH)
}

fn reject_length(field: &str, value: &str, min: usize, max: usize) -> AppResult<()> {
    let length = value.chars().count();
    if length < min || length > max {
        return Err(AppError::InvalidInput(format!("{field} must be between {min} and {max} characters")));
    }
    Ok(())
}

fn is_username_character(value: char) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, '_' | '-')
}

fn has_alphanumeric_edges(value: &str) -> bool {
    value
        .chars()
        .next()
        .zip(value.chars().next_back())
        .is_some_and(|(first, last)| first.is_ascii_alphanumeric() && last.is_ascii_alphanumeric())
}

fn reject_blank(field: &str, value: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::InvalidInput(format!("{field} cannot be blank")));
    }
    Ok(())
}

fn validate_ids(field: &str, values: &[String]) -> AppResult<()> {
    if values.iter().any(|value| value.trim().is_empty()) {
        return Err(AppError::InvalidInput(format!("{field} cannot contain blank values")));
    }
    Ok(())
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

fn validate_rate_limit(value: Option<i64>) -> AppResult<()> {
    if value.is_some_and(|rate| rate < 0) {
        return Err(AppError::InvalidInput("rate_limit_rpm must be greater than or equal to 0".into()));
    }
    Ok(())
}

fn validate_quota_mode(value: &str) -> AppResult<()> {
    if matches!(value, USER_QUOTA_MODE_WALLET | USER_QUOTA_MODE_UNLIMITED) {
        return Ok(());
    }
    Err(AppError::InvalidInput("quota_mode must be wallet or unlimited".into()))
}

use std::collections::BTreeSet;

use constants::auth::{PASSWORD_MAX_LENGTH, PASSWORD_MIN_LENGTH, USERNAME_MAX_LENGTH, USERNAME_MIN_LENGTH};
use constants::pagination::{MAX_PAGE_SIZE, MIN_PAGE_NUMBER, MIN_PAGE_SIZE};
use types::{
    pagination::{PageRequest, PageSliceRequest},
    user::{Credentials, NewUser, RegistrationEmailCodeRequest, ReplaceUser, SignUpUser, USER_QUOTA_MODE_UNLIMITED, USER_QUOTA_MODE_WALLET},
};

use crate::application::{AppError, AppResult, PasswordResetConfirm, PasswordResetRequest};

const MAX_LANG_LEN: usize = 32;
const MAX_RESET_ORIGIN_LEN: usize = 512;
const RESET_TOKEN_MIN_LENGTH: usize = 32;
const RESET_TOKEN_MAX_LENGTH: usize = 512;
const EMAIL_VERIFICATION_CODE_LENGTH: usize = 6;

pub(super) fn validate_credentials(input: &Credentials) -> AppResult<()> {
    reject_blank("identifier", &input.identifier)?;
    validate_password(&input.password)
}

pub(super) fn validate_new_user(input: &NewUser) -> AppResult<()> {
    validate_username(&input.username)?;
    validate_password(&input.password)?;
    validate_new_user_without_password(input)
}

pub(super) fn validate_passwordless_new_user(input: &NewUser) -> AppResult<()> {
    validate_username(&input.username)?;
    validate_new_user_without_password(input)
}

fn validate_new_user_without_password(input: &NewUser) -> AppResult<()> {
    validate_email(&input.email)?;
    reject_blank("role", &input.role)?;
    validate_optional_group_codes(&input.group_codes)?;
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
    validate_email(&input.email)?;
    reject_blank("role", &input.role)?;
    validate_group_codes(&input.group_codes)?;
    validate_ids("allowed_model_ids", &input.allowed_model_ids)?;
    validate_ids("allowed_provider_ids", &input.allowed_provider_ids)?;
    validate_rate_limit(input.rate_limit_rpm)?;
    validate_quota_mode(&input.quota_mode)
}

pub(super) fn validate_password_reset_request(input: &PasswordResetRequest) -> AppResult<()> {
    validate_email(&input.email)?;
    validate_lang(&input.lang)?;
    validate_reset_origin(&input.reset_origin)
}

pub(super) fn validate_password_reset_confirm(input: &PasswordResetConfirm) -> AppResult<()> {
    reject_length("reset token", &input.token, RESET_TOKEN_MIN_LENGTH, RESET_TOKEN_MAX_LENGTH)?;
    validate_password(&input.password)
}

pub(super) fn validate_registration_email_code_request(input: &RegistrationEmailCodeRequest) -> AppResult<()> {
    validate_email(&input.email)?;
    validate_lang(&input.lang)
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

pub(super) fn validate_page_slice(request: PageSliceRequest) -> AppResult<()> {
    validate_page(PageRequest {
        page: request.page,
        page_size: request.page_size,
    })?;
    if request.limit != request.page_size {
        return Err(AppError::InvalidInput("limit must equal page_size".into()));
    }
    let expected_offset = (request.page - 1) * request.page_size;
    if request.offset != expected_offset {
        return Err(AppError::InvalidInput("offset must match page and page_size".into()));
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
        group_codes: input.group_codes.map(normalize_ids),
        is_active: input.is_active,
        allowed_model_ids: normalize_ids(input.allowed_model_ids),
        allowed_provider_ids: normalize_ids(input.allowed_provider_ids),
        rate_limit_rpm: input.rate_limit_rpm,
        quota_mode: input.quota_mode,
        referrer_aff_code: trim_optional_nonempty(input.referrer_aff_code),
    }
}

pub(super) fn sanitize_replace_user(input: ReplaceUser) -> ReplaceUser {
    ReplaceUser {
        username: input.username.trim().into(),
        password: nonblank_password(input.password),
        email: input.email.trim().into(),
        role: input.role,
        group_codes: normalize_ids(input.group_codes),
        is_active: input.is_active,
        allowed_model_ids: normalize_ids(input.allowed_model_ids),
        allowed_provider_ids: normalize_ids(input.allowed_provider_ids),
        rate_limit_rpm: input.rate_limit_rpm,
        quota_mode: input.quota_mode,
    }
}

pub(super) fn sanitize_password_reset_request(input: PasswordResetRequest) -> PasswordResetRequest {
    PasswordResetRequest {
        email: input.email.trim().to_ascii_lowercase(),
        lang: input.lang.trim().to_ascii_lowercase(),
        reset_origin: input.reset_origin.trim().trim_end_matches('/').to_owned(),
    }
}

pub(super) fn sanitize_password_reset_confirm(input: PasswordResetConfirm) -> PasswordResetConfirm {
    PasswordResetConfirm {
        token: input.token.trim().to_owned(),
        password: input.password.trim().to_owned(),
    }
}

pub(super) fn sanitize_registration_email_code_request(input: RegistrationEmailCodeRequest) -> RegistrationEmailCodeRequest {
    RegistrationEmailCodeRequest {
        email: input.email.trim().to_ascii_lowercase(),
        lang: input.lang.trim().to_ascii_lowercase(),
    }
}

pub(super) fn sanitize_sign_up_user(input: SignUpUser) -> SignUpUser {
    SignUpUser {
        user: sanitize_new_user(input.user),
        email_verification_code: input.email_verification_code.map(|value| value.trim().to_owned()),
        aff_code: trim_optional_nonempty(input.aff_code),
    }
}

pub(super) fn validate_email_verification_code(code: &str) -> AppResult<()> {
    if code.len() != EMAIL_VERIFICATION_CODE_LENGTH || !code.chars().all(|value| value.is_ascii_digit()) {
        return Err(AppError::InvalidInput("email verification code must be 6 digits".into()));
    }
    Ok(())
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

fn trim_optional_nonempty(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_owned()).filter(|item| !item.is_empty())
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

fn validate_email(email: &str) -> AppResult<()> {
    reject_blank("email", email)?;
    if email.chars().any(char::is_whitespace) || email.matches('@').count() != 1 {
        return Err(invalid_email());
    }
    let (local, domain) = email.split_once('@').ok_or_else(invalid_email)?;
    if !valid_email_local(local) || !valid_email_domain(domain) {
        return Err(invalid_email());
    }
    Ok(())
}

fn valid_email_local(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('.')
        && !value.ends_with('.')
        && value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '%' | '+' | '-'))
}

fn valid_email_domain(value: &str) -> bool {
    value.contains('.')
        && value
            .split('.')
            .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-'))
}

fn invalid_email() -> AppError {
    AppError::InvalidInput("email must be a valid email address".into())
}

fn reject_blank(field: &str, value: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::InvalidInput(format!("{field} cannot be blank")));
    }
    Ok(())
}

fn validate_optional_group_codes(value: &Option<Vec<String>>) -> AppResult<()> {
    match value {
        Some(group_codes) => validate_group_codes(group_codes),
        None => Ok(()),
    }
}

fn validate_group_codes(group_codes: &[String]) -> AppResult<()> {
    if group_codes.is_empty() {
        return Err(AppError::InvalidInput("group_codes cannot be empty".into()));
    }
    validate_ids("group_codes", group_codes)
}

fn validate_lang(value: &str) -> AppResult<()> {
    reject_length("lang", value, 1, MAX_LANG_LEN)?;
    if !value.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_') {
        return Err(AppError::InvalidInput("lang contains invalid characters".into()));
    }
    Ok(())
}

fn validate_reset_origin(value: &str) -> AppResult<()> {
    reject_length("reset_origin", value, 1, MAX_RESET_ORIGIN_LEN)?;
    let has_valid_scheme = value.starts_with("https://") || value.starts_with("http://");
    let has_path = value.trim_start_matches("https://").trim_start_matches("http://").contains('/');
    if !has_valid_scheme || has_path {
        return Err(AppError::InvalidInput("reset_origin must be an http or https origin".into()));
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

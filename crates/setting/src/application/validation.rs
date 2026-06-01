use axum::http::HeaderName;
use rust_decimal::Decimal;
use types::system_setting::{SystemSettingsUpdate, public_base_url_is_valid};

use super::contact_methods::{sanitize_contact_methods, validate_contact_methods};
use super::mail_validation::validate_mail_settings;
use super::provider_cooldown::validate_provider_cooldown_policy;
use super::{SettingError, SettingResult};

const MAX_SITE_NAME_LENGTH: usize = 100;
const MAX_SITE_SUBTITLE_LENGTH: usize = 200;
const MAX_PUBLIC_BASE_URL_LENGTH: usize = 255;
const MAX_AUTH_CLIENT_ID_LENGTH: usize = 255;
const MAX_AUTH_CLIENT_SECRET_LENGTH: usize = 2048;
const MAX_AUTH_WALLET_STATEMENT_LENGTH: usize = 200;
const HEADER_SEPARATOR: &str = ", ";

pub fn sanitize_update(input: SystemSettingsUpdate) -> SystemSettingsUpdate {
    SystemSettingsUpdate {
        site_name: input.site_name.map(|value| value.trim().to_owned()),
        site_subtitle: input.site_subtitle.map(|value| value.trim().to_owned()),
        public_base_url: trim_optional(input.public_base_url),
        site_logo_base64: trim_optional(input.site_logo_base64),
        contact_methods: input.contact_methods.map(sanitize_contact_methods),
        default_user_group_code: trim_optional(input.default_user_group_code),
        client_sensitive_request_headers: normalize_optional_headers(input.client_sensitive_request_headers),
        provider_sensitive_request_headers: normalize_optional_headers(input.provider_sensitive_request_headers),
        auth_github_client_id: trim_optional(input.auth_github_client_id),
        auth_github_client_secret: trim_optional(input.auth_github_client_secret),
        auth_google_client_id: trim_optional(input.auth_google_client_id),
        auth_google_client_secret: trim_optional(input.auth_google_client_secret),
        auth_evm_chain_ids: trim_optional(input.auth_evm_chain_ids),
        auth_evm_statement: trim_optional(input.auth_evm_statement),
        smtp_host: trim_optional(input.smtp_host),
        smtp_username: trim_optional(input.smtp_username),
        smtp_password: trim_optional(input.smtp_password),
        smtp_from_email: trim_optional(input.smtp_from_email),
        smtp_from_name: trim_optional(input.smtp_from_name),
        email_suffixes: input.email_suffixes.map(|value| normalize_email_suffixes(&value)),
        email_template_registration_subject: trim_optional(input.email_template_registration_subject),
        email_template_password_reset_subject: trim_optional(input.email_template_password_reset_subject),
        ..input
    }
}

pub fn validate_update(input: &SystemSettingsUpdate) -> SettingResult<()> {
    if input.is_empty() {
        return Err(SettingError::InvalidInput("update payload is empty".into()));
    }
    validate_site_name(input.site_name.as_deref())?;
    validate_site_subtitle(input.site_subtitle.as_deref())?;
    validate_public_base_url(input.public_base_url.as_deref())?;
    validate_contact_methods(input.contact_methods.as_deref())?;
    validate_optional_code("default_user_group_code", input.default_user_group_code.as_deref())?;
    validate_positive_i64("client_max_request_body_size_kb", input.client_max_request_body_size_kb)?;
    validate_positive_i64("client_max_response_body_size_kb", input.client_max_response_body_size_kb)?;
    validate_positive_i64("provider_max_request_body_size_kb", input.provider_max_request_body_size_kb)?;
    validate_positive_i64("provider_max_response_body_size_kb", input.provider_max_response_body_size_kb)?;
    validate_sensitive_headers("client_sensitive_request_headers", input.client_sensitive_request_headers.as_deref())?;
    validate_sensitive_headers("provider_sensitive_request_headers", input.provider_sensitive_request_headers.as_deref())?;
    validate_non_negative_decimal("default_user_grant", input.default_user_grant)?;
    validate_non_negative_i64("default_rate_limit_rpm", input.default_rate_limit_rpm)?;
    validate_recharge_settings(input)?;
    validate_positive_i64("token_limit_per_user", input.token_limit_per_user)?;
    validate_positive_i64("cache_affinity_ttl_minutes", input.cache_affinity_ttl_minutes)?;
    validate_provider_cooldown_policy(input.provider_cooldown_policy.as_ref())?;
    validate_auth_provider_settings(input)?;
    validate_mail_settings(input)
}

fn validate_auth_provider_settings(input: &SystemSettingsUpdate) -> SettingResult<()> {
    validate_optional_length("auth_github_client_id", input.auth_github_client_id.as_deref(), MAX_AUTH_CLIENT_ID_LENGTH)?;
    validate_required_optional_length(
        "auth_github_client_secret",
        input.auth_github_client_secret.as_deref(),
        MAX_AUTH_CLIENT_SECRET_LENGTH,
    )?;
    validate_optional_length("auth_google_client_id", input.auth_google_client_id.as_deref(), MAX_AUTH_CLIENT_ID_LENGTH)?;
    validate_required_optional_length(
        "auth_google_client_secret",
        input.auth_google_client_secret.as_deref(),
        MAX_AUTH_CLIENT_SECRET_LENGTH,
    )?;
    validate_optional_length("auth_evm_chain_ids", input.auth_evm_chain_ids.as_deref(), MAX_AUTH_CLIENT_ID_LENGTH)?;
    validate_optional_length("auth_evm_statement", input.auth_evm_statement.as_deref(), MAX_AUTH_WALLET_STATEMENT_LENGTH)?;
    Ok(())
}

fn validate_optional_code(field: &str, value: Option<&str>) -> SettingResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.is_empty() || value.len() > 64 {
        return Err(SettingError::InvalidInput(format!("{field} length must be between 1 and 64")));
    }
    if !value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')) {
        return Err(SettingError::InvalidInput(format!("{field} contains invalid characters")));
    }
    Ok(())
}

pub fn validate_recharge_bounds(input: &SystemSettingsUpdate, current: &types::system_setting::SystemSettingsResponse) -> SettingResult<()> {
    let min = input.recharge_min_amount.unwrap_or(current.recharge_min_amount);
    let max = input.recharge_max_amount.unwrap_or(current.recharge_max_amount);
    if min > max {
        return Err(SettingError::InvalidInput(
            "recharge_min_amount must be less than or equal to recharge_max_amount".into(),
        ));
    }
    Ok(())
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_owned())
}

fn normalize_optional_headers(value: Option<String>) -> Option<String> {
    value.map(|item| normalize_sensitive_headers(&item))
}

fn normalize_sensitive_headers(value: &str) -> String {
    value
        .split(',')
        .map(|item| item.trim().to_ascii_lowercase())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join(HEADER_SEPARATOR)
}

fn normalize_email_suffixes(value: &str) -> String {
    value
        .split(',')
        .map(|item| item.trim().to_ascii_lowercase())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join(HEADER_SEPARATOR)
}

fn validate_site_name(value: Option<&str>) -> SettingResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.is_empty() || value.len() > MAX_SITE_NAME_LENGTH {
        return Err(SettingError::InvalidInput(format!(
            "site_name length must be between 1 and {MAX_SITE_NAME_LENGTH}"
        )));
    }
    Ok(())
}

fn validate_site_subtitle(value: Option<&str>) -> SettingResult<()> {
    if value.is_some_and(|text| text.len() > MAX_SITE_SUBTITLE_LENGTH) {
        return Err(SettingError::InvalidInput(format!(
            "site_subtitle length must be at most {MAX_SITE_SUBTITLE_LENGTH}"
        )));
    }
    Ok(())
}

fn validate_public_base_url(value: Option<&str>) -> SettingResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.is_empty() {
        return Ok(());
    }
    if value.len() > MAX_PUBLIC_BASE_URL_LENGTH {
        return Err(SettingError::InvalidInput(format!(
            "public_base_url length must be at most {MAX_PUBLIC_BASE_URL_LENGTH}"
        )));
    }
    let is_valid =
        public_base_url_is_valid(value).map_err(|error| SettingError::Infrastructure(format!("invalid public_base_url validation regex: {error}")))?;
    if !is_valid {
        return Err(SettingError::InvalidInput("public_base_url must be a valid HTTP or HTTPS URL".into()));
    }
    Ok(())
}

fn validate_non_negative_decimal(field: &str, value: Option<Decimal>) -> SettingResult<()> {
    if value.is_some_and(|item| item < Decimal::ZERO) {
        return Err(SettingError::InvalidInput(format!("{field} must be greater than or equal to 0")));
    }
    Ok(())
}

fn validate_non_negative_i64(field: &str, value: Option<i64>) -> SettingResult<()> {
    if value.is_some_and(|item| item < 0) {
        return Err(SettingError::InvalidInput(format!("{field} must be greater than or equal to 0")));
    }
    Ok(())
}

fn validate_positive_i64(field: &str, value: Option<i64>) -> SettingResult<()> {
    if value.is_some_and(|item| item <= 0) {
        return Err(SettingError::InvalidInput(format!("{field} must be greater than 0")));
    }
    Ok(())
}

fn validate_positive_decimal(field: &str, value: Option<Decimal>) -> SettingResult<()> {
    if value.is_some_and(|item| item <= Decimal::ZERO) {
        return Err(SettingError::InvalidInput(format!("{field} must be greater than 0")));
    }
    Ok(())
}

fn validate_recharge_settings(input: &SystemSettingsUpdate) -> SettingResult<()> {
    validate_positive_decimal("recharge_arrival_ratio", input.recharge_arrival_ratio)?;
    validate_positive_i64("recharge_order_expire_minutes", input.recharge_order_expire_minutes)?;
    validate_positive_i64("recharge_max_unpaid_orders", input.recharge_max_unpaid_orders)?;
    validate_positive_decimal("recharge_min_amount", input.recharge_min_amount)?;
    validate_positive_decimal("recharge_max_amount", input.recharge_max_amount)?;
    Ok(())
}

fn validate_sensitive_headers(field: &str, value: Option<&str>) -> SettingResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    for header in value.split(',').map(str::trim).filter(|item| !item.is_empty()) {
        if HeaderName::from_bytes(header.as_bytes()).is_err() {
            return Err(SettingError::InvalidInput(format!("{field} contains invalid header name: {header}")));
        }
    }
    Ok(())
}

fn validate_optional_length(field: &str, value: Option<&str>, max: usize) -> SettingResult<()> {
    if value.is_some_and(|item| item.len() > max) {
        return Err(SettingError::InvalidInput(format!("{field} length must be at most {max}")));
    }
    Ok(())
}

fn validate_required_optional_length(field: &str, value: Option<&str>, max: usize) -> SettingResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.is_empty() || value.len() > max {
        return Err(SettingError::InvalidInput(format!("{field} length must be between 1 and {max}")));
    }
    Ok(())
}

#[cfg(test)]
#[path = "validation_tests.rs"]
mod tests;

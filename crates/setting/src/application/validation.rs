use axum::http::HeaderName;
use rust_decimal::Decimal;
use types::system_setting::{EmailSuffixMode, RequestRecordLevel, SystemSettingsUpdate};

use super::{SettingError, SettingResult};

const MAX_SITE_NAME_LENGTH: usize = 100;
const MAX_SITE_SUBTITLE_LENGTH: usize = 200;
const MAX_SMTP_HOST_LENGTH: usize = 255;
const MAX_SMTP_USERNAME_LENGTH: usize = 255;
const MAX_SMTP_PASSWORD_LENGTH: usize = 1024;
const MAX_SMTP_FROM_EMAIL_LENGTH: usize = 255;
const MAX_SMTP_FROM_NAME_LENGTH: usize = 100;
const MAX_EMAIL_TEMPLATE_SUBJECT_LENGTH: usize = 200;
const MIN_SMTP_PORT: i64 = 1;
const MAX_SMTP_PORT: i64 = 65_535;
const HEADER_SEPARATOR: &str = ", ";

pub fn sanitize_update(input: SystemSettingsUpdate) -> SystemSettingsUpdate {
    let defaults = input.request_record_level.map(request_record_level_defaults);
    SystemSettingsUpdate {
        site_name: input.site_name.map(|value| value.trim().to_owned()),
        site_subtitle: input.site_subtitle.map(|value| value.trim().to_owned()),
        sensitive_request_headers: input.sensitive_request_headers.map(|value| normalize_sensitive_headers(&value)),
        smtp_host: trim_optional(input.smtp_host),
        smtp_username: trim_optional(input.smtp_username),
        smtp_password: trim_optional(input.smtp_password),
        smtp_from_email: trim_optional(input.smtp_from_email),
        smtp_from_name: trim_optional(input.smtp_from_name),
        email_suffixes: input.email_suffixes.map(|value| normalize_email_suffixes(&value)),
        email_template_registration_subject: trim_optional(input.email_template_registration_subject),
        email_template_password_reset_subject: trim_optional(input.email_template_password_reset_subject),
        record_request_headers: input.record_request_headers.or(defaults.map(|value| value.record_request_headers)),
        record_request_body: input.record_request_body.or(defaults.map(|value| value.record_request_body)),
        record_response_body: input.record_response_body.or(defaults.map(|value| value.record_response_body)),
        ..input
    }
}

pub fn validate_update(input: &SystemSettingsUpdate) -> SettingResult<()> {
    if input.is_empty() {
        return Err(SettingError::InvalidInput("update payload is empty".into()));
    }
    validate_site_name(input.site_name.as_deref())?;
    validate_site_subtitle(input.site_subtitle.as_deref())?;
    validate_positive_i64("request_record_retention_days", input.request_record_retention_days)?;
    validate_positive_i64("request_record_payload_retention_days", input.request_record_payload_retention_days)?;
    validate_positive_i64("performance_monitoring_retention_days", input.performance_monitoring_retention_days)?;
    validate_positive_i64("max_request_body_size_kb", input.max_request_body_size_kb)?;
    validate_positive_i64("max_response_body_size_kb", input.max_response_body_size_kb)?;
    validate_sensitive_request_headers(input.sensitive_request_headers.as_deref())?;
    validate_non_negative_decimal("default_user_grant", input.default_user_grant)?;
    validate_non_negative_i64("default_rate_limit_rpm", input.default_rate_limit_rpm)?;
    validate_mail_settings(input)
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_owned())
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

fn request_record_level_defaults(level: RequestRecordLevel) -> RequestRecordSwitchDefaults {
    match level {
        RequestRecordLevel::Basic => RequestRecordSwitchDefaults::new(false, false, false),
        RequestRecordLevel::Headers => RequestRecordSwitchDefaults::new(true, false, false),
        RequestRecordLevel::Full => RequestRecordSwitchDefaults::new(true, true, true),
    }
}

#[derive(Clone, Copy)]
struct RequestRecordSwitchDefaults {
    record_request_headers: bool,
    record_request_body: bool,
    record_response_body: bool,
}

impl RequestRecordSwitchDefaults {
    const fn new(record_request_headers: bool, record_request_body: bool, record_response_body: bool) -> Self {
        Self {
            record_request_headers,
            record_request_body,
            record_response_body,
        }
    }
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

fn validate_sensitive_request_headers(value: Option<&str>) -> SettingResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    for header in value.split(',').map(str::trim).filter(|item| !item.is_empty()) {
        if HeaderName::from_bytes(header.as_bytes()).is_err() {
            return Err(SettingError::InvalidInput(format!(
                "sensitive_request_headers contains invalid header name: {header}"
            )));
        }
    }
    Ok(())
}

fn validate_mail_settings(input: &SystemSettingsUpdate) -> SettingResult<()> {
    validate_optional_length("smtp_host", input.smtp_host.as_deref(), MAX_SMTP_HOST_LENGTH)?;
    validate_optional_length("smtp_username", input.smtp_username.as_deref(), MAX_SMTP_USERNAME_LENGTH)?;
    validate_required_optional_length("smtp_password", input.smtp_password.as_deref(), MAX_SMTP_PASSWORD_LENGTH)?;
    validate_optional_length("smtp_from_email", input.smtp_from_email.as_deref(), MAX_SMTP_FROM_EMAIL_LENGTH)?;
    validate_optional_length("smtp_from_name", input.smtp_from_name.as_deref(), MAX_SMTP_FROM_NAME_LENGTH)?;
    validate_smtp_port(input.smtp_port)?;
    validate_email_address("smtp_from_email", input.smtp_from_email.as_deref())?;
    validate_email_suffixes(input.email_suffix_mode, input.email_suffixes.as_deref())?;
    validate_template(
        "email_template_registration",
        input.email_template_registration_subject.as_deref(),
        input.email_template_registration_html.as_deref(),
    )?;
    validate_template(
        "email_template_password_reset",
        input.email_template_password_reset_subject.as_deref(),
        input.email_template_password_reset_html.as_deref(),
    )
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

fn validate_smtp_port(value: Option<i64>) -> SettingResult<()> {
    if value.is_some_and(|port| !(MIN_SMTP_PORT..=MAX_SMTP_PORT).contains(&port)) {
        return Err(SettingError::InvalidInput(format!(
            "smtp_port must be between {MIN_SMTP_PORT} and {MAX_SMTP_PORT}"
        )));
    }
    Ok(())
}

fn validate_email_address(field: &str, value: Option<&str>) -> SettingResult<()> {
    let Some(value) = value.filter(|item| !item.is_empty()) else {
        return Ok(());
    };
    let Some((local, domain)) = value.split_once('@') else {
        return Err(invalid_email(field));
    };
    if local.is_empty() || domain.is_empty() || !domain.contains('.') || value.matches('@').count() != 1 {
        return Err(invalid_email(field));
    }
    Ok(())
}

fn invalid_email(field: &str) -> SettingError {
    SettingError::InvalidInput(format!("{field} must be a valid email address"))
}

fn validate_email_suffixes(mode: Option<EmailSuffixMode>, value: Option<&str>) -> SettingResult<()> {
    if mode.is_some_and(|item| item != EmailSuffixMode::None) && value.is_some_and(str::is_empty) {
        return Err(SettingError::InvalidInput(
            "email_suffixes cannot be empty when suffix restriction is enabled".into(),
        ));
    }
    let Some(value) = value else {
        return Ok(());
    };
    for suffix in value.split(',').map(str::trim).filter(|item| !item.is_empty()) {
        if suffix.contains('@') || !suffix.contains('.') {
            return Err(SettingError::InvalidInput(format!("email_suffixes contains invalid suffix: {suffix}")));
        }
    }
    Ok(())
}

fn validate_template(field: &str, subject: Option<&str>, html: Option<&str>) -> SettingResult<()> {
    validate_required_optional_length(&format!("{field}_subject"), subject, MAX_EMAIL_TEMPLATE_SUBJECT_LENGTH)?;
    if html.is_some_and(str::is_empty) {
        return Err(SettingError::InvalidInput(format!("{field}_html cannot be empty")));
    }
    Ok(())
}

#[cfg(test)]
#[path = "validation_tests.rs"]
mod tests;

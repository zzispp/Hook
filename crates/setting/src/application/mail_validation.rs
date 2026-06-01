use types::system_setting::{EmailSuffixMode, SystemSettingsUpdate};

use super::{SettingError, SettingResult};

const MAX_SMTP_HOST_LENGTH: usize = 255;
const MAX_SMTP_USERNAME_LENGTH: usize = 255;
const MAX_SMTP_PASSWORD_LENGTH: usize = 1024;
const MAX_SMTP_FROM_EMAIL_LENGTH: usize = 255;
const MAX_SMTP_FROM_NAME_LENGTH: usize = 100;
const MAX_EMAIL_TEMPLATE_SUBJECT_LENGTH: usize = 200;
const MIN_SMTP_PORT: i64 = 1;
const MAX_SMTP_PORT: i64 = 65_535;

pub(super) fn validate_mail_settings(input: &SystemSettingsUpdate) -> SettingResult<()> {
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

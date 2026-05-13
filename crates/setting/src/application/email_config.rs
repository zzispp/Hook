use types::system_setting::{SystemSettingsResponse, SystemSettingsUpdate};

use super::{SettingError, SettingResult};

const MIN_SMTP_PORT: i64 = 1;

pub fn validate_email_verification_prerequisites(input: &SystemSettingsUpdate, current: &SystemSettingsResponse) -> SettingResult<()> {
    let state = effective_email_verification_state(input, current);
    if !state.registration_email_verification_enabled || state.email_config_enabled && state.smtp_config_complete {
        return Ok(());
    }
    Err(SettingError::InvalidInput(
        "registration_email_verification_enabled requires email_config_enabled and complete SMTP configuration".into(),
    ))
}

struct EmailVerificationState {
    email_config_enabled: bool,
    registration_email_verification_enabled: bool,
    smtp_config_complete: bool,
}

fn effective_email_verification_state(input: &SystemSettingsUpdate, current: &SystemSettingsResponse) -> EmailVerificationState {
    EmailVerificationState {
        email_config_enabled: input.email_config_enabled.unwrap_or(current.email_config_enabled),
        registration_email_verification_enabled: input
            .registration_email_verification_enabled
            .unwrap_or(current.registration_email_verification_enabled),
        smtp_config_complete: effective_smtp_config_complete(input, current),
    }
}

fn effective_smtp_config_complete(input: &SystemSettingsUpdate, current: &SystemSettingsResponse) -> bool {
    !effective_text(&input.smtp_host, &current.smtp_host).is_empty()
        && effective_smtp_port(input, current) >= MIN_SMTP_PORT
        && !effective_text(&input.smtp_username, &current.smtp_username).is_empty()
        && effective_smtp_password_set(input, current)
        && !effective_text(&input.smtp_from_email, &current.smtp_from_email).is_empty()
}

fn effective_text<'a>(input: &'a Option<String>, current: &'a str) -> &'a str {
    input.as_deref().unwrap_or(current)
}

fn effective_smtp_port(input: &SystemSettingsUpdate, current: &SystemSettingsResponse) -> i64 {
    input.smtp_port.unwrap_or(current.smtp_port)
}

fn effective_smtp_password_set(input: &SystemSettingsUpdate, current: &SystemSettingsResponse) -> bool {
    match input.smtp_password.as_deref() {
        Some(password) => !password.is_empty(),
        None => current.smtp_password_set,
    }
}

use rust_decimal::Decimal;
use types::system_setting::SystemSettingsUpdate;

use super::{SettingError, SettingResult};

const MAX_SITE_NAME_LENGTH: usize = 100;
const MAX_SITE_SUBTITLE_LENGTH: usize = 200;

pub fn sanitize_update(input: SystemSettingsUpdate) -> SystemSettingsUpdate {
    SystemSettingsUpdate {
        site_name: input.site_name.map(|value| value.trim().to_owned()),
        site_subtitle: input.site_subtitle.map(|value| value.trim().to_owned()),
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
    validate_non_negative_decimal("default_user_grant", input.default_user_grant)?;
    validate_non_negative_i64("default_rate_limit_rpm", input.default_rate_limit_rpm)
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

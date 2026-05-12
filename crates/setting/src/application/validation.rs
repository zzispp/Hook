use axum::http::HeaderName;
use rust_decimal::Decimal;
use types::system_setting::SystemSettingsUpdate;

use super::{SettingError, SettingResult};

const MAX_SITE_NAME_LENGTH: usize = 100;
const MAX_SITE_SUBTITLE_LENGTH: usize = 200;
const HEADER_SEPARATOR: &str = ", ";

pub fn sanitize_update(input: SystemSettingsUpdate) -> SystemSettingsUpdate {
    SystemSettingsUpdate {
        site_name: input.site_name.map(|value| value.trim().to_owned()),
        site_subtitle: input.site_subtitle.map(|value| value.trim().to_owned()),
        sensitive_request_headers: input.sensitive_request_headers.map(|value| normalize_sensitive_headers(&value)),
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
    validate_positive_i64("max_request_body_size_kb", input.max_request_body_size_kb)?;
    validate_positive_i64("max_response_body_size_kb", input.max_response_body_size_kb)?;
    validate_sensitive_request_headers(input.sensitive_request_headers.as_deref())?;
    validate_non_negative_decimal("default_user_grant", input.default_user_grant)?;
    validate_non_negative_i64("default_rate_limit_rpm", input.default_rate_limit_rpm)
}

fn normalize_sensitive_headers(value: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_update_normalizes_sensitive_request_headers() {
        let input = SystemSettingsUpdate {
            sensitive_request_headers: Some(" Authorization, X-API-Key , cookie ".into()),
            ..Default::default()
        };

        let sanitized = sanitize_update(input);

        assert_eq!(sanitized.sensitive_request_headers.as_deref(), Some("authorization, x-api-key, cookie"));
    }

    #[test]
    fn validate_update_rejects_invalid_sensitive_request_header() {
        let input = SystemSettingsUpdate {
            sensitive_request_headers: Some("authorization, bad header".into()),
            ..Default::default()
        };

        let error = validate_update(&input).unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid input: sensitive_request_headers contains invalid header name: bad header"
        );
    }

    #[test]
    fn validate_update_rejects_non_positive_request_record_body_limits() {
        let input = SystemSettingsUpdate {
            max_request_body_size_kb: Some(0),
            max_response_body_size_kb: Some(-1),
            ..Default::default()
        };

        let error = validate_update(&input).unwrap_err();

        assert_eq!(error.to_string(), "invalid input: max_request_body_size_kb must be greater than 0");
    }
}

use types::system_setting::{RequestRecordLevel, SystemSettingsUpdate};

use super::{sanitize_update, validate_update};

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
fn sanitize_update_applies_request_record_level_defaults_without_overriding_explicit_switches() {
    let input = SystemSettingsUpdate {
        request_record_level: Some(RequestRecordLevel::Full),
        record_response_body: Some(false),
        ..Default::default()
    };

    let sanitized = sanitize_update(input);

    assert_eq!(sanitized.record_request_headers, Some(true));
    assert_eq!(sanitized.record_request_body, Some(true));
    assert_eq!(sanitized.record_response_body, Some(false));
}

#[test]
fn sanitize_update_normalizes_email_suffixes() {
    let input = SystemSettingsUpdate {
        email_suffixes: Some(" Gmail.COM, outlook.com , ".into()),
        ..Default::default()
    };

    let sanitized = sanitize_update(input);

    assert_eq!(sanitized.email_suffixes.as_deref(), Some("gmail.com, outlook.com"));
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

#[test]
fn validate_update_rejects_non_positive_performance_monitoring_retention_days() {
    let input = SystemSettingsUpdate {
        performance_monitoring_retention_days: Some(0),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(error.to_string(), "invalid input: performance_monitoring_retention_days must be greater than 0");
}

#[test]
fn validate_update_rejects_invalid_smtp_port() {
    let input = SystemSettingsUpdate {
        smtp_port: Some(70_000),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(error.to_string(), "invalid input: smtp_port must be between 1 and 65535");
}

#[test]
fn validate_update_rejects_empty_template_html() {
    let input = SystemSettingsUpdate {
        email_template_registration_html: Some(String::new()),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(error.to_string(), "invalid input: email_template_registration_html cannot be empty");
}

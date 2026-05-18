use types::{
    provider::{ProviderCooldownPolicy, ProviderCooldownRule},
    system_setting::SystemSettingsUpdate,
};

use super::{sanitize_update, validate_update};

#[test]
fn sanitize_update_normalizes_client_sensitive_request_headers() {
    let input = SystemSettingsUpdate {
        client_sensitive_request_headers: Some(" Authorization, X-API-Key , cookie ".into()),
        ..Default::default()
    };

    let sanitized = sanitize_update(input);

    assert_eq!(sanitized.client_sensitive_request_headers.as_deref(), Some("authorization, x-api-key, cookie"));
}

#[test]
fn sanitize_update_normalizes_provider_sensitive_request_headers() {
    let input = SystemSettingsUpdate {
        provider_sensitive_request_headers: Some(" X-Provider-Key, Authorization ".into()),
        ..Default::default()
    };

    let sanitized = sanitize_update(input);

    assert_eq!(sanitized.provider_sensitive_request_headers.as_deref(), Some("x-provider-key, authorization"));
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
        client_sensitive_request_headers: Some("authorization, bad header".into()),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(
        error.to_string(),
        "invalid input: client_sensitive_request_headers contains invalid header name: bad header"
    );
}

#[test]
fn validate_update_rejects_non_positive_request_record_body_limits() {
    let input = SystemSettingsUpdate {
        client_max_request_body_size_kb: Some(0),
        provider_max_response_body_size_kb: Some(-1),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(error.to_string(), "invalid input: client_max_request_body_size_kb must be greater than 0");
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
fn validate_update_rejects_non_positive_request_record_cleanup_interval_hours() {
    let input = SystemSettingsUpdate {
        request_record_cleanup_interval_hours: Some(0),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(error.to_string(), "invalid input: request_record_cleanup_interval_hours must be greater than 0");
}

#[test]
fn validate_update_rejects_non_positive_performance_monitoring_cleanup_interval_hours() {
    let input = SystemSettingsUpdate {
        performance_monitoring_cleanup_interval_hours: Some(0),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(
        error.to_string(),
        "invalid input: performance_monitoring_cleanup_interval_hours must be greater than 0"
    );
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

#[test]
fn validate_update_accepts_empty_provider_cooldown_rules() {
    let input = SystemSettingsUpdate {
        provider_cooldown_policy: Some(ProviderCooldownPolicy {
            window_seconds: 0,
            rules: Vec::new(),
        }),
        ..Default::default()
    };

    assert!(validate_update(&input).is_ok());
}

#[test]
fn validate_update_rejects_invalid_provider_cooldown_status_code() {
    let input = SystemSettingsUpdate {
        provider_cooldown_policy: Some(policy_with_rule(99, 2, 60)),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(
        error.to_string(),
        "invalid input: provider_cooldown_policy.status_code must be between 100 and 599"
    );
}

#[test]
fn validate_update_rejects_duplicate_provider_cooldown_status_code() {
    let input = SystemSettingsUpdate {
        provider_cooldown_policy: Some(ProviderCooldownPolicy {
            window_seconds: 60,
            rules: vec![cooldown_rule(429, 2, 60), cooldown_rule(429, 3, 120)],
        }),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(error.to_string(), "invalid input: provider_cooldown_policy contains duplicate status_code: 429");
}

#[test]
fn validate_update_rejects_non_positive_provider_cooldown_values() {
    let input = SystemSettingsUpdate {
        provider_cooldown_policy: Some(policy_with_rule(429, 0, 60)),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(
        error.to_string(),
        "invalid input: provider_cooldown_policy.failure_count must be greater than 0"
    );
}

fn policy_with_rule(status_code: i32, failure_count: i64, cooldown_seconds: i64) -> ProviderCooldownPolicy {
    ProviderCooldownPolicy {
        window_seconds: 60,
        rules: vec![cooldown_rule(status_code, failure_count, cooldown_seconds)],
    }
}

fn cooldown_rule(status_code: i32, failure_count: i64, cooldown_seconds: i64) -> ProviderCooldownRule {
    ProviderCooldownRule {
        status_code,
        failure_count,
        cooldown_seconds,
    }
}

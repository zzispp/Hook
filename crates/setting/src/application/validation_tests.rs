use rust_decimal::Decimal;
use types::{
    provider::{ProviderCooldownPolicy, ProviderCooldownRule},
    system_setting::{EmailSuffixMode, RequestRecordLevel, SmtpEncryption, SystemSettingsResponse, SystemSettingsUpdate},
};

use super::{sanitize_update, validate_recharge_bounds, validate_update};

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
fn sanitize_update_trims_site_logo_base64() {
    let input = SystemSettingsUpdate {
        site_logo_base64: Some("  data:image/png;base64,AA==  ".into()),
        ..Default::default()
    };

    let sanitized = sanitize_update(input);

    assert_eq!(sanitized.site_logo_base64.as_deref(), Some("data:image/png;base64,AA=="));
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
fn validate_update_rejects_non_positive_cache_affinity_ttl_minutes() {
    let input = SystemSettingsUpdate {
        cache_affinity_ttl_minutes: Some(0),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(error.to_string(), "invalid input: cache_affinity_ttl_minutes must be greater than 0");
}

#[test]
fn validate_update_rejects_non_positive_recharge_values() {
    let input = SystemSettingsUpdate {
        recharge_arrival_ratio: Some(Decimal::ZERO),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(error.to_string(), "invalid input: recharge_arrival_ratio must be greater than 0");
}

#[test]
fn validate_recharge_bounds_rejects_min_greater_than_max() {
    let input = SystemSettingsUpdate {
        recharge_min_amount: Some(Decimal::new(4000, 0)),
        ..Default::default()
    };

    let error = validate_recharge_bounds(&input, &system_settings_response()).unwrap_err();

    assert_eq!(
        error.to_string(),
        "invalid input: recharge_min_amount must be less than or equal to recharge_max_amount"
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

fn system_settings_response() -> SystemSettingsResponse {
    SystemSettingsResponse {
        site_name: "Hook".into(),
        site_subtitle: String::new(),
        site_logo_base64: String::new(),
        allow_registration: true,
        login_captcha_enabled: false,
        registration_captcha_enabled: false,
        support_ticket_captcha_enabled: true,
        registration_email_verification_enabled: false,
        password_reset_enabled: false,
        email_config_enabled: false,
        support_ticket_email_notifications_enabled: false,
        token_limit_per_user: 5,
        client_request_record_level: RequestRecordLevel::Basic,
        client_record_request_headers: true,
        client_record_request_body: true,
        client_record_response_headers: true,
        client_record_response_body: true,
        client_max_request_body_size_kb: 5120,
        client_max_response_body_size_kb: 5120,
        client_sensitive_request_headers: String::new(),
        provider_request_record_level: RequestRecordLevel::Basic,
        provider_record_request_headers: true,
        provider_record_request_body: true,
        provider_record_response_headers: true,
        provider_record_response_body: true,
        provider_max_request_body_size_kb: 5120,
        provider_max_response_body_size_kb: 5120,
        provider_sensitive_request_headers: String::new(),
        default_user_grant: Decimal::ZERO,
        default_rate_limit_rpm: 0,
        recharge_enabled: false,
        recharge_arrival_ratio: Decimal::ONE,
        recharge_order_expire_minutes: 15,
        recharge_min_amount: Decimal::new(1, 2),
        recharge_max_amount: Decimal::new(3000, 0),
        scheduling_mode: types::provider::ProviderSchedulingMode::CacheAffinity,
        cache_affinity_ttl_minutes: 5,
        provider_cooldown_policy: ProviderCooldownPolicy::default(),
        smtp_host: String::new(),
        smtp_port: 587,
        smtp_username: String::new(),
        smtp_password_set: false,
        smtp_from_email: String::new(),
        smtp_from_name: "Hook".into(),
        smtp_encryption: SmtpEncryption::Tls,
        email_suffix_mode: EmailSuffixMode::None,
        email_suffixes: String::new(),
        email_template_registration_subject: "注册验证码".into(),
        email_template_registration_html: "<p>{{code}}</p>".into(),
        email_template_password_reset_subject: "找回密码".into(),
        email_template_password_reset_html: "<p>{{reset_link}}</p>".into(),
        created_at: "2026-05-25T00:00:00Z".into(),
        updated_at: "2026-05-25T00:00:00Z".into(),
    }
}

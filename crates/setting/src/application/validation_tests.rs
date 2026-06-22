use rust_decimal::Decimal;
use types::{
    provider::ProviderCooldownPolicy,
    system_setting::{ApiEndpoint, EmailSuffixMode, RequestRecordLevel, SmtpEncryption, SystemSettingsResponse, SystemSettingsUpdate},
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
fn sanitize_update_trims_api_endpoint_fields() {
    let input = SystemSettingsUpdate {
        api_endpoints: Some(vec![api_endpoint(
            "  endpoint-1  ",
            "  Global  ",
            "  https://api.example.com/  ",
            "  public network  ",
        )]),
        ..Default::default()
    };

    let sanitized = sanitize_update(input);

    assert_eq!(
        sanitized.api_endpoints,
        Some(vec![api_endpoint("endpoint-1", "Global", "https://api.example.com", "public network",)])
    );
}

#[test]
fn validate_update_accepts_empty_api_endpoint_list() {
    validate_update(&SystemSettingsUpdate {
        api_endpoints: Some(Vec::new()),
        ..Default::default()
    })
    .unwrap();
}

#[test]
fn validate_update_rejects_api_endpoint_without_url() {
    let input = SystemSettingsUpdate {
        api_endpoints: Some(vec![api_endpoint("endpoint-1", "Global", "", "")]),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(error.to_string(), "invalid input: api_endpoints.url length must be between 1 and 255");
}

#[test]
fn validate_update_rejects_invalid_api_endpoint_url() {
    let input = SystemSettingsUpdate {
        api_endpoints: Some(vec![api_endpoint("endpoint-1", "Global", "https://", "")]),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(error.to_string(), "invalid input: api_endpoints.url must be a valid HTTP or HTTPS URL");
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
fn validate_update_rejects_non_positive_recharge_max_unpaid_orders() {
    let input = SystemSettingsUpdate {
        recharge_max_unpaid_orders: Some(0),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(error.to_string(), "invalid input: recharge_max_unpaid_orders must be greater than 0");
}

#[test]
fn validate_update_rejects_affiliate_commission_percent_above_100() {
    let input = SystemSettingsUpdate {
        affiliate_commission_percent: Some(Decimal::new(10001, 2)),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(error.to_string(), "invalid input: affiliate_commission_percent must be between 0 and 100");
}

#[test]
fn validate_update_accepts_affiliate_commission_percent_boundaries() {
    validate_update(&SystemSettingsUpdate {
        affiliate_commission_percent: Some(Decimal::ZERO),
        ..Default::default()
    })
    .unwrap();
    validate_update(&SystemSettingsUpdate {
        affiliate_commission_percent: Some(Decimal::new(100, 0)),
        ..Default::default()
    })
    .unwrap();
}

#[test]
fn validate_update_rejects_negative_affiliate_min_commission_amount() {
    let input = SystemSettingsUpdate {
        affiliate_min_commission_amount: Some(Decimal::new(-1, 0)),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(
        error.to_string(),
        "invalid input: affiliate_min_commission_amount must be greater than or equal to 0"
    );
}

#[test]
fn validate_update_rejects_invalid_public_base_url() {
    let input = SystemSettingsUpdate {
        public_base_url: Some("https://".into()),
        ..Default::default()
    };

    let error = validate_update(&input).unwrap_err();

    assert_eq!(error.to_string(), "invalid input: public_base_url must be a valid HTTP or HTTPS URL");
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

fn system_settings_response() -> SystemSettingsResponse {
    SystemSettingsResponse {
        site_name: "Hook".into(),
        site_subtitle: String::new(),
        public_base_url: "https://hook.test".into(),
        site_logo_base64: String::new(),
        contact_methods: Vec::new(),
        api_endpoints: Vec::new(),
        allow_registration: true,
        login_captcha_enabled: false,
        registration_captcha_enabled: false,
        support_ticket_captcha_enabled: true,
        recharge_captcha_enabled: false,
        registration_email_verification_enabled: false,
        auth_github_enabled: false,
        auth_github_client_id: String::new(),
        auth_github_client_secret_set: false,
        auth_google_enabled: false,
        auth_google_client_id: String::new(),
        auth_google_client_secret_set: false,
        auth_evm_enabled: false,
        auth_evm_chain_ids: "1".into(),
        auth_evm_statement: "Sign in to Hook".into(),
        password_reset_enabled: false,
        email_config_enabled: false,
        support_ticket_email_notifications_enabled: false,
        default_user_group_code: "default".into(),
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
        recharge_max_unpaid_orders: 5,
        recharge_min_amount: Decimal::new(1, 2),
        recharge_max_amount: Decimal::new(3000, 0),
        affiliate_enabled: false,
        affiliate_commission_percent: Decimal::ZERO,
        affiliate_min_commission_amount: Decimal::ZERO,
        scheduling_mode: types::provider::ProviderSchedulingMode::CacheAffinity,
        provider_priority_mode: types::provider::ProviderPriorityMode::Provider,
        key_priority_snapshot_initialized: false,
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

fn api_endpoint(id: &str, name: &str, url: &str, description: &str) -> ApiEndpoint {
    ApiEndpoint {
        id: id.into(),
        name: name.into(),
        url: url.into(),
        description: description.into(),
    }
}

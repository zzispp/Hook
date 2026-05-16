use rust_decimal::Decimal;
use serde::Serialize;

use crate::provider::ProviderSchedulingMode;

use super::{DisplayCurrency, EmailSuffixMode, RequestRecordLevel, SmtpEncryption, SystemSettings};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SystemSettingsResponse {
    pub site_name: String,
    pub site_subtitle: String,
    pub allow_registration: bool,
    pub login_captcha_enabled: bool,
    pub registration_captcha_enabled: bool,
    pub registration_email_verification_enabled: bool,
    pub email_config_enabled: bool,
    pub support_ticket_email_notifications_enabled: bool,
    pub auto_delete_expired_tokens: bool,
    pub request_record_retention_days: i64,
    pub request_record_payload_retention_days: i64,
    pub performance_monitoring_retention_days: i64,
    pub request_record_level: RequestRecordLevel,
    pub max_request_body_size_kb: i64,
    pub max_response_body_size_kb: i64,
    pub sensitive_request_headers: String,
    pub record_request_headers: bool,
    pub record_request_body: bool,
    pub record_response_body: bool,
    #[serde(with = "rust_decimal::serde::float")]
    pub default_user_grant: Decimal,
    pub default_rate_limit_rpm: i64,
    pub scheduling_mode: ProviderSchedulingMode,
    pub currency: DisplayCurrency,
    pub smtp_host: String,
    pub smtp_port: i64,
    pub smtp_username: String,
    pub smtp_password_set: bool,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub smtp_encryption: SmtpEncryption,
    pub email_suffix_mode: EmailSuffixMode,
    pub email_suffixes: String,
    pub email_template_registration_subject: String,
    pub email_template_registration_html: String,
    pub email_template_password_reset_subject: String,
    pub email_template_password_reset_html: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExchangeRateResponse {
    pub base: String,
    pub target: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub rate: Decimal,
    pub source: String,
    pub source_date: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CurrencyDisplayResponse {
    pub currency: DisplayCurrency,
    pub usd_cny_rate: Option<ExchangeRateResponse>,
}

impl From<SystemSettings> for SystemSettingsResponse {
    fn from(value: SystemSettings) -> Self {
        Self {
            site_name: value.site_name,
            site_subtitle: value.site_subtitle,
            allow_registration: value.allow_registration,
            login_captcha_enabled: value.login_captcha_enabled,
            registration_captcha_enabled: value.registration_captcha_enabled,
            registration_email_verification_enabled: value.registration_email_verification_enabled,
            email_config_enabled: value.email_config_enabled,
            support_ticket_email_notifications_enabled: value.support_ticket_email_notifications_enabled,
            auto_delete_expired_tokens: value.auto_delete_expired_tokens,
            request_record_retention_days: value.request_record_retention_days,
            request_record_payload_retention_days: value.request_record_payload_retention_days,
            performance_monitoring_retention_days: value.performance_monitoring_retention_days,
            request_record_level: value.request_record_level,
            max_request_body_size_kb: value.max_request_body_size_kb,
            max_response_body_size_kb: value.max_response_body_size_kb,
            sensitive_request_headers: value.sensitive_request_headers,
            record_request_headers: value.record_request_headers,
            record_request_body: value.record_request_body,
            record_response_body: value.record_response_body,
            default_user_grant: value.default_user_grant,
            default_rate_limit_rpm: value.default_rate_limit_rpm,
            scheduling_mode: value.scheduling_mode,
            currency: value.currency,
            smtp_host: value.smtp_host,
            smtp_port: value.smtp_port,
            smtp_username: value.smtp_username,
            smtp_password_set: value.smtp_password_set,
            smtp_from_email: value.smtp_from_email,
            smtp_from_name: value.smtp_from_name,
            smtp_encryption: value.smtp_encryption,
            email_suffix_mode: value.email_suffix_mode,
            email_suffixes: value.email_suffixes,
            email_template_registration_subject: value.email_template_registration_subject,
            email_template_registration_html: value.email_template_registration_html,
            email_template_password_reset_subject: value.email_template_password_reset_subject,
            email_template_password_reset_html: value.email_template_password_reset_html,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

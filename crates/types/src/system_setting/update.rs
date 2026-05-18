use rust_decimal::Decimal;
use serde::Deserialize;

use crate::provider::{ProviderCooldownPolicy, ProviderSchedulingMode};

use super::{EmailSuffixMode, RequestRecordLevel, SmtpEncryption};

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct SystemSettingsUpdate {
    #[serde(default)]
    pub site_name: Option<String>,
    #[serde(default)]
    pub site_subtitle: Option<String>,
    #[serde(default)]
    pub allow_registration: Option<bool>,
    #[serde(default)]
    pub login_captcha_enabled: Option<bool>,
    #[serde(default)]
    pub registration_captcha_enabled: Option<bool>,
    #[serde(default)]
    pub registration_email_verification_enabled: Option<bool>,
    #[serde(default)]
    pub email_config_enabled: Option<bool>,
    #[serde(default)]
    pub support_ticket_email_notifications_enabled: Option<bool>,
    #[serde(default)]
    pub auto_delete_expired_tokens: Option<bool>,
    #[serde(default)]
    pub request_record_cleanup_enabled: Option<bool>,
    #[serde(default)]
    pub request_record_cleanup_interval_hours: Option<i64>,
    #[serde(default)]
    pub performance_monitoring_cleanup_enabled: Option<bool>,
    #[serde(default)]
    pub performance_monitoring_cleanup_interval_hours: Option<i64>,
    #[serde(default)]
    pub request_record_retention_days: Option<i64>,
    #[serde(default)]
    pub request_record_payload_retention_days: Option<i64>,
    #[serde(default)]
    pub performance_monitoring_retention_days: Option<i64>,
    #[serde(default)]
    pub client_request_record_level: Option<RequestRecordLevel>,
    #[serde(default)]
    pub client_max_request_body_size_kb: Option<i64>,
    #[serde(default)]
    pub client_max_response_body_size_kb: Option<i64>,
    #[serde(default)]
    pub client_sensitive_request_headers: Option<String>,
    #[serde(default)]
    pub provider_request_record_level: Option<RequestRecordLevel>,
    #[serde(default)]
    pub provider_max_request_body_size_kb: Option<i64>,
    #[serde(default)]
    pub provider_max_response_body_size_kb: Option<i64>,
    #[serde(default)]
    pub provider_sensitive_request_headers: Option<String>,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub default_user_grant: Option<Decimal>,
    #[serde(default)]
    pub default_rate_limit_rpm: Option<i64>,
    #[serde(default)]
    pub scheduling_mode: Option<ProviderSchedulingMode>,
    #[serde(default)]
    pub provider_cooldown_policy: Option<ProviderCooldownPolicy>,
    pub smtp_host: Option<String>,
    #[serde(default)]
    pub smtp_port: Option<i64>,
    #[serde(default)]
    pub smtp_username: Option<String>,
    #[serde(default)]
    pub smtp_password: Option<String>,
    #[serde(default)]
    pub smtp_from_email: Option<String>,
    #[serde(default)]
    pub smtp_from_name: Option<String>,
    #[serde(default)]
    pub smtp_encryption: Option<SmtpEncryption>,
    #[serde(default)]
    pub email_suffix_mode: Option<EmailSuffixMode>,
    #[serde(default)]
    pub email_suffixes: Option<String>,
    #[serde(default)]
    pub email_template_registration_subject: Option<String>,
    #[serde(default)]
    pub email_template_registration_html: Option<String>,
    #[serde(default)]
    pub email_template_password_reset_subject: Option<String>,
    #[serde(default)]
    pub email_template_password_reset_html: Option<String>,
}

impl SystemSettingsUpdate {
    pub fn is_empty(&self) -> bool {
        self.general_fields_empty() && self.request_record_fields_empty() && self.mail_fields_empty()
    }

    fn general_fields_empty(&self) -> bool {
        self.site_name.is_none()
            && self.site_subtitle.is_none()
            && self.allow_registration.is_none()
            && self.login_captcha_enabled.is_none()
            && self.registration_captcha_enabled.is_none()
            && self.registration_email_verification_enabled.is_none()
            && self.default_user_grant.is_none()
            && self.default_rate_limit_rpm.is_none()
            && self.scheduling_mode.is_none()
            && self.provider_cooldown_policy.is_none()
            && self.auto_delete_expired_tokens.is_none()
            && self.request_record_cleanup_enabled.is_none()
            && self.request_record_cleanup_interval_hours.is_none()
            && self.performance_monitoring_cleanup_enabled.is_none()
            && self.performance_monitoring_cleanup_interval_hours.is_none()
    }

    fn request_record_fields_empty(&self) -> bool {
        self.request_record_retention_days.is_none()
            && self.request_record_payload_retention_days.is_none()
            && self.performance_monitoring_retention_days.is_none()
            && self.client_request_record_level.is_none()
            && self.client_max_request_body_size_kb.is_none()
            && self.client_max_response_body_size_kb.is_none()
            && self.client_sensitive_request_headers.is_none()
            && self.provider_request_record_level.is_none()
            && self.provider_max_request_body_size_kb.is_none()
            && self.provider_max_response_body_size_kb.is_none()
            && self.provider_sensitive_request_headers.is_none()
    }

    fn mail_fields_empty(&self) -> bool {
        self.smtp_host.is_none()
            && self.email_config_enabled.is_none()
            && self.support_ticket_email_notifications_enabled.is_none()
            && self.smtp_port.is_none()
            && self.smtp_username.is_none()
            && self.smtp_password.is_none()
            && self.smtp_from_email.is_none()
            && self.smtp_from_name.is_none()
            && self.smtp_encryption.is_none()
            && self.email_suffix_mode.is_none()
            && self.email_suffixes.is_none()
            && self.email_template_registration_subject.is_none()
            && self.email_template_registration_html.is_none()
            && self.email_template_password_reset_subject.is_none()
            && self.email_template_password_reset_html.is_none()
    }
}

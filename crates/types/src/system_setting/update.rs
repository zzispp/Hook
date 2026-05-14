use rust_decimal::Decimal;
use serde::Deserialize;

use crate::provider::ProviderSchedulingMode;

use super::{DisplayCurrency, EmailSuffixMode, RequestRecordLevel, SmtpEncryption};

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
    pub request_record_retention_days: Option<i64>,
    #[serde(default)]
    pub request_record_payload_retention_days: Option<i64>,
    #[serde(default)]
    pub request_record_level: Option<RequestRecordLevel>,
    #[serde(default)]
    pub max_request_body_size_kb: Option<i64>,
    #[serde(default)]
    pub max_response_body_size_kb: Option<i64>,
    #[serde(default)]
    pub sensitive_request_headers: Option<String>,
    #[serde(default)]
    pub record_request_headers: Option<bool>,
    #[serde(default)]
    pub record_request_body: Option<bool>,
    #[serde(default)]
    pub record_response_body: Option<bool>,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub default_user_grant: Option<Decimal>,
    #[serde(default)]
    pub default_rate_limit_rpm: Option<i64>,
    #[serde(default)]
    pub scheduling_mode: Option<ProviderSchedulingMode>,
    #[serde(default)]
    pub currency: Option<DisplayCurrency>,
    #[serde(default)]
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
            && self.currency.is_none()
            && self.auto_delete_expired_tokens.is_none()
    }

    fn request_record_fields_empty(&self) -> bool {
        self.request_record_retention_days.is_none()
            && self.request_record_payload_retention_days.is_none()
            && self.request_record_level.is_none()
            && self.max_request_body_size_kb.is_none()
            && self.max_response_body_size_kb.is_none()
            && self.sensitive_request_headers.is_none()
            && self.record_request_headers.is_none()
            && self.record_request_body.is_none()
            && self.record_response_body.is_none()
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

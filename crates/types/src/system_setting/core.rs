use rust_decimal::Decimal;

use crate::provider::ProviderSchedulingMode;

use super::{DisplayCurrency, EmailSuffixMode, RequestRecordLevel, SmtpEncryption};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemSettings {
    pub site_name: String,
    pub site_subtitle: String,
    pub allow_registration: bool,
    pub login_captcha_enabled: bool,
    pub registration_captcha_enabled: bool,
    pub registration_email_verification_enabled: bool,
    pub email_config_enabled: bool,
    pub auto_delete_expired_tokens: bool,
    pub request_record_retention_days: i64,
    pub request_record_payload_retention_days: i64,
    pub request_record_level: RequestRecordLevel,
    pub max_request_body_size_kb: i64,
    pub max_response_body_size_kb: i64,
    pub sensitive_request_headers: String,
    pub record_request_headers: bool,
    pub record_request_body: bool,
    pub record_response_body: bool,
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

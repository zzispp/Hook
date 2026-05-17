use rust_decimal::Decimal;
use types::provider::{ProviderCooldownPolicy, ProviderSchedulingMode};
use types::system_setting::{DisplayCurrency, EmailSuffixMode, RequestRecordLevel, SmtpEncryption};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SystemSettingsRecordPatch {
    pub site_name: Option<String>,
    pub site_subtitle: Option<String>,
    pub allow_registration: Option<bool>,
    pub login_captcha_enabled: Option<bool>,
    pub registration_captcha_enabled: Option<bool>,
    pub registration_email_verification_enabled: Option<bool>,
    pub email_config_enabled: Option<bool>,
    pub support_ticket_email_notifications_enabled: Option<bool>,
    pub auto_delete_expired_tokens: Option<bool>,
    pub request_record_retention_days: Option<i64>,
    pub request_record_payload_retention_days: Option<i64>,
    pub performance_monitoring_retention_days: Option<i64>,
    pub request_record_level: Option<RequestRecordLevel>,
    pub max_request_body_size_kb: Option<i64>,
    pub max_response_body_size_kb: Option<i64>,
    pub sensitive_request_headers: Option<String>,
    pub record_request_headers: Option<bool>,
    pub record_request_body: Option<bool>,
    pub record_response_body: Option<bool>,
    pub default_user_grant: Option<Decimal>,
    pub default_rate_limit_rpm: Option<i64>,
    pub scheduling_mode: Option<ProviderSchedulingMode>,
    pub provider_cooldown_policy: Option<ProviderCooldownPolicy>,
    pub currency: Option<DisplayCurrency>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<i64>,
    pub smtp_username: Option<String>,
    pub encrypted_smtp_password: Option<String>,
    pub smtp_from_email: Option<String>,
    pub smtp_from_name: Option<String>,
    pub smtp_encryption: Option<SmtpEncryption>,
    pub email_suffix_mode: Option<EmailSuffixMode>,
    pub email_suffixes: Option<String>,
    pub email_template_registration_subject: Option<String>,
    pub email_template_registration_html: Option<String>,
    pub email_template_password_reset_subject: Option<String>,
    pub email_template_password_reset_html: Option<String>,
}

pub struct SystemSettingsSmtpRecord {
    pub smtp_host: String,
    pub smtp_port: i64,
    pub smtp_username: String,
    pub encrypted_smtp_password: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub smtp_encryption: SmtpEncryption,
}

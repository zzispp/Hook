use sea_orm::entity::prelude::*;
use time::format_description::well_known::Rfc3339;
use types::provider::ProviderSchedulingMode;
use types::system_setting::{DisplayCurrency, EmailSuffixMode, RequestRecordLevel, SmtpEncryption, SystemSettings};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "system_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
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
    pub request_record_level: String,
    pub max_request_body_size_kb: i64,
    pub max_response_body_size_kb: i64,
    pub sensitive_request_headers: String,
    pub record_request_headers: bool,
    pub record_request_body: bool,
    pub record_response_body: bool,
    pub default_user_grant: rust_decimal::Decimal,
    pub default_rate_limit_rpm: i64,
    pub scheduling_mode: String,
    pub currency: String,
    pub smtp_host: String,
    pub smtp_port: i64,
    pub smtp_username: String,
    pub encrypted_smtp_password: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub smtp_encryption: String,
    pub email_suffix_mode: String,
    pub email_suffixes: String,
    pub email_template_registration_subject: String,
    pub email_template_registration_html: String,
    pub email_template_password_reset_subject: String,
    pub email_template_password_reset_html: String,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl TryFrom<Model> for SystemSettings {
    type Error = String;

    fn try_from(value: Model) -> Result<Self, Self::Error> {
        Ok(Self {
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
            request_record_level: RequestRecordLevel::try_from(value.request_record_level.as_str())?,
            max_request_body_size_kb: value.max_request_body_size_kb,
            max_response_body_size_kb: value.max_response_body_size_kb,
            sensitive_request_headers: value.sensitive_request_headers,
            record_request_headers: value.record_request_headers,
            record_request_body: value.record_request_body,
            record_response_body: value.record_response_body,
            default_user_grant: value.default_user_grant,
            default_rate_limit_rpm: value.default_rate_limit_rpm,
            scheduling_mode: ProviderSchedulingMode::from(value.scheduling_mode.as_str()),
            currency: DisplayCurrency::from(value.currency.as_str()),
            smtp_host: value.smtp_host,
            smtp_port: value.smtp_port,
            smtp_username: value.smtp_username,
            smtp_password_set: !value.encrypted_smtp_password.is_empty(),
            smtp_from_email: value.smtp_from_email,
            smtp_from_name: value.smtp_from_name,
            smtp_encryption: SmtpEncryption::try_from(value.smtp_encryption.as_str())?,
            email_suffix_mode: EmailSuffixMode::try_from(value.email_suffix_mode.as_str())?,
            email_suffixes: value.email_suffixes,
            email_template_registration_subject: value.email_template_registration_subject,
            email_template_registration_html: value.email_template_registration_html,
            email_template_password_reset_subject: value.email_template_password_reset_subject,
            email_template_password_reset_html: value.email_template_password_reset_html,
            created_at: format_timestamp(value.created_at),
            updated_at: format_timestamp(value.updated_at),
        })
    }
}

fn format_timestamp(value: TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("system setting timestamp must format as RFC3339")
}

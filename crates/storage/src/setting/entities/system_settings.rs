use sea_orm::entity::prelude::*;
use time::format_description::well_known::Rfc3339;
use types::provider::{ProviderCooldownPolicy, ProviderPriorityMode, ProviderSchedulingMode};
use types::system_setting::{EmailSuffixMode, RequestRecordLevel, SmtpEncryption, SystemSettings};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "system_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub site_name: String,
    pub site_subtitle: String,
    pub public_base_url: String,
    pub site_logo_base64: String,
    pub contact_methods: String,
    pub allow_registration: bool,
    pub login_captcha_enabled: bool,
    pub registration_captcha_enabled: bool,
    pub support_ticket_captcha_enabled: bool,
    pub recharge_captcha_enabled: bool,
    pub registration_email_verification_enabled: bool,
    pub auth_github_enabled: bool,
    pub auth_github_client_id: String,
    pub encrypted_auth_github_client_secret: String,
    pub auth_google_enabled: bool,
    pub auth_google_client_id: String,
    pub encrypted_auth_google_client_secret: String,
    pub auth_evm_enabled: bool,
    pub auth_evm_chain_ids: String,
    pub auth_evm_statement: String,
    pub password_reset_enabled: bool,
    pub email_config_enabled: bool,
    pub support_ticket_email_notifications_enabled: bool,
    pub default_user_group_code: String,
    pub token_limit_per_user: i64,
    pub client_request_record_level: String,
    pub client_record_request_headers: bool,
    pub client_record_request_body: bool,
    pub client_record_response_headers: bool,
    pub client_record_response_body: bool,
    pub client_max_request_body_size_kb: i64,
    pub client_max_response_body_size_kb: i64,
    pub client_sensitive_request_headers: String,
    pub provider_request_record_level: String,
    pub provider_record_request_headers: bool,
    pub provider_record_request_body: bool,
    pub provider_record_response_headers: bool,
    pub provider_record_response_body: bool,
    pub provider_max_request_body_size_kb: i64,
    pub provider_max_response_body_size_kb: i64,
    pub provider_sensitive_request_headers: String,
    pub default_user_grant: rust_decimal::Decimal,
    pub default_rate_limit_rpm: i64,
    pub recharge_enabled: bool,
    pub recharge_arrival_ratio: rust_decimal::Decimal,
    pub recharge_order_expire_minutes: i64,
    pub recharge_max_unpaid_orders: i64,
    pub recharge_min_amount: rust_decimal::Decimal,
    pub recharge_max_amount: rust_decimal::Decimal,
    pub scheduling_mode: String,
    pub provider_priority_mode: String,
    pub key_priority_snapshot_initialized: bool,
    pub cache_affinity_ttl_minutes: i64,
    pub provider_cooldown_policy: String,
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
            public_base_url: value.public_base_url,
            site_logo_base64: value.site_logo_base64,
            contact_methods: decode_contact_methods(&value.contact_methods)?,
            allow_registration: value.allow_registration,
            login_captcha_enabled: value.login_captcha_enabled,
            registration_captcha_enabled: value.registration_captcha_enabled,
            support_ticket_captcha_enabled: value.support_ticket_captcha_enabled,
            recharge_captcha_enabled: value.recharge_captcha_enabled,
            registration_email_verification_enabled: value.registration_email_verification_enabled,
            auth_github_enabled: value.auth_github_enabled,
            auth_github_client_id: value.auth_github_client_id,
            auth_github_client_secret_set: !value.encrypted_auth_github_client_secret.is_empty(),
            auth_google_enabled: value.auth_google_enabled,
            auth_google_client_id: value.auth_google_client_id,
            auth_google_client_secret_set: !value.encrypted_auth_google_client_secret.is_empty(),
            auth_evm_enabled: value.auth_evm_enabled,
            auth_evm_chain_ids: value.auth_evm_chain_ids,
            auth_evm_statement: value.auth_evm_statement,
            password_reset_enabled: value.password_reset_enabled,
            email_config_enabled: value.email_config_enabled,
            support_ticket_email_notifications_enabled: value.support_ticket_email_notifications_enabled,
            default_user_group_code: value.default_user_group_code,
            token_limit_per_user: value.token_limit_per_user,
            client_request_record_level: RequestRecordLevel::try_from(value.client_request_record_level.as_str())?,
            client_record_request_headers: value.client_record_request_headers,
            client_record_request_body: value.client_record_request_body,
            client_record_response_headers: value.client_record_response_headers,
            client_record_response_body: value.client_record_response_body,
            client_max_request_body_size_kb: value.client_max_request_body_size_kb,
            client_max_response_body_size_kb: value.client_max_response_body_size_kb,
            client_sensitive_request_headers: value.client_sensitive_request_headers,
            provider_request_record_level: RequestRecordLevel::try_from(value.provider_request_record_level.as_str())?,
            provider_record_request_headers: value.provider_record_request_headers,
            provider_record_request_body: value.provider_record_request_body,
            provider_record_response_headers: value.provider_record_response_headers,
            provider_record_response_body: value.provider_record_response_body,
            provider_max_request_body_size_kb: value.provider_max_request_body_size_kb,
            provider_max_response_body_size_kb: value.provider_max_response_body_size_kb,
            provider_sensitive_request_headers: value.provider_sensitive_request_headers,
            default_user_grant: value.default_user_grant,
            default_rate_limit_rpm: value.default_rate_limit_rpm,
            recharge_enabled: value.recharge_enabled,
            recharge_arrival_ratio: value.recharge_arrival_ratio,
            recharge_order_expire_minutes: value.recharge_order_expire_minutes,
            recharge_max_unpaid_orders: value.recharge_max_unpaid_orders,
            recharge_min_amount: value.recharge_min_amount,
            recharge_max_amount: value.recharge_max_amount,
            scheduling_mode: ProviderSchedulingMode::from(value.scheduling_mode.as_str()),
            provider_priority_mode: ProviderPriorityMode::from(value.provider_priority_mode.as_str()),
            key_priority_snapshot_initialized: value.key_priority_snapshot_initialized,
            cache_affinity_ttl_minutes: value.cache_affinity_ttl_minutes,
            provider_cooldown_policy: decode_provider_cooldown_policy(&value.provider_cooldown_policy)?,
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

fn decode_provider_cooldown_policy(value: &str) -> Result<ProviderCooldownPolicy, String> {
    serde_json::from_str(value).map_err(|error| format!("invalid provider cooldown policy: {error}"))
}

fn decode_contact_methods(value: &str) -> Result<Vec<types::system_setting::ContactMethod>, String> {
    serde_json::from_str(value).map_err(|error| format!("invalid contact methods: {error}"))
}

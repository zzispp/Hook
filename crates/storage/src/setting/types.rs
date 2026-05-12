use rust_decimal::Decimal;
use types::provider::ProviderSchedulingMode;
use types::system_setting::{DisplayCurrency, RequestRecordLevel};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SystemSettingsRecordPatch {
    pub site_name: Option<String>,
    pub site_subtitle: Option<String>,
    pub allow_registration: Option<bool>,
    pub login_captcha_enabled: Option<bool>,
    pub registration_captcha_enabled: Option<bool>,
    pub auto_delete_expired_tokens: Option<bool>,
    pub request_record_retention_days: Option<i64>,
    pub request_record_payload_retention_days: Option<i64>,
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
    pub currency: Option<DisplayCurrency>,
}

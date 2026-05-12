use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::provider::ProviderSchedulingMode;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DisplayCurrency {
    Usd,
    Cny,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestRecordLevel {
    #[default]
    Basic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemSettings {
    pub site_name: String,
    pub site_subtitle: String,
    pub allow_registration: bool,
    pub login_captcha_enabled: bool,
    pub registration_captcha_enabled: bool,
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
    pub created_at: String,
    pub updated_at: String,
}

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
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SystemSettingsResponse {
    pub site_name: String,
    pub site_subtitle: String,
    pub allow_registration: bool,
    pub login_captcha_enabled: bool,
    pub registration_captcha_enabled: bool,
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
    #[serde(with = "rust_decimal::serde::float")]
    pub default_user_grant: Decimal,
    pub default_rate_limit_rpm: i64,
    pub scheduling_mode: ProviderSchedulingMode,
    pub currency: DisplayCurrency,
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

impl SystemSettingsUpdate {
    pub fn is_empty(&self) -> bool {
        self.site_name.is_none()
            && self.site_subtitle.is_none()
            && self.allow_registration.is_none()
            && self.login_captcha_enabled.is_none()
            && self.registration_captcha_enabled.is_none()
            && self.auto_delete_expired_tokens.is_none()
            && self.request_record_retention_days.is_none()
            && self.request_record_payload_retention_days.is_none()
            && self.request_record_level.is_none()
            && self.max_request_body_size_kb.is_none()
            && self.max_response_body_size_kb.is_none()
            && self.sensitive_request_headers.is_none()
            && self.record_request_headers.is_none()
            && self.record_request_body.is_none()
            && self.record_response_body.is_none()
            && self.default_user_grant.is_none()
            && self.default_rate_limit_rpm.is_none()
            && self.scheduling_mode.is_none()
            && self.currency.is_none()
    }
}

impl From<SystemSettings> for SystemSettingsResponse {
    fn from(value: SystemSettings) -> Self {
        Self {
            site_name: value.site_name,
            site_subtitle: value.site_subtitle,
            allow_registration: value.allow_registration,
            login_captcha_enabled: value.login_captcha_enabled,
            registration_captcha_enabled: value.registration_captcha_enabled,
            auto_delete_expired_tokens: value.auto_delete_expired_tokens,
            request_record_retention_days: value.request_record_retention_days,
            request_record_payload_retention_days: value.request_record_payload_retention_days,
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
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl RequestRecordLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Basic => "basic",
        }
    }
}

impl TryFrom<&str> for RequestRecordLevel {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "basic" => Ok(Self::Basic),
            _ => Err(format!("unsupported request_record_level: {value}")),
        }
    }
}

impl DisplayCurrency {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Usd => "USD",
            Self::Cny => "CNY",
        }
    }
}

impl From<&str> for DisplayCurrency {
    fn from(value: &str) -> Self {
        match value {
            "CNY" => Self::Cny,
            _ => Self::Usd,
        }
    }
}

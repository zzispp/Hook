use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::provider::ProviderSchedulingMode;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DisplayCurrency {
    Usd,
    Cny,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemSettings {
    pub site_name: String,
    pub site_subtitle: String,
    pub allow_registration: bool,
    pub auto_delete_expired_tokens: bool,
    pub request_record_retention_days: i64,
    pub request_record_payload_retention_days: i64,
    pub default_user_grant: Decimal,
    pub default_rate_limit_rpm: i64,
    pub scheduling_mode: ProviderSchedulingMode,
    pub currency: DisplayCurrency,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct SystemSettingsUpdate {
    #[serde(default)]
    pub site_name: Option<String>,
    #[serde(default)]
    pub site_subtitle: Option<String>,
    #[serde(default)]
    pub allow_registration: Option<bool>,
    #[serde(default)]
    pub auto_delete_expired_tokens: Option<bool>,
    #[serde(default)]
    pub request_record_retention_days: Option<i64>,
    #[serde(default)]
    pub request_record_payload_retention_days: Option<i64>,
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
    pub auto_delete_expired_tokens: bool,
    pub request_record_retention_days: i64,
    pub request_record_payload_retention_days: i64,
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

impl SystemSettingsUpdate {
    pub fn is_empty(&self) -> bool {
        self.site_name.is_none()
            && self.site_subtitle.is_none()
            && self.allow_registration.is_none()
            && self.auto_delete_expired_tokens.is_none()
            && self.request_record_retention_days.is_none()
            && self.request_record_payload_retention_days.is_none()
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
            auto_delete_expired_tokens: value.auto_delete_expired_tokens,
            request_record_retention_days: value.request_record_retention_days,
            request_record_payload_retention_days: value.request_record_payload_retention_days,
            default_user_grant: value.default_user_grant,
            default_rate_limit_rpm: value.default_rate_limit_rpm,
            scheduling_mode: value.scheduling_mode,
            currency: value.currency,
            created_at: value.created_at,
            updated_at: value.updated_at,
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

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::provider::ProviderSchedulingMode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemSettings {
    pub site_name: String,
    pub site_subtitle: String,
    pub allow_registration: bool,
    pub auto_delete_expired_tokens: bool,
    pub default_user_grant: Decimal,
    pub default_rate_limit_rpm: i64,
    pub scheduling_mode: ProviderSchedulingMode,
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
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub default_user_grant: Option<Decimal>,
    #[serde(default)]
    pub default_rate_limit_rpm: Option<i64>,
    #[serde(default)]
    pub scheduling_mode: Option<ProviderSchedulingMode>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SystemSettingsResponse {
    pub site_name: String,
    pub site_subtitle: String,
    pub allow_registration: bool,
    pub auto_delete_expired_tokens: bool,
    #[serde(with = "rust_decimal::serde::float")]
    pub default_user_grant: Decimal,
    pub default_rate_limit_rpm: i64,
    pub scheduling_mode: ProviderSchedulingMode,
    pub created_at: String,
    pub updated_at: String,
}

impl SystemSettingsUpdate {
    pub fn is_empty(&self) -> bool {
        self.site_name.is_none()
            && self.site_subtitle.is_none()
            && self.allow_registration.is_none()
            && self.auto_delete_expired_tokens.is_none()
            && self.default_user_grant.is_none()
            && self.default_rate_limit_rpm.is_none()
            && self.scheduling_mode.is_none()
    }
}

impl From<SystemSettings> for SystemSettingsResponse {
    fn from(value: SystemSettings) -> Self {
        Self {
            site_name: value.site_name,
            site_subtitle: value.site_subtitle,
            allow_registration: value.allow_registration,
            auto_delete_expired_tokens: value.auto_delete_expired_tokens,
            default_user_grant: value.default_user_grant,
            default_rate_limit_rpm: value.default_rate_limit_rpm,
            scheduling_mode: value.scheduling_mode,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

use rust_decimal::Decimal;
use types::provider::ProviderSchedulingMode;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SystemSettingsRecordPatch {
    pub site_name: Option<String>,
    pub site_subtitle: Option<String>,
    pub allow_registration: Option<bool>,
    pub auto_delete_expired_tokens: Option<bool>,
    pub default_user_grant: Option<Decimal>,
    pub default_rate_limit_rpm: Option<i64>,
    pub scheduling_mode: Option<ProviderSchedulingMode>,
}

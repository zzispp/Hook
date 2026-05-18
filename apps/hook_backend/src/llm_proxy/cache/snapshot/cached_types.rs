use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use types::{
    model::TieredPricingConfig,
    provider::{ProviderCooldownPolicy, ProviderModelMapping, ProviderSchedulingMode},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SchedulingSnapshot {
    #[serde(default)]
    pub default_rate_limit_rpm: i64,
    pub scheduling_mode: ProviderSchedulingMode,
    pub record_request_headers: bool,
    pub record_request_body: bool,
    pub record_response_body: bool,
    pub max_request_body_size_kb: i64,
    pub max_response_body_size_kb: i64,
    pub sensitive_request_headers: String,
    #[serde(default)]
    pub provider_cooldown_policy: ProviderCooldownPolicy,
    pub models: Vec<CachedGlobalModel>,
    pub groups: Vec<CachedBillingGroup>,
    pub users: Vec<CachedUserAccess>,
    pub providers: Vec<CachedProvider>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedGlobalModel {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub default_price_per_request: Option<Decimal>,
    pub default_tiered_pricing: TieredPricingConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedBillingGroup {
    pub code: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub billing_multiplier: Decimal,
    pub allowed_model_ids: Vec<String>,
    pub allowed_provider_ids: Vec<String>,
    pub is_active: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedUserAccess {
    pub id: String,
    pub username: String,
    pub is_active: bool,
    pub allowed_model_ids: Vec<String>,
    pub allowed_provider_ids: Vec<String>,
    pub quota_mode: String,
    #[serde(default)]
    pub rate_limit_rpm: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedProvider {
    pub id: String,
    pub name: String,
    pub max_retries: Option<i32>,
    pub request_timeout_seconds: Option<f64>,
    pub stream_first_byte_timeout_seconds: Option<f64>,
    pub priority: i32,
    pub keep_priority_on_conversion: bool,
    pub enable_format_conversion: bool,
    pub is_active: bool,
    pub endpoints: Vec<CachedEndpoint>,
    pub keys: Vec<CachedProviderKey>,
    pub models: Vec<CachedModelBinding>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedEndpoint {
    pub id: String,
    pub provider_id: String,
    pub api_format: String,
    pub base_url: String,
    pub custom_path: Option<String>,
    pub max_retries: Option<i32>,
    pub is_active: bool,
    pub format_acceptance_config: Option<serde_json::Value>,
    pub header_rules: Option<serde_json::Value>,
    pub body_rules: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedProviderKey {
    pub id: String,
    pub provider_id: String,
    pub name: String,
    pub api_formats: Vec<String>,
    pub allowed_model_ids: Vec<String>,
    pub key_preview: String,
    pub encrypted_api_key: String,
    pub internal_priority: i32,
    #[serde(default)]
    pub rpm_limit: Option<i32>,
    pub cache_ttl_minutes: i32,
    #[serde(default)]
    pub time_range_enabled: bool,
    #[serde(default)]
    pub time_range_start_minute: Option<u16>,
    #[serde(default)]
    pub time_range_end_minute: Option<u16>,
    pub is_active: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedModelBinding {
    pub id: String,
    pub provider_id: String,
    pub global_model_id: String,
    pub provider_model_name: String,
    pub provider_model_mapping: Option<ProviderModelMapping>,
    pub is_active: bool,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub price_per_request: Option<Decimal>,
    pub tiered_pricing: Option<TieredPricingConfig>,
}

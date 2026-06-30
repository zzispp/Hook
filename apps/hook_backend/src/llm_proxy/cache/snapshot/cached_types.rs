use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use types::{
    model::TieredPricingConfig,
    provider::{ProviderCooldownPolicy, ProviderPriorityMode, ProviderSchedulingMode, RoutingProfileId},
    system_setting::RequestRecordLevel,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SchedulingSnapshot {
    #[serde(default)]
    pub default_rate_limit_rpm: i64,
    pub scheduling_mode: ProviderSchedulingMode,
    pub provider_priority_mode: ProviderPriorityMode,
    pub cache_affinity_ttl_minutes: i64,
    pub client_request_record_level: RequestRecordLevel,
    pub client_record_request_headers: bool,
    pub client_record_request_body: bool,
    pub client_record_response_headers: bool,
    pub client_record_response_body: bool,
    pub client_max_request_body_size_kb: i64,
    pub client_max_response_body_size_kb: i64,
    pub client_sensitive_request_headers: String,
    pub provider_request_record_level: RequestRecordLevel,
    pub provider_record_request_headers: bool,
    pub provider_record_request_body: bool,
    pub provider_record_response_headers: bool,
    pub provider_record_response_body: bool,
    pub provider_max_request_body_size_kb: i64,
    pub provider_max_response_body_size_kb: i64,
    pub provider_sensitive_request_headers: String,
    #[serde(default)]
    pub provider_cooldown_policy: ProviderCooldownPolicy,
    pub models: Vec<CachedGlobalModel>,
    pub groups: Vec<CachedBillingGroup>,
    pub active_user_group_codes: Vec<String>,
    pub users: Vec<CachedUserAccess>,
    pub providers: Vec<CachedProvider>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedGlobalModel {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    #[serde(default)]
    pub supported_capabilities: Option<Vec<String>>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub default_price_per_request: Option<Decimal>,
    pub default_tiered_pricing: TieredPricingConfig,
    #[serde(default)]
    pub routing_profile_id: Option<RoutingProfileId>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedBillingGroup {
    pub code: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub billing_multiplier: Decimal,
    pub allowed_model_ids: Vec<String>,
    pub allowed_provider_key_ids: Option<Vec<String>>,
    #[serde(default)]
    pub routing_profile_id: Option<RoutingProfileId>,
    #[serde(default)]
    pub provider_priorities: BTreeMap<String, i32>,
    #[serde(default)]
    pub provider_key_priorities: BTreeMap<String, i32>,
    pub visible_user_group_codes: Vec<String>,
    pub is_active: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedUserAccess {
    pub id: String,
    pub username: String,
    pub group_codes: Vec<String>,
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
    #[serde(default)]
    pub stream_response_headers_timeout_seconds: Option<f64>,
    pub stream_first_byte_timeout_seconds: Option<f64>,
    #[serde(default)]
    pub stream_first_token_timeout_seconds: Option<f64>,
    #[serde(default)]
    pub stream_idle_timeout_seconds: Option<f64>,
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
    pub global_priority_by_format: BTreeMap<String, i32>,
    #[serde(default)]
    pub rpm_limit: Option<i32>,
    pub cache_ttl_minutes: i32,
    #[serde(default)]
    pub time_range_enabled: bool,
    #[serde(default)]
    pub time_range_start_minute: Option<u16>,
    #[serde(default)]
    pub time_range_end_minute: Option<u16>,
    #[serde(default)]
    pub supports_image_generation: bool,
    pub is_active: bool,
    #[serde(default)]
    pub model_mappings: BTreeMap<String, CachedKeyModelMapping>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedModelBinding {
    pub id: String,
    pub provider_id: String,
    pub global_model_id: String,
    pub is_active: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedKeyModelMapping {
    pub provider_model_id: String,
    pub global_model_id: String,
    pub upstream_model_name: String,
    pub reasoning_effort: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectiveProviderModel {
    pub upstream_model_name: String,
    pub reasoning_effort: Option<String>,
}

impl CachedProviderKey {
    pub fn effective_provider_model(&self, binding: &CachedModelBinding, global_model: &CachedGlobalModel) -> EffectiveProviderModel {
        self.model_mappings
            .get(&binding.id)
            .map(|mapping| EffectiveProviderModel {
                upstream_model_name: mapping.upstream_model_name.clone(),
                reasoning_effort: mapping.reasoning_effort.clone(),
            })
            .unwrap_or_else(|| EffectiveProviderModel {
                upstream_model_name: global_model.name.clone(),
                reasoning_effort: None,
            })
    }
}

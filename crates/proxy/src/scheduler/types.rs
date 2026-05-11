use crate::format_conversion::ApiFormat;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchedulingMode {
    FixedOrder,
    CacheAffinity,
    LoadBalance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelAccessPolicy {
    All,
    Limited(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchedulerInput {
    pub group_code: String,
    pub group_is_active: bool,
    pub group_allowed_model_ids: Vec<String>,
    pub group_allowed_provider_ids: Vec<String>,
    pub token_model_policy: ModelAccessPolicy,
    pub requested_model_id: String,
    pub client_format: ApiFormat,
    pub is_stream: bool,
    pub affinity_key: Option<String>,
    pub scheduling_mode: SchedulingMode,
    pub global_keep_priority_on_conversion: bool,
    pub global_format_conversion_enabled: bool,
    pub providers: Vec<ProviderSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderSnapshot {
    pub id: String,
    pub name: String,
    pub priority: i32,
    pub keep_priority_on_conversion: bool,
    pub enable_format_conversion: bool,
    pub is_active: bool,
    pub endpoints: Vec<EndpointSnapshot>,
    pub keys: Vec<KeySnapshot>,
    pub models: Vec<ModelBindingSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndpointSnapshot {
    pub id: String,
    pub api_format: ApiFormat,
    pub is_active: bool,
    pub accepts_format_conversion: bool,
    pub supports_stream_conversion: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeySnapshot {
    pub id: String,
    pub internal_priority: i32,
    pub api_formats: Option<Vec<ApiFormat>>,
    pub cache_ttl_minutes: i32,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelBindingSnapshot {
    pub global_model_id: String,
    pub provider_model_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub provider_id: String,
    pub provider_name: String,
    pub endpoint_id: String,
    pub key_id: String,
    pub global_model_id: String,
    pub provider_model_name: String,
    pub provider_api_format: ApiFormat,
    pub needs_conversion: bool,
    pub is_cached: bool,
    pub provider_priority: i32,
    pub key_priority: i32,
}

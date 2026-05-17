use serde::{Deserialize, Serialize};

const DEFAULT_PROVIDER_COOLDOWN_LIMIT: u64 = 20;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProviderCooldownPolicy {
    #[serde(default)]
    pub window_seconds: i64,
    #[serde(default)]
    pub rules: Vec<ProviderCooldownRule>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProviderCooldownRule {
    pub status_code: i32,
    pub failure_count: i64,
    pub cooldown_seconds: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct ProviderCooldownListRequest {
    #[serde(default)]
    pub skip: u64,
    #[serde(default = "default_provider_cooldown_limit")]
    pub limit: u64,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub status_code: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderCooldownListResponse {
    pub cooldowns: Vec<ProviderCooldown>,
    pub total: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderCooldown {
    pub provider_id: String,
    pub provider_name: String,
    pub status_code: i32,
    pub observed_count: i64,
    pub threshold_count: i64,
    pub window_seconds: i64,
    pub cooldown_seconds: i64,
    pub triggered_at: String,
    pub cooldown_until: String,
    pub released_at: Option<String>,
    pub request_id: String,
    pub candidate_index: i32,
    pub retry_index: i32,
    pub endpoint_id: Option<String>,
    pub endpoint_name: Option<String>,
    pub key_id: Option<String>,
    pub key_name: Option<String>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub error_code: Option<String>,
    pub error_param: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn default_provider_cooldown_limit() -> u64 {
    DEFAULT_PROVIDER_COOLDOWN_LIMIT
}

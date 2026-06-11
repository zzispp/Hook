use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::model::{PatchField, deserialize_patch_value};

use super::quick_import_sync::ProviderQuickImportKeySyncInfo;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderApiKey {
    pub id: String,
    pub provider_id: String,
    pub name: String,
    pub api_formats: Vec<String>,
    pub allowed_model_ids: Vec<String>,
    pub note: Option<String>,
    pub internal_priority: i32,
    pub global_priority_by_format: BTreeMap<String, i32>,
    pub rpm_limit: Option<i32>,
    pub learned_rpm_limit: Option<i32>,
    pub cache_ttl_minutes: i32,
    pub max_probe_interval_minutes: i32,
    pub time_range_enabled: bool,
    pub time_range_start: Option<String>,
    pub time_range_end: Option<String>,
    pub health_by_format: Option<serde_json::Value>,
    pub circuit_breaker_by_format: Option<serde_json::Value>,
    pub is_active: bool,
    pub has_api_key: bool,
    pub quick_import_sync: Option<ProviderQuickImportKeySyncInfo>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderApiKeyCreate {
    pub name: String,
    pub api_key: String,
    #[serde(default)]
    pub api_formats: Vec<String>,
    #[serde(default)]
    pub allowed_model_ids: Vec<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub internal_priority: Option<i32>,
    #[serde(default)]
    pub rpm_limit: Option<i32>,
    #[serde(default)]
    pub cache_ttl_minutes: Option<i32>,
    #[serde(default)]
    pub max_probe_interval_minutes: Option<i32>,
    #[serde(default)]
    pub time_range_enabled: Option<bool>,
    #[serde(default)]
    pub time_range_start: Option<String>,
    #[serde(default)]
    pub time_range_end: Option<String>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct ProviderApiKeyUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub api_formats: Option<Vec<String>>,
    #[serde(default)]
    pub allowed_model_ids: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub note: PatchField<String>,
    #[serde(default)]
    pub internal_priority: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub rpm_limit: PatchField<i32>,
    #[serde(default)]
    pub cache_ttl_minutes: Option<i32>,
    #[serde(default)]
    pub max_probe_interval_minutes: Option<i32>,
    #[serde(default)]
    pub time_range_enabled: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub time_range_start: PatchField<String>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub time_range_end: PatchField<String>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ProviderApiKeyPriorityUpdate {
    pub provider_id: String,
    pub key_id: String,
    pub global_priority_by_format: BTreeMap<String, i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ProviderApiKeyPriorityBatchUpdate {
    #[serde(default)]
    pub updates: Vec<ProviderApiKeyPriorityUpdate>,
}

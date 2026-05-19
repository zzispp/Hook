use serde::{Deserialize, Serialize};

use crate::model::{PatchField, deserialize_patch_value};

const DEFAULT_PROVIDER_LIMIT: u64 = 100;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub max_retries: Option<i32>,
    pub request_timeout_seconds: Option<f64>,
    pub stream_first_byte_timeout_seconds: Option<f64>,
    pub stream_idle_timeout_seconds: Option<f64>,
    pub priority: i32,
    pub keep_priority_on_conversion: bool,
    pub enable_format_conversion: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct ProviderListRequest {
    #[serde(default)]
    pub skip: u64,
    #[serde(default = "default_provider_limit")]
    pub limit: u64,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub api_format: Option<String>,
    #[serde(default)]
    pub model_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderCreate {
    pub name: String,
    pub provider_type: String,
    #[serde(default)]
    pub max_retries: Option<i32>,
    #[serde(default)]
    pub request_timeout_seconds: Option<f64>,
    #[serde(default)]
    pub stream_first_byte_timeout_seconds: Option<f64>,
    #[serde(default)]
    pub stream_idle_timeout_seconds: Option<f64>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub keep_priority_on_conversion: Option<bool>,
    #[serde(default)]
    pub enable_format_conversion: Option<bool>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct ProviderUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub provider_type: Option<String>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub max_retries: PatchField<i32>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub request_timeout_seconds: PatchField<f64>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub stream_first_byte_timeout_seconds: PatchField<f64>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub stream_idle_timeout_seconds: PatchField<f64>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub keep_priority_on_conversion: Option<bool>,
    #[serde(default)]
    pub enable_format_conversion: Option<bool>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderListResponse {
    pub providers: Vec<Provider>,
    pub total: u64,
}

impl ProviderUpdate {
    pub fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.provider_type.is_none()
            && self.max_retries.is_missing()
            && self.request_timeout_seconds.is_missing()
            && self.stream_first_byte_timeout_seconds.is_missing()
            && self.stream_idle_timeout_seconds.is_missing()
            && self.priority.is_none()
            && self.keep_priority_on_conversion.is_none()
            && self.enable_format_conversion.is_none()
            && self.is_active.is_none()
    }
}

fn default_provider_limit() -> u64 {
    DEFAULT_PROVIDER_LIMIT
}

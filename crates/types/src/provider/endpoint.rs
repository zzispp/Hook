use serde::{Deserialize, Serialize};

use crate::model::{PatchField, deserialize_patch_value};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderEndpoint {
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
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderEndpointCreate {
    pub api_format: String,
    pub base_url: String,
    #[serde(default)]
    pub custom_path: Option<String>,
    #[serde(default)]
    pub max_retries: Option<i32>,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub format_acceptance_config: Option<serde_json::Value>,
    #[serde(default)]
    pub header_rules: Option<serde_json::Value>,
    #[serde(default)]
    pub body_rules: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct ProviderEndpointUpdate {
    #[serde(default)]
    pub api_format: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub custom_path: PatchField<String>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub max_retries: PatchField<i32>,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub format_acceptance_config: PatchField<serde_json::Value>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub header_rules: PatchField<serde_json::Value>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub body_rules: PatchField<serde_json::Value>,
}

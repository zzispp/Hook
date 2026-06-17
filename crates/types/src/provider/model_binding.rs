use serde::{Deserialize, Serialize};

use crate::model::{PatchField, deserialize_patch_value};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderModelBinding {
    pub id: String,
    pub provider_id: String,
    pub global_model_id: String,
    pub is_active: bool,
    pub config: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderModelBindingCreate {
    pub global_model_id: String,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderModelBindingBatchUpdate {
    #[serde(default)]
    pub create: Vec<ProviderModelBindingCreate>,
    #[serde(default)]
    pub delete_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct ProviderModelBindingUpdate {
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub config: PatchField<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderUpstreamModelsResponse {
    pub models: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProviderKeyModelMapping {
    pub id: String,
    pub provider_id: String,
    pub key_id: String,
    pub provider_model_id: String,
    pub global_model_id: String,
    pub upstream_model_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderKeyModelMappingsByKey {
    pub provider_id: String,
    pub key_id: String,
    pub key_name: String,
    pub is_quick_import_key: bool,
    pub mappings: Vec<ProviderKeyModelMapping>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderKeyModelMappingsResponse {
    pub provider_id: String,
    pub keys: Vec<ProviderKeyModelMappingsByKey>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderKeyModelMappingCandidate {
    pub upstream_model_name: String,
    pub suggested_global_model_id: Option<String>,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderKeyModelMappingsForKeyResponse {
    pub provider_id: String,
    pub key_id: String,
    pub key_name: String,
    pub is_quick_import_key: bool,
    pub mappings: Vec<ProviderKeyModelMapping>,
    pub candidates: Vec<ProviderKeyModelMappingCandidate>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ProviderKeyModelMappingInput {
    pub global_model_id: String,
    pub upstream_model_name: String,
    #[serde(default)]
    pub reasoning_effort: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderKeyModelMappingsUpdate {
    #[serde(default)]
    pub model_mappings: Vec<ProviderKeyModelMappingInput>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderModelTestRequest {
    pub endpoint_id: String,
    pub key_id: String,
    #[serde(default)]
    pub request_headers: std::collections::BTreeMap<String, String>,
    pub request_body: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderModelTestEndpoint {
    pub id: String,
    pub api_format: String,
    pub base_url: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderModelTestKey {
    pub id: String,
    pub name: String,
    pub preview: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderModelTestResponse {
    pub success: bool,
    pub model: String,
    pub endpoint: ProviderModelTestEndpoint,
    pub key: Option<ProviderModelTestKey>,
    pub status_code: Option<u16>,
    pub latency_ms: u128,
    pub request_url: String,
    pub request_body: serde_json::Value,
    pub response_headers: std::collections::BTreeMap<String, String>,
    pub response_body: serde_json::Value,
    pub error: Option<String>,
}

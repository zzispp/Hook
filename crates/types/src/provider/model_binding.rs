use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::model::{PatchField, TieredPricingConfig, deserialize_patch_value};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderModelMapping {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderModelBinding {
    pub id: String,
    pub provider_id: String,
    pub global_model_id: String,
    pub provider_model_name: String,
    pub provider_model_mapping: Option<ProviderModelMapping>,
    pub is_active: bool,
    pub price_per_request: Option<Decimal>,
    pub tiered_pricing: Option<TieredPricingConfig>,
    pub config: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderModelBindingCreate {
    pub global_model_id: String,
    pub provider_model_name: String,
    #[serde(default)]
    pub provider_model_mapping: Option<ProviderModelMapping>,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct ProviderModelBindingUpdate {
    #[serde(default)]
    pub provider_model_name: Option<String>,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub provider_model_mapping: PatchField<ProviderModelMapping>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub config: PatchField<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderUpstreamModelsResponse {
    pub models: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderModelTestRequest {
    pub endpoint_id: String,
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
pub struct ProviderModelTestResponse {
    pub success: bool,
    pub model: String,
    pub endpoint: ProviderModelTestEndpoint,
    pub status_code: Option<u16>,
    pub latency_ms: u128,
    pub request_url: String,
    pub request_body: serde_json::Value,
    pub response_headers: std::collections::BTreeMap<String, String>,
    pub response_body: serde_json::Value,
    pub error: Option<String>,
}

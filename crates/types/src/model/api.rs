use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{ModelPriceRange, PatchField, TieredPricingConfig, patch::deserialize_patch_value};

const DEFAULT_GLOBAL_MODEL_LIMIT: u64 = 100;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct GlobalModelListRequest {
    #[serde(default)]
    pub skip: u64,
    #[serde(default = "default_global_model_limit")]
    pub limit: u64,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub search: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct GlobalModelCreate {
    pub name: String,
    pub display_name: String,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub default_price_per_request: Option<Decimal>,
    pub default_tiered_pricing: TieredPricingConfig,
    #[serde(default)]
    pub supported_capabilities: Option<Vec<String>>,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct GlobalModelUpdate {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub default_price_per_request: PatchField<Decimal>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub default_tiered_pricing: PatchField<TieredPricingConfig>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub supported_capabilities: PatchField<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub config: PatchField<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GlobalModelResponse {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub is_active: bool,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub default_price_per_request: Option<Decimal>,
    pub default_tiered_pricing: TieredPricingConfig,
    pub supported_capabilities: Option<Vec<String>>,
    pub config: Option<serde_json::Value>,
    pub provider_count: Option<u64>,
    pub active_provider_count: Option<u64>,
    pub usage_count: Option<i64>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GlobalModelWithStats {
    #[serde(flatten)]
    pub model: GlobalModelResponse,
    pub total_models: u64,
    pub total_providers: u64,
    pub price_range: ModelPriceRange,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GlobalModelListResponse {
    pub models: Vec<GlobalModelResponse>,
    pub total: u64,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BatchDeleteGlobalModelsRequest {
    pub ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BatchDeleteGlobalModelsResponse {
    pub success_count: u64,
    pub failed: Vec<BatchDeleteFailure>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BatchDeleteFailure {
    pub id: String,
    pub error: String,
}

impl GlobalModelUpdate {
    pub fn is_empty(&self) -> bool {
        self.display_name.is_none()
            && self.is_active.is_none()
            && self.default_price_per_request.is_missing()
            && self.default_tiered_pricing.is_missing()
            && self.supported_capabilities.is_missing()
            && self.config.is_missing()
    }
}

fn default_global_model_limit() -> u64 {
    DEFAULT_GLOBAL_MODEL_LIMIT
}

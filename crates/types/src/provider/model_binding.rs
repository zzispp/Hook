use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::model::{PatchField, TieredPricingConfig, deserialize_patch_value};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderModelBinding {
    pub id: String,
    pub provider_id: String,
    pub global_model_id: String,
    pub provider_model_name: String,
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
    pub config: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct ProviderModelBindingUpdate {
    #[serde(default)]
    pub provider_model_name: Option<String>,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub config: PatchField<serde_json::Value>,
}

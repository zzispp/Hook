use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::model::TieredPricingConfig;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderModelBinding {
    pub id: String,
    pub provider_id: String,
    pub global_model_id: String,
    pub provider_model_name: String,
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

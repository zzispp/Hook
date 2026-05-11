use rust_decimal::Decimal;
use serde::Serialize;

use super::TieredPricingConfig;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ModelCapabilities {
    pub supports_vision: bool,
    pub supports_function_calling: bool,
    pub supports_streaming: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ModelPriceRange {
    #[serde(with = "rust_decimal::serde::float_option")]
    pub min_input: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub max_input: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub min_output: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub max_output: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ModelCatalogProviderDetail {
    pub provider_id: String,
    pub provider_name: String,
    pub model_id: Option<String>,
    pub target_model: String,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub input_price_per_1m: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub output_price_per_1m: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub cache_creation_price_per_1m: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub cache_read_price_per_1m: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub cache_1h_creation_price_per_1m: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub price_per_request: Option<Decimal>,
    pub effective_tiered_pricing: Option<TieredPricingConfig>,
    pub tier_count: u64,
    pub supports_vision: Option<bool>,
    pub supports_function_calling: Option<bool>,
    pub supports_streaming: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ModelCatalogItem {
    pub global_model_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub providers: Vec<ModelCatalogProviderDetail>,
    pub price_range: ModelPriceRange,
    pub total_providers: u64,
    pub capabilities: ModelCapabilities,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ModelCatalogResponse {
    pub models: Vec<ModelCatalogItem>,
    pub total: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GlobalModelProvidersResponse {
    pub providers: Vec<ModelCatalogProviderDetail>,
    pub total: u64,
}

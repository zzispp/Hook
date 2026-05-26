use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderModelCostMode {
    PerRequest,
    PerToken,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderModelCostSource {
    Configured,
    GlobalDefault,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderModelCost {
    pub id: String,
    pub provider_id: String,
    pub key_id: String,
    pub provider_model_id: String,
    pub cost_mode: ProviderModelCostMode,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub price_per_request: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub input_price_per_million: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub output_price_per_million: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub cache_creation_price_per_million: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub cache_read_price_per_million: Option<Decimal>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderModelCostUpsert {
    pub provider_model_id: String,
    pub cost_mode: ProviderModelCostMode,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub price_per_request: Option<Decimal>,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub input_price_per_million: Option<Decimal>,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub output_price_per_million: Option<Decimal>,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub cache_creation_price_per_million: Option<Decimal>,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub cache_read_price_per_million: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderModelCostBatchUpsert {
    #[serde(default)]
    pub costs: Vec<ProviderModelCostUpsert>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderModelCostListResponse {
    pub costs: Vec<ProviderModelCost>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct RequestUpstreamCost {
    pub upstream_cost_mode: Option<ProviderModelCostMode>,
    pub upstream_cost_source: Option<ProviderModelCostSource>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub upstream_price_per_request: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub upstream_input_price_per_million: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub upstream_output_price_per_million: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub upstream_cache_creation_price_per_million: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub upstream_cache_read_price_per_million: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub upstream_request_cost: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub upstream_input_cost: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub upstream_output_cost: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub upstream_cache_creation_cost: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub upstream_cache_read_cost: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub upstream_total_cost: Option<Decimal>,
}

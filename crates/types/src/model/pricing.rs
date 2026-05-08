use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CacheTTLPricing {
    pub ttl_minutes: u64,
    #[serde(with = "rust_decimal::serde::float")]
    pub cache_creation_price_per_1m: Decimal,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub cache_read_price_per_1m: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PricingTier {
    pub up_to: Option<u64>,
    #[serde(with = "rust_decimal::serde::float")]
    pub input_price_per_1m: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub output_price_per_1m: Decimal,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub cache_creation_price_per_1m: Option<Decimal>,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub cache_read_price_per_1m: Option<Decimal>,
    #[serde(default)]
    pub cache_ttl_pricing: Option<Vec<CacheTTLPricing>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TieredPricingConfig {
    pub tiers: Vec<PricingTier>,
}

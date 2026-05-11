use rust_decimal::Decimal;
use types::model::{GlobalModelResponse, ModelCatalogProviderDetail, ModelPriceRange, PricingTier, TieredPricingConfig};

use crate::{StorageResult, json};

#[path = "entities/mod.rs"]
pub mod entities;

pub use entities::{global_models, provider_models};

pub type GlobalModelRecord = global_models::Model;
pub type ModelRecord = provider_models::Model;

impl GlobalModelRecord {
    pub fn with_counts(self, provider_count: u64, active_provider_count: u64) -> StorageResult<GlobalModelResponse> {
        let default_tiered_pricing = self.default_tiered_pricing()?;
        let supported_capabilities = self.supported_capabilities()?;
        let config = self.config()?;
        Ok(GlobalModelResponse {
            id: self.id,
            name: self.name,
            display_name: self.display_name,
            is_active: self.is_active,
            default_price_per_request: self.default_price_per_request,
            default_tiered_pricing,
            supported_capabilities,
            config,
            provider_count: Some(provider_count),
            active_provider_count: Some(active_provider_count),
            usage_count: Some(self.usage_count),
            created_at: self.created_at.to_string(),
            updated_at: Some(self.updated_at.to_string()),
        })
    }

    pub fn price_range(&self) -> StorageResult<ModelPriceRange> {
        Ok(first_tier_range(self.default_tiered_pricing()?.tiers.first()))
    }

    pub fn default_tiered_pricing(&self) -> StorageResult<TieredPricingConfig> {
        json::decode_required(self.default_tiered_pricing.clone())
    }

    pub fn supported_capabilities(&self) -> StorageResult<Option<Vec<String>>> {
        json::decode_optional(self.supported_capabilities.clone())
    }

    pub fn config(&self) -> StorageResult<Option<serde_json::Value>> {
        json::decode_optional(self.config.clone())
    }
}

impl ModelRecord {
    pub fn provider_detail(self, global_model: &GlobalModelRecord) -> StorageResult<ModelCatalogProviderDetail> {
        let tiered = self.tiered_pricing()?.unwrap_or(global_model.default_tiered_pricing()?);
        let tier = tiered.tiers.first();
        Ok(ModelCatalogProviderDetail {
            provider_id: self.provider_id.clone(),
            provider_name: self.provider_id,
            model_id: Some(self.id),
            target_model: self.provider_model_name,
            input_price_per_1m: tier.map(|item| item.input_price_per_1m),
            output_price_per_1m: tier.map(|item| item.output_price_per_1m),
            cache_creation_price_per_1m: tier.and_then(|item| item.cache_creation_price_per_1m),
            cache_read_price_per_1m: tier.and_then(|item| item.cache_read_price_per_1m),
            cache_1h_creation_price_per_1m: tier.and_then(cache_1h_creation_price),
            price_per_request: self.price_per_request.or(global_model.default_price_per_request),
            effective_tiered_pricing: Some(tiered.clone()),
            tier_count: tiered.tiers.len() as u64,
            supports_vision: Some(global_model.config_bool("vision")?),
            supports_function_calling: Some(global_model.config_bool("function_calling")?),
            supports_streaming: Some(global_model.config_bool_with_default("streaming", true)?),
        })
    }

    pub fn tiered_pricing(&self) -> StorageResult<Option<TieredPricingConfig>> {
        json::decode_optional(self.tiered_pricing.clone())
    }
}

impl GlobalModelRecord {
    fn config_bool(&self, key: &str) -> StorageResult<bool> {
        self.config_bool_with_default(key, false)
    }

    fn config_bool_with_default(&self, key: &str, default: bool) -> StorageResult<bool> {
        Ok(self
            .config()?
            .and_then(|config| config.get(key).and_then(serde_json::Value::as_bool))
            .unwrap_or(default))
    }
}

fn first_tier_range(tier: Option<&PricingTier>) -> ModelPriceRange {
    ModelPriceRange {
        min_input: tier.map(|item| item.input_price_per_1m),
        max_input: tier.map(|item| item.input_price_per_1m),
        min_output: tier.map(|item| item.output_price_per_1m),
        max_output: tier.map(|item| item.output_price_per_1m),
    }
}

fn cache_1h_creation_price(tier: &PricingTier) -> Option<Decimal> {
    tier.cache_ttl_pricing
        .as_ref()?
        .iter()
        .find(|item| item.ttl_minutes == 60)
        .map(|item| item.cache_creation_price_per_1m)
}

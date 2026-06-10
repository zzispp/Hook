use rust_decimal::Decimal;
use types::{
    model::{GlobalModelResponse, PricingTier},
    provider::{ProviderModelCostMode, ProviderModelCostUpsert},
};

use crate::application::{ProviderError, ProviderResult};

const CACHE_READ_TTL_MINUTES: u64 = 5;

pub fn model_cost(global_model: &GlobalModelResponse, multiplier: Decimal) -> ProviderResult<ProviderModelCostUpsert> {
    if let Some(tier) = global_model.default_tiered_pricing.tiers.first() {
        return Ok(token_cost(global_model.id.clone(), tier, multiplier));
    }
    request_cost(global_model, multiplier)
}

pub fn has_default_cost(global_model: &GlobalModelResponse) -> bool {
    !global_model.default_tiered_pricing.tiers.is_empty() || global_model.default_price_per_request.is_some()
}

fn request_cost(global_model: &GlobalModelResponse, multiplier: Decimal) -> ProviderResult<ProviderModelCostUpsert> {
    let Some(price) = global_model.default_price_per_request else {
        return Err(ProviderError::InvalidInput(format!("global model has no default cost: {}", global_model.name)));
    };
    Ok(ProviderModelCostUpsert {
        provider_model_id: global_model.id.clone(),
        cost_mode: ProviderModelCostMode::PerRequest,
        price_per_request: Some(price * multiplier),
        input_price_per_million: None,
        output_price_per_million: None,
        cache_creation_price_per_million: None,
        cache_read_price_per_million: None,
    })
}

fn token_cost(global_model_id: String, tier: &PricingTier, multiplier: Decimal) -> ProviderModelCostUpsert {
    let input = tier.input_price_per_1m * multiplier;
    ProviderModelCostUpsert {
        provider_model_id: global_model_id,
        cost_mode: ProviderModelCostMode::PerToken,
        price_per_request: None,
        input_price_per_million: Some(input),
        output_price_per_million: Some(tier.output_price_per_1m * multiplier),
        cache_creation_price_per_million: Some(cache_creation_price(tier) * multiplier),
        cache_read_price_per_million: Some(cache_read_price(tier) * multiplier),
    }
}

fn cache_creation_price(tier: &PricingTier) -> Decimal {
    tier.cache_creation_price_per_1m.unwrap_or(tier.input_price_per_1m * Decimal::new(125, 2))
}

fn cache_read_price(tier: &PricingTier) -> Decimal {
    tier.cache_read_price_per_1m
        .or_else(|| cache_ttl_read_price(tier))
        .unwrap_or(tier.input_price_per_1m * Decimal::new(1, 1))
}

fn cache_ttl_read_price(tier: &PricingTier) -> Option<Decimal> {
    tier.cache_ttl_pricing
        .as_ref()?
        .iter()
        .find(|item| item.ttl_minutes == CACHE_READ_TTL_MINUTES)
        .and_then(|item| item.cache_read_price_per_1m)
}

#[cfg(test)]
mod tests {
    use types::model::TieredPricingConfig;

    use super::*;

    #[test]
    fn token_cost_applies_effective_multiplier() {
        let model = global_model(TieredPricingConfig { tiers: vec![tier()] }, None);
        let cost = model_cost(&model, Decimal::new(1, 1)).unwrap();

        assert_eq!(cost.input_price_per_million, Some(Decimal::new(1, 2)));
        assert_eq!(cost.cache_creation_price_per_million, Some(Decimal::new(125, 4)));
        assert_eq!(cost.cache_read_price_per_million, Some(Decimal::new(1, 3)));
    }

    #[test]
    fn model_without_default_cost_is_rejected() {
        let model = global_model(TieredPricingConfig { tiers: vec![] }, None);
        assert!(model_cost(&model, Decimal::ONE).is_err());
    }

    fn global_model(tiered: TieredPricingConfig, request_price: Option<Decimal>) -> GlobalModelResponse {
        GlobalModelResponse {
            id: "model-id".into(),
            name: "gpt-test".into(),
            display_name: "GPT Test".into(),
            is_active: true,
            default_price_per_request: request_price,
            default_tiered_pricing: tiered,
            supported_capabilities: None,
            config: None,
            provider_count: None,
            active_provider_count: None,
            usage_count: None,
            created_at: String::new(),
            updated_at: None,
        }
    }

    fn tier() -> PricingTier {
        PricingTier {
            up_to: None,
            input_price_per_1m: Decimal::new(1, 1),
            output_price_per_1m: Decimal::new(2, 1),
            cache_creation_price_per_1m: None,
            cache_read_price_per_1m: None,
            cache_ttl_pricing: None,
        }
    }
}

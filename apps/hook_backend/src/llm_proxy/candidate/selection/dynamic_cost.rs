use rust_decimal::Decimal;
use storage::provider::ProviderStore;
use types::{
    model::PricingTier,
    provider::{ProviderModelCost, ProviderModelCostMode, RoutingRequestFeatures},
};

use super::GlobalModelRef;
use crate::llm_proxy::{LlmProxyError, LlmProxyState};

const PRICE_SCALE: i64 = 1_000_000;

pub(super) async fn model_cost_config(state: &LlmProxyState, key_id: &str, model_id: &str) -> Result<Option<ProviderModelCost>, LlmProxyError> {
    Ok(ProviderStore::new(state.database.clone()).find_model_cost(key_id, model_id).await?)
}

pub(super) fn estimated_cost_from_config(
    configured: Option<&ProviderModelCost>,
    global_model: &GlobalModelRef,
    features: &RoutingRequestFeatures,
) -> Option<Decimal> {
    configured
        .and_then(|cost| configured_cost(cost, features))
        .or_else(|| default_cost(global_model, features))
}

fn configured_cost(cost: &ProviderModelCost, features: &RoutingRequestFeatures) -> Option<Decimal> {
    match cost.cost_mode {
        ProviderModelCostMode::PerRequest => cost.price_per_request,
        ProviderModelCostMode::PerToken => token_price_basis(
            cost.input_price_per_million,
            cost.output_price_per_million,
            cost.cache_creation_price_per_million,
            cost.cache_read_price_per_million,
            features,
        ),
    }
}

fn default_cost(model: &GlobalModelRef, features: &RoutingRequestFeatures) -> Option<Decimal> {
    model
        .default_price_per_request
        .or_else(|| model.default_tiered_pricing.tiers.first().and_then(|tier| tier_cost(tier, features)))
}

fn tier_cost(tier: &PricingTier, features: &RoutingRequestFeatures) -> Option<Decimal> {
    token_price_basis(
        Some(tier.input_price_per_1m),
        Some(tier.output_price_per_1m),
        tier.cache_creation_price_per_1m,
        tier.cache_read_price_per_1m,
        features,
    )
}

fn token_price_basis(
    input: Option<Decimal>,
    output: Option<Decimal>,
    cache_write: Option<Decimal>,
    cache_read: Option<Decimal>,
    features: &RoutingRequestFeatures,
) -> Option<Decimal> {
    if features.input_token_estimate.is_some() || features.output_token_estimate.is_some() {
        return request_token_cost(input, output, features);
    }
    let total = input.unwrap_or_default() + output.unwrap_or_default() + cache_write.unwrap_or_default() + cache_read.unwrap_or_default();
    (total > Decimal::ZERO).then(|| total / Decimal::from(PRICE_SCALE))
}

fn request_token_cost(input: Option<Decimal>, output: Option<Decimal>, features: &RoutingRequestFeatures) -> Option<Decimal> {
    let input_cost = feature_token_cost(input, features.input_token_estimate);
    let output_cost = feature_token_cost(output, features.output_token_estimate);
    let total = input_cost + output_cost;
    (total > Decimal::ZERO).then_some(total)
}

fn feature_token_cost(price_per_million: Option<Decimal>, tokens: Option<u64>) -> Decimal {
    let Some(price) = price_per_million else {
        return Decimal::ZERO;
    };
    let Some(tokens) = tokens else {
        return Decimal::ZERO;
    };
    price * Decimal::from(tokens) / Decimal::from(PRICE_SCALE)
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use types::provider::RoutingRequestFeatures;

    use super::token_price_basis;

    #[test]
    fn token_price_uses_request_estimates_when_available() {
        let features = RoutingRequestFeatures::new("openai:chat", false, Some(2_000), Some(1_000), None);

        let cost = token_price_basis(Some(Decimal::from(10)), Some(Decimal::from(20)), None, None, &features);

        assert_eq!(cost, Some(Decimal::new(4, 2)));
    }

    #[test]
    fn token_price_keeps_unit_basis_when_estimates_are_unknown() {
        let features = RoutingRequestFeatures::unknown("openai:chat", false, None);

        let cost = token_price_basis(Some(Decimal::from(10)), Some(Decimal::from(20)), None, None, &features);

        assert_eq!(cost, Some(Decimal::new(3, 5)));
    }
}

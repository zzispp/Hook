use rust_decimal::Decimal;
use storage::provider::ProviderStore;
use types::{
    model::PricingTier,
    provider::{ProviderModelCost, ProviderModelCostMode},
};

use super::GlobalModelRef;
use crate::llm_proxy::{LlmProxyError, LlmProxyState};

const PRICE_SCALE: i64 = 1_000_000;

pub(super) async fn estimated_cost(
    state: &LlmProxyState,
    key_id: &str,
    model_id: &str,
    global_model: &GlobalModelRef,
) -> Result<Option<Decimal>, LlmProxyError> {
    let configured = ProviderStore::new(state.database.clone()).find_model_cost(key_id, model_id).await?;
    Ok(configured.as_ref().and_then(configured_cost).or_else(|| default_cost(global_model)))
}

fn configured_cost(cost: &ProviderModelCost) -> Option<Decimal> {
    match cost.cost_mode {
        ProviderModelCostMode::PerRequest => cost.price_per_request,
        ProviderModelCostMode::PerToken => token_price_basis(
            cost.input_price_per_million,
            cost.output_price_per_million,
            cost.cache_creation_price_per_million,
            cost.cache_read_price_per_million,
        ),
    }
}

fn default_cost(model: &GlobalModelRef) -> Option<Decimal> {
    model
        .default_price_per_request
        .or_else(|| model.default_tiered_pricing.tiers.first().and_then(tier_cost))
}

fn tier_cost(tier: &PricingTier) -> Option<Decimal> {
    token_price_basis(
        Some(tier.input_price_per_1m),
        Some(tier.output_price_per_1m),
        tier.cache_creation_price_per_1m,
        tier.cache_read_price_per_1m,
    )
}

fn token_price_basis(input: Option<Decimal>, output: Option<Decimal>, cache_write: Option<Decimal>, cache_read: Option<Decimal>) -> Option<Decimal> {
    let total = input.unwrap_or_default() + output.unwrap_or_default() + cache_write.unwrap_or_default() + cache_read.unwrap_or_default();
    (total > Decimal::ZERO).then(|| total / Decimal::from(PRICE_SCALE))
}

use std::collections::HashMap;

use rust_decimal::{Decimal, prelude::ToPrimitive};
use storage::provider::{ProviderStore, RoutingMetricRecord};
use types::{
    model::{PricingTier, TieredPricingConfig},
    provider::{ProviderModelCost, ProviderModelCostMode, RouteIdentity},
};

use crate::llm_proxy::{
    LlmProxyError, LlmProxyState,
    cache::snapshot::{CachedGlobalModel, SchedulingSnapshot},
};

const PRICE_SCALE: i64 = 1_000_000;

pub(super) type CurrentCostCatalog = HashMap<RouteIdentity, f64>;

pub(super) async fn current_costs(state: &LlmProxyState, metrics: &[RoutingMetricRecord]) -> Result<CurrentCostCatalog, LlmProxyError> {
    let snapshot = state.scheduling_snapshot().await?;
    let store = ProviderStore::new(state.database.clone());
    let mut output = HashMap::new();
    for record in metrics {
        if output.contains_key(&record.route) {
            continue;
        }
        let Some(context) = current_cost_context(&snapshot, &record.route) else {
            continue;
        };
        let configured = store.find_model_cost(&record.route.key_id, &context.model_binding_id).await?;
        if let Some(cost) = current_unit_cost(configured.as_ref(), &context).and_then(decimal_f64) {
            output.insert(record.route.clone(), cost);
        }
    }
    Ok(output)
}

#[derive(Clone, Copy)]
pub(super) struct CostRange {
    min: Option<f64>,
    max: Option<f64>,
}

impl CostRange {
    pub(super) fn from_records(records: &[&RoutingMetricRecord], current_costs: &CurrentCostCatalog) -> Self {
        let mut values = records.iter().filter_map(|record| current_costs.get(&record.route).copied());
        let Some(first) = values.next() else {
            return Self { min: None, max: None };
        };
        let (min, max) = values.fold((first, first), |(min, max), value| (min.min(value), max.max(value)));
        Self {
            min: Some(min),
            max: Some(max),
        }
    }

    pub(super) fn score(self, value: Option<f64>) -> f64 {
        let Some(value) = value else {
            return 0.5;
        };
        let Some(min) = self.min else {
            return 1.0;
        };
        let Some(max) = self.max else {
            return 1.0;
        };
        if (max - min).abs() < f64::EPSILON {
            return 1.0;
        }
        ((max - value) / (max - min)).clamp(0.0, 1.0)
    }
}

struct CurrentCostContext {
    model_binding_id: String,
    default_price_per_request: Option<Decimal>,
    default_tiered_pricing: TieredPricingConfig,
}

fn current_cost_context(snapshot: &SchedulingSnapshot, route: &RouteIdentity) -> Option<CurrentCostContext> {
    let global_model = snapshot.models.iter().find(|model| model.id == route.global_model_id)?;
    let provider = snapshot.providers.iter().find(|provider| provider.id == route.provider_id)?;
    let binding = provider.models.iter().find(|model| model.global_model_id == route.global_model_id)?;
    Some(context(global_model, binding.id.clone()))
}

fn context(global_model: &CachedGlobalModel, model_binding_id: String) -> CurrentCostContext {
    CurrentCostContext {
        model_binding_id,
        default_price_per_request: global_model.default_price_per_request,
        default_tiered_pricing: global_model.default_tiered_pricing.clone(),
    }
}

fn current_unit_cost(configured: Option<&ProviderModelCost>, context: &CurrentCostContext) -> Option<Decimal> {
    configured
        .and_then(configured_unit_cost)
        .or_else(|| default_unit_cost(context.default_price_per_request, &context.default_tiered_pricing))
}

fn configured_unit_cost(cost: &ProviderModelCost) -> Option<Decimal> {
    match cost.cost_mode {
        ProviderModelCostMode::PerRequest => cost.price_per_request,
        ProviderModelCostMode::PerToken => token_unit_cost(
            cost.input_price_per_million,
            cost.output_price_per_million,
            cost.cache_creation_price_per_million,
            cost.cache_read_price_per_million,
        ),
    }
}

fn default_unit_cost(price_per_request: Option<Decimal>, tiered: &TieredPricingConfig) -> Option<Decimal> {
    price_per_request.or_else(|| tiered.tiers.first().and_then(tier_unit_cost))
}

fn tier_unit_cost(tier: &PricingTier) -> Option<Decimal> {
    token_unit_cost(
        Some(tier.input_price_per_1m),
        Some(tier.output_price_per_1m),
        tier.cache_creation_price_per_1m,
        tier.cache_read_price_per_1m,
    )
}

fn token_unit_cost(input: Option<Decimal>, output: Option<Decimal>, cache_write: Option<Decimal>, cache_read: Option<Decimal>) -> Option<Decimal> {
    let total = input.unwrap_or_default() + output.unwrap_or_default() + cache_write.unwrap_or_default() + cache_read.unwrap_or_default();
    (total > Decimal::ZERO).then(|| total / Decimal::from(PRICE_SCALE))
}

fn decimal_f64(value: Decimal) -> Option<f64> {
    value.to_f64()
}

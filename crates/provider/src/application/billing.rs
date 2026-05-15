use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use types::model::{PricingTier, TieredPricingConfig};

const TOKENS_PER_MILLION: i64 = 1_000_000;
const BILLING_SCALE: u32 = 8;

#[derive(Clone, Debug, PartialEq)]
pub struct RequestBillingInput {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub cache_creation_input_tokens: i64,
    pub cache_read_input_tokens: i64,
    pub price_per_request: Option<Decimal>,
    pub tiered_pricing: TieredPricingConfig,
    pub billing_multiplier: Decimal,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RequestBillingAmount {
    pub input_cost: Decimal,
    pub output_cost: Decimal,
    pub cache_creation_cost: Decimal,
    pub cache_read_cost: Decimal,
    pub request_cost: Decimal,
    pub token_cost: Decimal,
    pub base_cost: Decimal,
    pub total_cost: Decimal,
    pub billing_multiplier: Decimal,
    pub input_price_per_1m: Option<Decimal>,
    pub output_price_per_1m: Option<Decimal>,
    pub cache_creation_price_per_1m: Option<Decimal>,
    pub cache_read_price_per_1m: Option<Decimal>,
    pub currency: String,
}

pub fn calculate_request_billing(input: RequestBillingInput) -> RequestBillingAmount {
    let tier = tier_for_tokens(&input.tiered_pricing, input.prompt_tokens + input.cache_read_input_tokens);
    let breakdown = tier.map(|tier| cost_breakdown(&input, tier)).unwrap_or_default();
    let token_cost = breakdown.token_cost();
    let request_cost = input.price_per_request.unwrap_or(Decimal::ZERO);
    let base_cost = token_cost + request_cost;
    let total_cost = quantize(base_cost * input.billing_multiplier);

    RequestBillingAmount {
        input_cost: quantize(breakdown.input_cost),
        output_cost: quantize(breakdown.output_cost),
        cache_creation_cost: quantize(breakdown.cache_creation_cost),
        cache_read_cost: quantize(breakdown.cache_read_cost),
        request_cost: quantize(request_cost),
        token_cost: quantize(token_cost),
        base_cost: quantize(base_cost),
        total_cost,
        billing_multiplier: input.billing_multiplier,
        input_price_per_1m: tier.map(|tier| tier.input_price_per_1m),
        output_price_per_1m: tier.map(|tier| tier.output_price_per_1m),
        cache_creation_price_per_1m: tier.and_then(|tier| tier.cache_creation_price_per_1m),
        cache_read_price_per_1m: tier.and_then(cache_read_price),
        currency: currency::ACCOUNTING_CURRENCY.into(),
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct CostBreakdown {
    input_cost: Decimal,
    output_cost: Decimal,
    cache_creation_cost: Decimal,
    cache_read_cost: Decimal,
}

impl CostBreakdown {
    fn token_cost(self) -> Decimal {
        self.input_cost + self.output_cost + self.cache_creation_cost + self.cache_read_cost
    }
}

fn tier_for_tokens(pricing: &TieredPricingConfig, total_input_tokens: i64) -> Option<&PricingTier> {
    let normalized = u64::try_from(total_input_tokens.max(0)).unwrap_or(0);
    pricing
        .tiers
        .iter()
        .find(|tier| tier.up_to.is_none_or(|limit| normalized <= limit))
        .or_else(|| pricing.tiers.last())
}

fn cost_breakdown(input: &RequestBillingInput, tier: &PricingTier) -> CostBreakdown {
    CostBreakdown {
        input_cost: per_million(input.prompt_tokens, tier.input_price_per_1m),
        output_cost: per_million(input.completion_tokens, tier.output_price_per_1m),
        cache_creation_cost: optional_per_million(input.cache_creation_input_tokens, tier.cache_creation_price_per_1m),
        cache_read_cost: optional_per_million(input.cache_read_input_tokens, cache_read_price(tier)),
    }
}

fn cache_read_price(tier: &PricingTier) -> Option<Decimal> {
    tier.cache_read_price_per_1m.or_else(|| {
        tier.cache_ttl_pricing
            .as_ref()?
            .iter()
            .find(|item| item.ttl_minutes == 5)
            .and_then(|item| item.cache_read_price_per_1m)
    })
}

fn optional_per_million(tokens: i64, price: Option<Decimal>) -> Decimal {
    price.map(|value| per_million(tokens, value)).unwrap_or(Decimal::ZERO)
}

fn per_million(tokens: i64, price: Decimal) -> Decimal {
    Decimal::from_i64(tokens.max(0)).unwrap_or(Decimal::ZERO) * price / Decimal::from(TOKENS_PER_MILLION)
}

fn quantize(value: Decimal) -> Decimal {
    value.round_dp(BILLING_SCALE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn billing_applies_token_price_request_price_and_group_multiplier() {
        let billing = calculate_request_billing(RequestBillingInput {
            prompt_tokens: 1_000,
            completion_tokens: 200,
            cache_creation_input_tokens: 100,
            cache_read_input_tokens: 50,
            price_per_request: Some(Decimal::new(1, 2)),
            tiered_pricing: TieredPricingConfig {
                tiers: vec![PricingTier {
                    up_to: None,
                    input_price_per_1m: Decimal::new(250, 2),
                    output_price_per_1m: Decimal::new(1500, 2),
                    cache_creation_price_per_1m: Some(Decimal::new(125, 2)),
                    cache_read_price_per_1m: Some(Decimal::new(25, 2)),
                    cache_ttl_pricing: None,
                }],
            },
            billing_multiplier: Decimal::new(2, 0),
        });

        assert_eq!(billing.token_cost, Decimal::new(563750, 8));
        assert_eq!(billing.base_cost, Decimal::new(1563750, 8));
        assert_eq!(billing.total_cost, Decimal::new(3127500, 8));
        assert_eq!(billing.currency, "USD");
    }

    #[test]
    fn billing_exposes_cost_breakdown_and_unit_prices() {
        let billing = calculate_request_billing(RequestBillingInput {
            prompt_tokens: 1_000,
            completion_tokens: 200,
            cache_creation_input_tokens: 100,
            cache_read_input_tokens: 50,
            price_per_request: Some(Decimal::new(1, 2)),
            tiered_pricing: TieredPricingConfig {
                tiers: vec![PricingTier {
                    up_to: None,
                    input_price_per_1m: Decimal::new(250, 2),
                    output_price_per_1m: Decimal::new(1500, 2),
                    cache_creation_price_per_1m: Some(Decimal::new(125, 2)),
                    cache_read_price_per_1m: Some(Decimal::new(25, 2)),
                    cache_ttl_pricing: None,
                }],
            },
            billing_multiplier: Decimal::new(2, 0),
        });

        assert_eq!(billing.input_cost, Decimal::new(250000, 8));
        assert_eq!(billing.output_cost, Decimal::new(300000, 8));
        assert_eq!(billing.cache_creation_cost, Decimal::new(12500, 8));
        assert_eq!(billing.cache_read_cost, Decimal::new(1250, 8));
        assert_eq!(billing.request_cost, Decimal::new(1, 2));
        assert_eq!(billing.input_price_per_1m, Some(Decimal::new(250, 2)));
        assert_eq!(billing.output_price_per_1m, Some(Decimal::new(1500, 2)));
        assert_eq!(billing.cache_creation_price_per_1m, Some(Decimal::new(125, 2)));
        assert_eq!(billing.cache_read_price_per_1m, Some(Decimal::new(25, 2)));
    }

    #[test]
    fn billing_selects_tier_by_input_context_plus_cache_read_tokens() {
        let billing = calculate_request_billing(RequestBillingInput {
            prompt_tokens: 900,
            completion_tokens: 100,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 200,
            price_per_request: None,
            tiered_pricing: TieredPricingConfig {
                tiers: vec![
                    PricingTier {
                        up_to: Some(1_000),
                        input_price_per_1m: Decimal::new(1, 0),
                        output_price_per_1m: Decimal::new(1, 0),
                        cache_creation_price_per_1m: None,
                        cache_read_price_per_1m: None,
                        cache_ttl_pricing: None,
                    },
                    PricingTier {
                        up_to: None,
                        input_price_per_1m: Decimal::new(2, 0),
                        output_price_per_1m: Decimal::new(3, 0),
                        cache_creation_price_per_1m: None,
                        cache_read_price_per_1m: Some(Decimal::new(1, 0)),
                        cache_ttl_pricing: None,
                    },
                ],
            },
            billing_multiplier: Decimal::new(1, 0),
        });

        assert_eq!(billing.token_cost, Decimal::new(230000, 8));
    }
}

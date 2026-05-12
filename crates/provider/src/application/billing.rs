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
    pub token_cost: Decimal,
    pub base_cost: Decimal,
    pub total_cost: Decimal,
    pub billing_multiplier: Decimal,
    pub currency: String,
}

pub fn calculate_request_billing(input: RequestBillingInput) -> RequestBillingAmount {
    let tier = tier_for_tokens(&input.tiered_pricing, input.prompt_tokens + input.cache_read_input_tokens);
    let token_cost = tier.map(|tier| token_cost(&input, tier)).unwrap_or(Decimal::ZERO);
    let request_cost = input.price_per_request.unwrap_or(Decimal::ZERO);
    let base_cost = token_cost + request_cost;
    let total_cost = quantize(base_cost * input.billing_multiplier);

    RequestBillingAmount {
        token_cost: quantize(token_cost),
        base_cost: quantize(base_cost),
        total_cost,
        billing_multiplier: input.billing_multiplier,
        currency: "USD".into(),
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

fn token_cost(input: &RequestBillingInput, tier: &PricingTier) -> Decimal {
    per_million(input.prompt_tokens, tier.input_price_per_1m)
        + per_million(input.completion_tokens, tier.output_price_per_1m)
        + optional_per_million(input.cache_creation_input_tokens, tier.cache_creation_price_per_1m)
        + optional_per_million(input.cache_read_input_tokens, cache_read_price(tier))
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

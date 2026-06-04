use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde_json::json;
use types::model::{CacheTTLPricing, PricingTier, TieredPricingConfig};

use super::{calculate_default, legacy_cache_creation_rule, pricing};
use crate::application::billing::{BillingService, BillingServiceInput, RequestBillingAmount};

#[test]
fn split_cache_creation_tokens_are_billed() {
    let result = calculate_default(
        "claude:chat",
        BTreeMap::from([
            ("input_tokens".into(), json!(1000)),
            ("output_tokens".into(), json!(100)),
            ("cache_creation_input_tokens".into(), json!(200)),
            ("cache_creation_5m_input_tokens".into(), json!(80)),
            ("cache_creation_1h_input_tokens".into(), json!(120)),
        ]),
        pricing_with_cache_ttl(),
    );

    assert_eq!(result.snapshot.resolved_dimensions["cache_creation_tokens"], 200);
    assert_eq!(result.snapshot.resolved_dimensions["cache_creation_uncategorized_tokens"], 0);
    assert_eq!(result.snapshot.cost_breakdown["cache_creation_ephemeral_5m_cost"], Decimal::new(20000, 8));
    assert_eq!(result.snapshot.cost_breakdown["cache_creation_ephemeral_1h_cost"], Decimal::new(45000, 8));
    assert_eq!(RequestBillingAmount::from(result).cache_creation_cost, Decimal::new(65000, 8));
}

#[test]
fn split_only_cache_creation_tokens_are_billed() {
    let result = calculate_default(
        "claude:chat",
        BTreeMap::from([
            ("input_tokens".into(), json!(1000)),
            ("output_tokens".into(), json!(100)),
            ("cache_creation_5m_input_tokens".into(), json!(80)),
            ("cache_creation_1h_input_tokens".into(), json!(120)),
        ]),
        pricing(2, 4),
    );

    assert_eq!(result.snapshot.resolved_dimensions["cache_creation_tokens"], 200);
    assert_eq!(result.snapshot.resolved_dimensions["cache_creation_uncategorized_tokens"], 0);
    assert_eq!(RequestBillingAmount::from(result).cache_creation_cost, Decimal::new(50000, 8));
}

#[test]
fn unclassified_cache_creation_delta_is_billed_once() {
    let result = calculate_default(
        "claude:chat",
        BTreeMap::from([
            ("input_tokens".into(), json!(1000)),
            ("output_tokens".into(), json!(100)),
            ("cache_creation_input_tokens".into(), json!(250)),
            ("cache_creation_5m_input_tokens".into(), json!(80)),
            ("cache_creation_1h_input_tokens".into(), json!(120)),
        ]),
        pricing(2, 4),
    );

    assert_eq!(result.snapshot.resolved_dimensions["cache_creation_uncategorized_tokens"], 50);
    assert_eq!(result.snapshot.cost_breakdown["cache_creation_uncategorized_cost"], Decimal::new(12500, 8));
    assert_eq!(RequestBillingAmount::from(result).cache_creation_cost, Decimal::new(62500, 8));
}

#[test]
fn request_billing_amount_sums_legacy_cache_creation_cost() {
    let result = BillingService::calculate_from_response(BillingServiceInput {
        task_type: "chat".into(),
        model_name: "gpt-test".into(),
        global_model_id: "global".into(),
        provider_model_id: "model".into(),
        provider_id: "provider".into(),
        api_format: "openai:chat".into(),
        request: None,
        response: None,
        metadata: None,
        base_dimensions: BTreeMap::from([("cache_creation_tokens".into(), json!(100))]),
        group_code: None,
        billing_multiplier: Decimal::ONE,
        price_per_request: None,
        tiered_pricing: pricing(2, 4),
        explicit_rule: Some(legacy_cache_creation_rule()),
        collectors: Vec::new(),
    });

    assert_eq!(RequestBillingAmount::from(result).cache_creation_cost, Decimal::new(25000, 8));
}

fn pricing_with_cache_ttl() -> TieredPricingConfig {
    TieredPricingConfig {
        tiers: vec![PricingTier {
            up_to: None,
            input_price_per_1m: Decimal::new(2, 0),
            output_price_per_1m: Decimal::new(4, 0),
            cache_creation_price_per_1m: Some(Decimal::new(25, 1)),
            cache_read_price_per_1m: None,
            cache_ttl_pricing: Some(vec![
                CacheTTLPricing {
                    ttl_minutes: 5,
                    cache_creation_price_per_1m: Decimal::new(25, 1),
                    cache_read_price_per_1m: None,
                },
                CacheTTLPricing {
                    ttl_minutes: 60,
                    cache_creation_price_per_1m: Decimal::new(375, 2),
                    cache_read_price_per_1m: None,
                },
            ]),
        }],
    }
}

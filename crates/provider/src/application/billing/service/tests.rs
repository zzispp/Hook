use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde_json::json;
use types::model::{PricingTier, TieredPricingConfig};

use super::*;
use crate::application::billing::{CollectorSource, DimensionCollector, DimensionValueType};

#[test]
fn grouped_total_is_applied_after_base_cost() {
    let result = BillingService::calculate_from_response(BillingServiceInput {
        task_type: "chat".into(),
        model_name: "gpt-test".into(),
        global_model_id: "global".into(),
        provider_model_id: "model".into(),
        provider_id: "provider".into(),
        api_format: "openai_chat".into(),
        request: None,
        response: None,
        metadata: None,
        base_dimensions: BTreeMap::from([("input_tokens".into(), json!(1000)), ("output_tokens".into(), json!(1000))]),
        group_code: Some("vip".into()),
        billing_multiplier: Decimal::new(2, 0),
        price_per_request: None,
        tiered_pricing: pricing(2, 4),
        explicit_rule: None,
        collectors: Vec::new(),
    });

    assert_eq!(result.snapshot.base_total_cost, Decimal::new(600000, 8));
    assert_eq!(result.snapshot.total_cost, Decimal::new(1200000, 8));
}

#[test]
fn base_usage_dimensions_are_billed_when_collectors_have_no_value() {
    let result = BillingService::calculate_from_response(BillingServiceInput {
        task_type: "chat".into(),
        model_name: "gpt-test".into(),
        global_model_id: "global".into(),
        provider_model_id: "model".into(),
        provider_id: "provider".into(),
        api_format: "openai_chat".into(),
        request: None,
        response: Some(json!({"usage": {}})),
        metadata: None,
        base_dimensions: BTreeMap::from([("input_tokens".into(), json!(27)), ("output_tokens".into(), json!(1698))]),
        group_code: Some("default".into()),
        billing_multiplier: Decimal::ONE,
        price_per_request: None,
        tiered_pricing: pricing(5, 30),
        explicit_rule: None,
        collectors: vec![
            response_collector("input_tokens", "usage.prompt_tokens"),
            response_collector("output_tokens", "usage.completion_tokens"),
        ],
    });

    assert_eq!(result.snapshot.resolved_dimensions["input_tokens"], 27);
    assert_eq!(result.snapshot.resolved_dimensions["output_tokens"], 1698);
    assert_eq!(result.snapshot.total_cost, Decimal::new(5107500, 8));
}

fn pricing(input_price_per_1m: i64, output_price_per_1m: i64) -> TieredPricingConfig {
    TieredPricingConfig {
        tiers: vec![PricingTier {
            up_to: None,
            input_price_per_1m: Decimal::new(input_price_per_1m, 0),
            output_price_per_1m: Decimal::new(output_price_per_1m, 0),
            cache_creation_price_per_1m: None,
            cache_read_price_per_1m: None,
            cache_ttl_pricing: None,
        }],
    }
}

fn response_collector(dimension_name: &str, source_path: &str) -> DimensionCollector {
    DimensionCollector {
        api_format: "openai_chat".into(),
        task_type: "chat".into(),
        dimension_name: dimension_name.into(),
        source_type: CollectorSource::Response,
        source_path: Some(source_path.into()),
        value_type: DimensionValueType::Int,
        transform_expression: None,
        default_value: None,
        priority: 100,
        is_enabled: true,
    }
}

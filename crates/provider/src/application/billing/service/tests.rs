use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde_json::{Value, json};
use types::model::{PricingTier, TieredPricingConfig};

use super::*;
use crate::application::billing::rules::{BillingRule, BillingRuleLookup, BillingRuleScope};
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

#[test]
fn openai_cache_read_tokens_are_removed_from_billable_input() {
    let result = calculate_default(
        "openai_cli",
        BTreeMap::from([
            ("input_tokens".into(), json!(146000)),
            ("output_tokens".into(), json!(92)),
            ("cache_read_input_tokens".into(), json!(138000)),
        ]),
        pricing(2, 4),
    );

    assert_eq!(result.snapshot.resolved_dimensions["input_tokens"], 8000);
    assert_eq!(result.snapshot.resolved_dimensions["cache_read_tokens"], 138000);
    assert_eq!(result.snapshot.resolved_dimensions["total_input_context"], 146000);
    assert_eq!(result.snapshot.cost_breakdown["input_cost"], Decimal::new(1600000, 8));
    assert_eq!(result.snapshot.cost_breakdown["output_cost"], Decimal::new(36800, 8));
    assert_eq!(result.snapshot.cost_breakdown["cache_read_cost"], Decimal::new(2760000, 8));
    assert_eq!(result.snapshot.total_cost, Decimal::new(4396800, 8));
}

#[test]
fn gemini_cache_read_tokens_are_removed_from_billable_input() {
    let result = calculate_default(
        "gemini_chat",
        BTreeMap::from([
            ("input_tokens".into(), json!(100)),
            ("output_tokens".into(), json!(10)),
            ("cache_read_tokens".into(), json!(20)),
        ]),
        pricing(5, 30),
    );

    assert_eq!(result.snapshot.resolved_dimensions["input_tokens"], 80);
    assert_eq!(result.snapshot.resolved_dimensions["total_input_context"], 100);
    assert_eq!(result.snapshot.cost_breakdown["input_cost"], Decimal::new(40000, 8));
}

#[test]
fn openai_cache_creation_tokens_are_removed_from_billable_input() {
    let result = calculate_default(
        "openai_chat",
        BTreeMap::from([
            ("input_tokens".into(), json!(1000)),
            ("output_tokens".into(), json!(100)),
            ("cache_creation_tokens".into(), json!(200)),
            ("cache_read_tokens".into(), json!(300)),
        ]),
        pricing(2, 4),
    );

    assert_eq!(result.snapshot.resolved_dimensions["input_tokens"], 500);
    assert_eq!(result.snapshot.resolved_dimensions["total_input_context"], 1000);
    assert_eq!(result.snapshot.cost_breakdown["input_cost"], Decimal::new(100000, 8));
    assert_eq!(result.snapshot.cost_breakdown["cache_creation_cost"], Decimal::new(50000, 8));
    assert_eq!(result.snapshot.cost_breakdown["cache_read_cost"], Decimal::new(6000, 8));
}

#[test]
fn claude_cache_read_tokens_are_not_removed_from_billable_input() {
    let result = calculate_default(
        "claude_chat",
        BTreeMap::from([
            ("input_tokens".into(), json!(8000)),
            ("output_tokens".into(), json!(92)),
            ("cache_read_input_tokens".into(), json!(138000)),
        ]),
        pricing(2, 4),
    );

    assert_eq!(result.snapshot.resolved_dimensions["input_tokens"], 8000);
    assert_eq!(result.snapshot.resolved_dimensions["cache_read_tokens"], 138000);
    assert_eq!(result.snapshot.resolved_dimensions["total_input_context"], 146000);
    assert_eq!(result.snapshot.cost_breakdown["input_cost"], Decimal::new(1600000, 8));
    assert_eq!(result.snapshot.cost_breakdown["cache_read_cost"], Decimal::new(2760000, 8));
}

#[test]
fn explicit_billing_rules_keep_raw_input_tokens() {
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
        base_dimensions: BTreeMap::from([("input_tokens".into(), json!(100)), ("cache_read_tokens".into(), json!(20))]),
        group_code: None,
        billing_multiplier: Decimal::ONE,
        price_per_request: None,
        tiered_pricing: pricing(2, 4),
        explicit_rule: Some(raw_input_rule()),
        collectors: Vec::new(),
    });

    assert_eq!(result.snapshot.resolved_dimensions["input_tokens"], 100);
    assert_eq!(result.snapshot.resolved_dimensions["total_input_context"], 100);
    assert_eq!(result.snapshot.total_cost, Decimal::new(10000, 2));
}

#[test]
fn request_price_is_billed_without_token_usage() {
    let result = BillingService::calculate_from_response(BillingServiceInput {
        task_type: "image".into(),
        model_name: "gpt-image-2".into(),
        global_model_id: "global".into(),
        provider_model_id: "model".into(),
        provider_id: "provider".into(),
        api_format: "openai_image".into(),
        request: None,
        response: None,
        metadata: None,
        base_dimensions: BTreeMap::new(),
        group_code: None,
        billing_multiplier: Decimal::ONE,
        price_per_request: Some(Decimal::new(5, 1)),
        tiered_pricing: pricing(0, 0),
        explicit_rule: None,
        collectors: Vec::new(),
    });

    assert_eq!(result.status, BillingSnapshotStatus::Complete);
    assert_eq!(result.snapshot.cost_breakdown["request_cost"], Decimal::new(5, 1));
    assert_eq!(result.snapshot.total_cost, Decimal::new(5, 1));
}

fn calculate_default(api_format: &str, base_dimensions: BTreeMap<String, Value>, tiered_pricing: TieredPricingConfig) -> CostResult {
    BillingService::calculate_from_response(BillingServiceInput {
        task_type: "chat".into(),
        model_name: "gpt-test".into(),
        global_model_id: "global".into(),
        provider_model_id: "model".into(),
        provider_id: "provider".into(),
        api_format: api_format.into(),
        request: None,
        response: None,
        metadata: None,
        base_dimensions,
        group_code: None,
        billing_multiplier: Decimal::ONE,
        price_per_request: None,
        tiered_pricing,
        explicit_rule: None,
        collectors: Vec::new(),
    })
}

fn raw_input_rule() -> BillingRuleLookup {
    BillingRuleLookup {
        rule: BillingRule {
            id: "custom".into(),
            name: "custom".into(),
            task_type: "chat".into(),
            expression: "input_cost".into(),
            variables: json!({}),
            dimension_mappings: json!({
                "input_tokens": {"source": "dimension", "key": "input_tokens", "required": true},
                "input_cost": {"source": "computed", "expression": "input_tokens", "required": true}
            }),
        },
        scope: BillingRuleScope::Model,
        effective_task_type: "chat".into(),
    }
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

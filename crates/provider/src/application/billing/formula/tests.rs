use std::collections::BTreeMap;

use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde_json::{Value, json};

use super::{FormulaEngine, FormulaStatus, SafeExpressionEvaluator};

#[test]
fn evaluator_rejects_attribute_like_expression() {
    let err = SafeExpressionEvaluator::eval_decimal("__import__(1)", &BTreeMap::new()).unwrap_err();
    assert!(err.contains("dunder"));
}

#[test]
fn evaluator_calculates_decimal_expression() {
    let vars = BTreeMap::from([("input_tokens".into(), Value::from(1000)), ("price".into(), Value::from(2.5))]);
    let value = SafeExpressionEvaluator::eval_decimal("input_tokens * price / 1000000", &vars).unwrap();
    assert_eq!(value, Decimal::from_f64(0.0025).unwrap());
}

#[test]
fn required_dimension_missing_returns_incomplete() {
    let result = FormulaEngine::evaluate(
        "input_cost",
        BTreeMap::new(),
        BTreeMap::new(),
        object_map(json!({"input_tokens": {"source": "dimension", "required": true}})),
        false,
    )
    .unwrap();
    assert_eq!(result.status, FormulaStatus::Incomplete);
    assert_eq!(result.missing_required, vec!["input_tokens"]);
}

#[test]
fn computed_mapping_waits_for_dependency() {
    let result = FormulaEngine::evaluate(
        "total_cost",
        BTreeMap::new(),
        object_map(json!({"input_tokens": 100})),
        object_map(json!({
            "unit_cost": {"source": "computed", "expression": "input_tokens / 100"},
            "total_cost": {"source": "computed", "expression": "unit_cost * 2"}
        })),
        false,
    )
    .unwrap();
    assert_eq!(result.status, FormulaStatus::Complete);
    assert_eq!(result.cost, Decimal::new(200000000, 8));
}

#[test]
fn matrix_and_ttl_tiered_mappings_resolve_cost() {
    let result = FormulaEngine::evaluate(
        "matrix_cost + ttl_cost",
        BTreeMap::new(),
        object_map(json!({"region": "us", "total_input_context": 200, "cache_ttl_minutes": 60})),
        object_map(json!({
            "matrix_cost": {"source": "matrix", "key": "region", "map": {"us": 2}},
            "ttl_cost": {
                "source": "tiered",
                "tier_key": "total_input_context",
                "ttl_key": "cache_ttl_minutes",
                "ttl_value_key": "cache_read_price_per_1m",
                "tiers": [{"up_to": null, "value": 1, "cache_ttl_pricing": [
                    {"ttl_minutes": 5, "cache_read_price_per_1m": 3},
                    {"ttl_minutes": 60, "cache_read_price_per_1m": 4}
                ]}]
            }
        })),
        false,
    )
    .unwrap();
    assert_eq!(result.cost, Decimal::new(600000000, 8));
    assert_eq!(result.tier_index, Some(0));
}

#[test]
fn ttl_tiered_mapping_falls_back_when_ttl_is_not_exact_match() {
    let result = FormulaEngine::evaluate(
        "ttl_cost",
        BTreeMap::new(),
        object_map(json!({"total_input_context": 200, "cache_ttl_minutes": 30})),
        object_map(json!({
            "ttl_cost": {
                "source": "tiered",
                "tier_key": "total_input_context",
                "ttl_key": "cache_ttl_minutes",
                "ttl_value_key": "cache_read_price_per_1m",
                "tiers": [{"up_to": null, "value": 1, "cache_ttl_pricing": [
                    {"ttl_minutes": 5, "cache_read_price_per_1m": 3},
                    {"ttl_minutes": 60, "cache_read_price_per_1m": 4}
                ]}]
            }
        })),
        false,
    )
    .unwrap();

    assert_eq!(result.cost, Decimal::new(100000000, 8));
    assert_eq!(result.tier_index, Some(0));
}

#[test]
fn negative_cost_returns_incomplete() {
    let result = FormulaEngine::evaluate("-1", BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), false).unwrap();
    assert_eq!(result.status, FormulaStatus::Incomplete);
    assert_eq!(result.error.as_deref(), Some("negative_cost"));
}

fn object_map(value: Value) -> BTreeMap<String, Value> {
    value
        .as_object()
        .map(|object| object.iter().map(|(key, value)| (key.clone(), value.clone())).collect())
        .unwrap_or_default()
}

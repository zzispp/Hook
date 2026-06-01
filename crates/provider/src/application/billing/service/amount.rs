use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde_json::Value;

use super::super::types::{BillingSnapshot, CostResult, RequestBillingAmount};
use super::quantize;
use crate::application::billing::types::ACCOUNTING_CURRENCY;

impl From<CostResult> for RequestBillingAmount {
    fn from(result: CostResult) -> Self {
        let breakdown = &result.snapshot.cost_breakdown;
        let input_cost = breakdown_cost(breakdown, "input_cost");
        let output_cost = breakdown_cost(breakdown, "output_cost");
        let cache_creation_cost = cache_creation_breakdown_cost(breakdown);
        let cache_read_cost = breakdown_cost(breakdown, "cache_read_cost");
        let request_cost = breakdown_cost(breakdown, "request_cost");
        Self {
            input_cost,
            output_cost,
            cache_creation_cost,
            cache_read_cost,
            request_cost,
            token_cost: quantize(input_cost + output_cost + cache_creation_cost + cache_read_cost),
            base_cost: result.snapshot.base_total_cost,
            total_cost: result.snapshot.total_cost,
            billing_multiplier: result.snapshot.billing_multiplier,
            input_price_per_1m: snapshot_decimal(&result.snapshot, "input_price_per_1m"),
            output_price_per_1m: snapshot_decimal(&result.snapshot, "output_price_per_1m"),
            cache_creation_price_per_1m: snapshot_decimal(&result.snapshot, "cache_creation_price_per_1m"),
            cache_read_price_per_1m: snapshot_decimal(&result.snapshot, "cache_read_price_per_1m"),
            currency: ACCOUNTING_CURRENCY.into(),
            snapshot: result.snapshot,
        }
    }
}

fn cache_creation_breakdown_cost(breakdown: &BTreeMap<String, Decimal>) -> Decimal {
    let split_total = breakdown_cost(breakdown, "cache_creation_uncategorized_cost")
        + breakdown_cost(breakdown, "cache_creation_ephemeral_5m_cost")
        + breakdown_cost(breakdown, "cache_creation_ephemeral_1h_cost");
    if split_total != Decimal::ZERO {
        return split_total;
    }
    breakdown_cost(breakdown, "cache_creation_cost")
}

fn breakdown_cost(breakdown: &BTreeMap<String, Decimal>, key: &str) -> Decimal {
    breakdown.get(key).copied().unwrap_or(Decimal::ZERO)
}

fn snapshot_decimal(snapshot: &BillingSnapshot, key: &str) -> Option<Decimal> {
    snapshot.resolved_variables.get(key).and_then(|value| match value {
        Value::Number(number) => number.as_f64().and_then(Decimal::from_f64_retain),
        Value::String(text) => text.parse().ok(),
        _ => None,
    })
}

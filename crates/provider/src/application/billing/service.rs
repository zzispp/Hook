use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde_json::Value;
use time::format_description::well_known::Rfc3339;

use super::{
    BuiltinRuleInput, DimensionCollectInput, DimensionCollector, DimensionCollectorRuntime, FormulaEngine, FormulaStatus, RequestBillingAmount,
    rules::{BillingRuleLookup, BillingRuleScope, effective_rule_task_type, universal_rule},
    types::{ACCOUNTING_CURRENCY, BILLING_SNAPSHOT_SCHEMA_VERSION, BillingSnapshot, BillingSnapshotStatus, CostResult, quantize},
};

#[derive(Clone, Debug)]
pub struct BillingServiceInput {
    pub task_type: String,
    pub model_name: String,
    pub global_model_id: String,
    pub provider_model_id: String,
    pub provider_id: String,
    pub api_format: String,
    pub request: Option<Value>,
    pub response: Option<Value>,
    pub metadata: Option<Value>,
    pub base_dimensions: BTreeMap<String, Value>,
    pub group_code: Option<String>,
    pub billing_multiplier: Decimal,
    pub price_per_request: Option<Decimal>,
    pub tiered_pricing: types::model::TieredPricingConfig,
    pub explicit_rule: Option<BillingRuleLookup>,
    pub collectors: Vec<DimensionCollector>,
}

pub struct BillingService;

impl BillingService {
    pub fn calculate_from_response(input: BillingServiceInput) -> CostResult {
        let task_type = effective_rule_task_type(&input.task_type);
        let dimensions = normalized_dimensions(DimensionCollectorRuntime::collect(
            &input.collectors,
            DimensionCollectInput {
                request: input.request.clone(),
                response: input.response.clone(),
                metadata: input.metadata.clone(),
                base_dimensions: input.base_dimensions.clone(),
            },
        ));
        let lookup = input.explicit_rule.clone().unwrap_or_else(|| {
            let rule = universal_rule(BuiltinRuleInput {
                global_model_name: input.model_name.clone(),
                task_type: task_type.clone(),
                price_per_request: input.price_per_request,
                tiered_pricing: input.tiered_pricing.clone(),
            });
            BillingRuleLookup {
                rule,
                scope: BillingRuleScope::Default,
                effective_task_type: task_type.clone(),
            }
        });
        calculate_with_rule(input, lookup, dimensions)
    }
}

fn calculate_with_rule(input: BillingServiceInput, lookup: BillingRuleLookup, dimensions: BTreeMap<String, Value>) -> CostResult {
    let variables = object_map(lookup.rule.variables.clone());
    let mappings = object_map(lookup.rule.dimension_mappings.clone());
    let evaluated = FormulaEngine::evaluate(&lookup.rule.expression, variables, dimensions, mappings, false);
    let result = match evaluated {
        Ok(result) => result,
        Err(error) => {
            let snapshot = snapshot_for_incomplete(input, lookup, BTreeMap::new(), BTreeMap::new(), error.missing_required, None);
            return CostResult {
                cost: Decimal::ZERO,
                status: BillingSnapshotStatus::Incomplete,
                snapshot,
            };
        }
    };
    let status = match result.status {
        FormulaStatus::Complete => BillingSnapshotStatus::Complete,
        FormulaStatus::Incomplete => BillingSnapshotStatus::Incomplete,
    };
    let base_total = quantized_total(&result.cost_breakdown, result.cost);
    let total_cost = if status == BillingSnapshotStatus::Complete {
        quantize(base_total * input.billing_multiplier)
    } else {
        Decimal::ZERO
    };
    let snapshot = BillingSnapshot {
        schema_version: BILLING_SNAPSHOT_SCHEMA_VERSION.into(),
        rule_id: Some(lookup.rule.id),
        rule_name: Some(lookup.rule.name),
        scope: Some(scope_name(&lookup.scope).into()),
        expression: Some(lookup.rule.expression),
        resolved_dimensions: result.resolved_dimensions,
        resolved_variables: result.resolved_variables,
        cost_breakdown: result.cost_breakdown,
        base_total_cost: base_total,
        total_cost,
        group_code: input.group_code,
        billing_multiplier: input.billing_multiplier,
        tier_index: result.tier_index,
        tier_info: result.tier_info,
        missing_required: result.missing_required,
        status: status.clone(),
        calculated_at: now_rfc3339(),
        engine_version: BILLING_SNAPSHOT_SCHEMA_VERSION.into(),
    };
    CostResult {
        cost: total_cost,
        status,
        snapshot,
    }
}

impl From<CostResult> for RequestBillingAmount {
    fn from(result: CostResult) -> Self {
        let breakdown = &result.snapshot.cost_breakdown;
        let input_cost = breakdown_cost(breakdown, "input_cost");
        let output_cost = breakdown_cost(breakdown, "output_cost");
        let cache_creation_cost = breakdown_cost(breakdown, "cache_creation_cost");
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

fn normalized_dimensions(mut dimensions: BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    alias(&mut dimensions, "cache_creation_tokens", "cache_creation_input_tokens");
    alias(&mut dimensions, "cache_read_tokens", "cache_read_input_tokens");
    dimensions.entry("request_count".into()).or_insert(Value::from(1));
    if !dimensions.contains_key("total_input_context") {
        let total = int_dim(&dimensions, "input_tokens") + int_dim(&dimensions, "cache_creation_tokens") + int_dim(&dimensions, "cache_read_tokens");
        dimensions.insert("total_input_context".into(), Value::from(total));
    }
    dimensions
}

fn alias(dimensions: &mut BTreeMap<String, Value>, target: &str, source: &str) {
    if !dimensions.contains_key(target) {
        if let Some(value) = dimensions.get(source).cloned() {
            dimensions.insert(target.into(), value);
        }
    }
}

fn int_dim(dimensions: &BTreeMap<String, Value>, key: &str) -> i64 {
    dimensions.get(key).and_then(value_i64).unwrap_or(0)
}

fn value_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn object_map(value: Value) -> BTreeMap<String, Value> {
    value
        .as_object()
        .map(|object| object.iter().map(|(key, value)| (key.clone(), value.clone())).collect())
        .unwrap_or_default()
}

fn quantized_total(breakdown: &BTreeMap<String, Decimal>, fallback: Decimal) -> Decimal {
    if breakdown.is_empty() {
        return quantize(fallback);
    }
    quantize(breakdown.values().copied().sum())
}

fn snapshot_for_incomplete(
    input: BillingServiceInput,
    lookup: BillingRuleLookup,
    dimensions: BTreeMap<String, Value>,
    variables: BTreeMap<String, Value>,
    missing_required: Vec<String>,
    error: Option<String>,
) -> BillingSnapshot {
    let mut resolved_variables = variables;
    if let Some(error) = error {
        resolved_variables.insert("error".into(), Value::String(error));
    }
    BillingSnapshot {
        schema_version: BILLING_SNAPSHOT_SCHEMA_VERSION.into(),
        rule_id: Some(lookup.rule.id),
        rule_name: Some(lookup.rule.name),
        scope: Some(scope_name(&lookup.scope).into()),
        expression: Some(lookup.rule.expression),
        resolved_dimensions: dimensions,
        resolved_variables,
        cost_breakdown: BTreeMap::new(),
        base_total_cost: Decimal::ZERO,
        total_cost: Decimal::ZERO,
        group_code: input.group_code,
        billing_multiplier: input.billing_multiplier,
        tier_index: None,
        tier_info: None,
        missing_required,
        status: BillingSnapshotStatus::Incomplete,
        calculated_at: now_rfc3339(),
        engine_version: BILLING_SNAPSHOT_SCHEMA_VERSION.into(),
    }
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

fn scope_name(scope: &BillingRuleScope) -> &'static str {
    match scope {
        BillingRuleScope::Model => "model",
        BillingRuleScope::Global => "global",
        BillingRuleScope::Default => "default",
    }
}

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("billing timestamp must format as RFC3339")
}

#[cfg(test)]
mod tests;

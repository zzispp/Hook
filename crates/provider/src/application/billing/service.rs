mod amount;

use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde_json::Value;
use time::format_description::well_known::Rfc3339;

use super::{
    BuiltinRuleInput, DimensionCollectInput, DimensionCollector, DimensionCollectorRuntime, FormulaEngine, FormulaStatus,
    rules::{BillingRuleLookup, BillingRuleScope, effective_rule_task_type, universal_rule},
    types::{BILLING_SNAPSHOT_SCHEMA_VERSION, BillingSnapshot, BillingSnapshotStatus, CostResult, quantize},
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
        let collected_dimensions = DimensionCollectorRuntime::collect(
            &input.collectors,
            DimensionCollectInput {
                request: input.request.clone(),
                response: input.response.clone(),
                metadata: input.metadata.clone(),
                base_dimensions: input.base_dimensions.clone(),
            },
        );
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
        let dimensions = normalized_dimensions(&input.api_format, collected_dimensions, lookup.scope == BillingRuleScope::Default);
        calculate_with_rule(input, lookup, dimensions)
    }
}

pub fn normalized_default_dimensions(api_format: &str, dimensions: BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    normalized_dimensions(api_format, dimensions, true)
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

fn normalized_dimensions(api_format: &str, mut dimensions: BTreeMap<String, Value>, normalize_cache_tokens: bool) -> BTreeMap<String, Value> {
    alias(&mut dimensions, "cache_creation_tokens", "cache_creation_input_tokens");
    derive_cache_creation_tokens(&mut dimensions);
    derive_uncategorized_cache_creation_tokens(&mut dimensions);
    alias(&mut dimensions, "cache_read_tokens", "cache_read_input_tokens");
    dimensions.entry("request_count".into()).or_insert(Value::from(1));
    preserve_raw_input_tokens(&mut dimensions);
    if !dimensions.contains_key("total_input_context") {
        dimensions.insert("total_input_context".into(), Value::from(total_input_context(api_format, &dimensions)));
    }
    if normalize_cache_tokens {
        normalize_billable_input_tokens(api_format, &mut dimensions);
    }
    dimensions
}

fn derive_cache_creation_tokens(dimensions: &mut BTreeMap<String, Value>) {
    if dimensions.contains_key("cache_creation_tokens") {
        return;
    }
    let split_total = int_dim(dimensions, "cache_creation_5m_input_tokens").saturating_add(int_dim(dimensions, "cache_creation_1h_input_tokens"));
    if split_total > 0 {
        dimensions.insert("cache_creation_tokens".into(), Value::from(split_total));
    }
}

fn derive_uncategorized_cache_creation_tokens(dimensions: &mut BTreeMap<String, Value>) {
    if dimensions.contains_key("cache_creation_uncategorized_tokens") {
        return;
    }
    let split_total = int_dim(dimensions, "cache_creation_5m_input_tokens").saturating_add(int_dim(dimensions, "cache_creation_1h_input_tokens"));
    let uncategorized = int_dim(dimensions, "cache_creation_tokens").saturating_sub(split_total).max(0);
    dimensions.insert("cache_creation_uncategorized_tokens".into(), Value::from(uncategorized));
}

fn preserve_raw_input_tokens(dimensions: &mut BTreeMap<String, Value>) {
    if dimensions.contains_key("raw_input_tokens") {
        return;
    }
    if let Some(value) = dimensions.get("input_tokens").cloned() {
        dimensions.insert("raw_input_tokens".into(), value);
    }
}

fn total_input_context(api_format: &str, dimensions: &BTreeMap<String, Value>) -> i64 {
    let input_tokens = int_dim(dimensions, "input_tokens");
    if input_tokens_include_cache_dimensions(api_format) {
        return input_tokens;
    }
    input_tokens + int_dim(dimensions, "cache_creation_tokens") + int_dim(dimensions, "cache_read_tokens")
}

fn normalize_billable_input_tokens(api_format: &str, dimensions: &mut BTreeMap<String, Value>) {
    if !input_tokens_include_cache_dimensions(api_format) {
        return;
    }
    let input_tokens = int_dim(dimensions, "input_tokens");
    let cache_tokens = int_dim(dimensions, "cache_creation_tokens").saturating_add(int_dim(dimensions, "cache_read_tokens"));
    if input_tokens <= 0 || cache_tokens <= 0 {
        return;
    }
    dimensions.insert("input_tokens".into(), Value::from((input_tokens - cache_tokens).max(0)));
}

fn input_tokens_include_cache_dimensions(api_format: &str) -> bool {
    let api_format = api_format.trim().to_ascii_lowercase();
    api_format == "openai"
        || api_format == "gemini"
        || api_format.starts_with("openai_")
        || api_format.starts_with("openai:")
        || api_format.starts_with("gemini_")
        || api_format.starts_with("gemini:")
}

fn alias(dimensions: &mut BTreeMap<String, Value>, target: &str, source: &str) {
    if !dimensions.contains_key(target)
        && let Some(value) = dimensions.get(source).cloned()
    {
        dimensions.insert(target.into(), value);
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

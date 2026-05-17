mod expr;
mod mapping;
#[cfg(test)]
mod tests;
mod value;

use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde_json::Value;

pub use self::expr::SafeExpressionEvaluator;
use self::mapping::MappingEvaluation;
use super::types::quantize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BillingIncompleteError {
    pub missing_required: Vec<String>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormulaStatus {
    Complete,
    Incomplete,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FormulaEvaluationResult {
    pub status: FormulaStatus,
    pub cost: Decimal,
    pub resolved_dimensions: BTreeMap<String, Value>,
    pub resolved_variables: BTreeMap<String, Value>,
    pub cost_breakdown: BTreeMap<String, Decimal>,
    pub tier_index: Option<usize>,
    pub tier_info: Option<Value>,
    pub missing_required: Vec<String>,
    pub error: Option<String>,
}

pub struct FormulaEngine;

impl FormulaEngine {
    pub fn evaluate(
        expression: &str,
        variables: BTreeMap<String, Value>,
        dimensions: BTreeMap<String, Value>,
        mappings: BTreeMap<String, Value>,
        strict_mode: bool,
    ) -> Result<FormulaEvaluationResult, BillingIncompleteError> {
        let resolved = mapping::resolve_mappings(variables, dimensions, mappings);
        if !resolved.missing_required.is_empty() {
            return missing_required_result(resolved, strict_mode);
        }
        formula_result(expression, resolved, strict_mode)
    }
}

fn missing_required_result(resolved: MappingEvaluation, strict_mode: bool) -> Result<FormulaEvaluationResult, BillingIncompleteError> {
    if strict_mode {
        return Err(BillingIncompleteError {
            missing_required: resolved.missing_required,
            message: None,
        });
    }
    Ok(incomplete_result(resolved, None))
}

fn formula_result(expression: &str, resolved: MappingEvaluation, strict_mode: bool) -> Result<FormulaEvaluationResult, BillingIncompleteError> {
    match SafeExpressionEvaluator::eval_decimal(expression, &resolved.resolved) {
        Ok(cost) if cost >= Decimal::ZERO => Ok(complete_result(resolved, cost)),
        Ok(_) => Ok(incomplete_result(resolved, Some("negative_cost".into()))),
        Err(error) if strict_mode => Err(BillingIncompleteError {
            missing_required: Vec::new(),
            message: Some(error),
        }),
        Err(error) => Ok(incomplete_result(resolved, Some(error))),
    }
}

fn complete_result(resolved: MappingEvaluation, cost: Decimal) -> FormulaEvaluationResult {
    FormulaEvaluationResult {
        status: FormulaStatus::Complete,
        cost: quantize(cost),
        resolved_dimensions: resolved.dimensions,
        cost_breakdown: extract_cost_breakdown(&resolved.resolved),
        resolved_variables: resolved.resolved,
        tier_index: resolved.tier_index,
        tier_info: resolved.tier_info,
        missing_required: Vec::new(),
        error: None,
    }
}

fn incomplete_result(resolved: MappingEvaluation, error: Option<String>) -> FormulaEvaluationResult {
    FormulaEvaluationResult {
        status: FormulaStatus::Incomplete,
        cost: Decimal::ZERO,
        resolved_dimensions: resolved.dimensions,
        resolved_variables: resolved.resolved,
        cost_breakdown: BTreeMap::new(),
        tier_index: resolved.tier_index,
        tier_info: resolved.tier_info,
        missing_required: resolved.missing_required,
        error,
    }
}

fn extract_cost_breakdown(resolved: &BTreeMap<String, Value>) -> BTreeMap<String, Decimal> {
    resolved
        .iter()
        .filter(|(key, _)| key.ends_with("_cost"))
        .filter_map(|(key, value)| value::value_decimal(value).ok().map(|amount| (key.clone(), quantize(amount))))
        .collect()
}

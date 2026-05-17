use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde_json::Value;

use super::{
    expr::SafeExpressionEvaluator,
    value::{decimal_json, value_decimal},
};
mod tiered;

const MAX_COMPUTED_ITERATIONS: usize = 64;

pub(super) struct MappingEvaluation {
    pub(super) resolved: BTreeMap<String, Value>,
    pub(super) dimensions: BTreeMap<String, Value>,
    pub(super) missing_required: Vec<String>,
    pub(super) tier_index: Option<usize>,
    pub(super) tier_info: Option<Value>,
}

pub(super) struct MappingResolution {
    pub(super) value: Value,
    pub(super) missing_required: bool,
    pub(super) tier_meta: Option<TierMeta>,
}

pub(super) struct TierMeta {
    pub(super) tier_index: Option<usize>,
    pub(super) tier_info: Option<Value>,
}

pub(super) fn resolve_mappings(
    variables: BTreeMap<String, Value>,
    dimensions: BTreeMap<String, Value>,
    mappings: BTreeMap<String, Value>,
) -> MappingEvaluation {
    let mut resolved = variables;
    let mut missing_required = Vec::new();
    let mut computed = BTreeMap::new();
    let mut tier_index = None;
    let mut tier_info = None;

    for (name, mapping_value) in mappings {
        let mapping = mapping_value.as_object().cloned().unwrap_or_default();
        let source = mapping_string(&mapping, "source", "constant");
        if source == "computed" {
            computed.insert(name, Value::Object(mapping));
            continue;
        }
        if source == "constant" && resolved.contains_key(&name) {
            continue;
        }
        let mapped = resolve_mapping(&name, &source, &mapping, &dimensions);
        if let Some(meta) = mapped.tier_meta {
            tier_index = tier_index.or(meta.tier_index);
            tier_info = tier_info.or(meta.tier_info);
        }
        if mapped.missing_required {
            missing_required.push(name);
        } else {
            resolved.insert(name, mapped.value);
        }
    }
    resolve_computed(&mut resolved, &dimensions, computed, &mut missing_required);
    MappingEvaluation {
        resolved,
        dimensions,
        missing_required,
        tier_index,
        tier_info,
    }
}

fn resolve_mapping(name: &str, source: &str, mapping: &serde_json::Map<String, Value>, dimensions: &BTreeMap<String, Value>) -> MappingResolution {
    match source {
        "dimension" => resolve_dimension(name, mapping, dimensions),
        "matrix" => resolve_matrix(name, mapping, dimensions),
        "tiered" => tiered::resolve_tiered(mapping, dimensions, missing_or_default),
        _ => MappingResolution {
            value: mapping.get("default").cloned().unwrap_or(Value::from(0)),
            missing_required: false,
            tier_meta: None,
        },
    }
}

fn resolve_dimension(name: &str, mapping: &serde_json::Map<String, Value>, dimensions: &BTreeMap<String, Value>) -> MappingResolution {
    let key = mapping_string(mapping, "key", name);
    let required = mapping_bool(mapping, "required");
    let allow_zero = mapping_bool(mapping, "allow_zero");
    let default = mapping.get("default").cloned().unwrap_or(Value::from(0));
    let Some(raw) = dimensions.get(&key) else {
        return missing_or_default(required, default);
    };
    if is_missing_dimension(raw, allow_zero) {
        return missing_or_default(required, default);
    }
    let value = value_decimal(raw).map(decimal_json).unwrap_or_else(|_| raw.clone());
    MappingResolution {
        value,
        missing_required: false,
        tier_meta: None,
    }
}

fn resolve_matrix(name: &str, mapping: &serde_json::Map<String, Value>, dimensions: &BTreeMap<String, Value>) -> MappingResolution {
    let key = mapping_string(mapping, "key", name);
    let required = mapping_bool(mapping, "required");
    let default = mapping.get("default").cloned().unwrap_or(Value::from(0));
    let Some(raw) = dimensions.get(&key).and_then(Value::as_str) else {
        return missing_or_default(required, default);
    };
    mapping
        .get("map")
        .and_then(Value::as_object)
        .and_then(|map| map.get(raw))
        .cloned()
        .map(|value| MappingResolution {
            value,
            missing_required: false,
            tier_meta: None,
        })
        .unwrap_or_else(|| missing_or_default(required, default))
}

fn resolve_computed(
    resolved: &mut BTreeMap<String, Value>,
    dimensions: &BTreeMap<String, Value>,
    mut computed: BTreeMap<String, Value>,
    missing_required: &mut Vec<String>,
) {
    for _ in 0..MAX_COMPUTED_ITERATIONS.min(computed.len().saturating_add(4)) {
        let mut progressed = false;
        for (name, mapping_value) in computed.clone() {
            progressed |= resolve_computed_item(resolved, dimensions, &mut computed, missing_required, name, mapping_value);
        }
        if !progressed {
            break;
        }
    }
    for (name, mapping_value) in computed {
        let mapping = mapping_value.as_object().cloned().unwrap_or_default();
        apply_computed_default(
            resolved,
            missing_required,
            &name,
            mapping_bool(&mapping, "required"),
            mapping.get("default").cloned().unwrap_or(Value::from(0)),
        );
    }
}

fn resolve_computed_item(
    resolved: &mut BTreeMap<String, Value>,
    dimensions: &BTreeMap<String, Value>,
    computed: &mut BTreeMap<String, Value>,
    missing_required: &mut Vec<String>,
    name: String,
    mapping_value: Value,
) -> bool {
    if resolved.contains_key(&name) {
        computed.remove(&name);
        return false;
    }
    let mapping = mapping_value.as_object().cloned().unwrap_or_default();
    let required = mapping_bool(&mapping, "required");
    let default = mapping.get("default").cloned().unwrap_or(Value::from(0));
    let Some(expression) = mapping
        .get("expression")
        .or_else(|| mapping.get("transform_expression"))
        .and_then(Value::as_str)
    else {
        apply_computed_default(resolved, missing_required, &name, required, default);
        computed.remove(&name);
        return true;
    };
    apply_computed_expression(
        resolved,
        dimensions,
        computed,
        missing_required,
        &name,
        expression,
        ComputedDefault { required, default },
    )
}

struct ComputedDefault {
    required: bool,
    default: Value,
}

fn apply_computed_expression(
    resolved: &mut BTreeMap<String, Value>,
    dimensions: &BTreeMap<String, Value>,
    computed: &mut BTreeMap<String, Value>,
    missing_required: &mut Vec<String>,
    name: &str,
    expression: &str,
    fallback: ComputedDefault,
) -> bool {
    let mut env = dimensions.clone();
    env.extend(resolved.clone());
    match SafeExpressionEvaluator::eval_decimal(expression, &env) {
        Ok(value) => {
            resolved.insert(name.to_owned(), decimal_json(value));
            computed.remove(name);
            true
        }
        Err(error) if error.starts_with("missing_variable:") => false,
        Err(_) => {
            apply_computed_default(resolved, missing_required, name, fallback.required, fallback.default);
            computed.remove(name);
            true
        }
    }
}

fn apply_computed_default(resolved: &mut BTreeMap<String, Value>, missing_required: &mut Vec<String>, name: &str, required: bool, default: Value) {
    if required {
        missing_required.push(name.to_owned());
    } else {
        resolved.insert(name.to_owned(), default);
    }
}

fn missing_or_default(required: bool, default: Value) -> MappingResolution {
    MappingResolution {
        value: if required { Value::Null } else { default },
        missing_required: required,
        tier_meta: None,
    }
}

fn mapping_string(mapping: &serde_json::Map<String, Value>, key: &str, default: &str) -> String {
    mapping.get(key).and_then(Value::as_str).unwrap_or(default).to_ascii_lowercase()
}

fn mapping_bool(mapping: &serde_json::Map<String, Value>, key: &str) -> bool {
    mapping.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn is_missing_dimension(value: &Value, allow_zero: bool) -> bool {
    match value {
        Value::Null => true,
        Value::String(text) if text.is_empty() => true,
        value => !allow_zero && value_decimal(value).is_ok_and(|number| number == Decimal::ZERO),
    }
}

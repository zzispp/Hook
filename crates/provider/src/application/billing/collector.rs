use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::formula::SafeExpressionEvaluator;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollectorSource {
    Request,
    Response,
    Metadata,
    Computed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DimensionValueType {
    Float,
    Int,
    String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DimensionCollector {
    pub api_format: String,
    pub task_type: String,
    pub dimension_name: String,
    pub source_type: CollectorSource,
    pub source_path: Option<String>,
    pub value_type: DimensionValueType,
    pub transform_expression: Option<String>,
    pub default_value: Option<String>,
    pub priority: i32,
    pub is_enabled: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DimensionCollectInput {
    pub request: Option<Value>,
    pub response: Option<Value>,
    pub metadata: Option<Value>,
    pub base_dimensions: BTreeMap<String, Value>,
}

pub struct DimensionCollectorRuntime;

impl DimensionCollectorRuntime {
    pub fn collect(collectors: &[DimensionCollector], input: DimensionCollectInput) -> BTreeMap<String, Value> {
        let mut input = input;
        let mut dimensions = std::mem::take(&mut input.base_dimensions);
        let grouped = grouped_collectors(collectors);
        let mut computed_only = BTreeSet::new();

        for (name, collectors) in &grouped {
            let non_computed = collectors
                .iter()
                .filter(|collector| collector.source_type != CollectorSource::Computed)
                .cloned()
                .collect::<Vec<_>>();
            if non_computed.is_empty() {
                computed_only.insert(name.clone());
                continue;
            }
            if let Some(value) = resolve_dimension(&non_computed, &dimensions, &input) {
                dimensions.insert(name.clone(), value);
            }
        }

        for name in computed_order(&grouped, &computed_only) {
            let computed = grouped
                .get(&name)
                .into_iter()
                .flatten()
                .filter(|collector| collector.source_type == CollectorSource::Computed)
                .cloned()
                .collect::<Vec<_>>();
            if !computed.is_empty()
                && let Some(value) = resolve_computed_dimension(&computed, &dimensions)
            {
                dimensions.insert(name, value);
            }
        }
        dimensions
    }
}

fn grouped_collectors(collectors: &[DimensionCollector]) -> BTreeMap<String, Vec<DimensionCollector>> {
    let mut grouped = BTreeMap::<String, Vec<DimensionCollector>>::new();
    for collector in collectors.iter().filter(|collector| collector.is_enabled) {
        grouped.entry(collector.dimension_name.clone()).or_default().push(collector.clone());
    }
    for collectors in grouped.values_mut() {
        collectors.sort_by(|left, right| right.priority.cmp(&left.priority));
    }
    grouped
}

fn resolve_dimension(collectors: &[DimensionCollector], dimensions: &BTreeMap<String, Value>, input: &DimensionCollectInput) -> Option<Value> {
    for collector in collectors {
        let raw = source_value(collector, input);
        let Some(raw) = raw else {
            continue;
        };
        if let Ok(value) = transform_value(collector, raw, dimensions) {
            return Some(value);
        }
    }
    default_value(collectors)
}

fn resolve_computed_dimension(collectors: &[DimensionCollector], dimensions: &BTreeMap<String, Value>) -> Option<Value> {
    for collector in collectors {
        let Some(expression) = collector.transform_expression.as_deref() else {
            continue;
        };
        if let Ok(value) = SafeExpressionEvaluator::eval_decimal(expression, dimensions) {
            return cast_value(&Value::String(value.to_string()), &collector.value_type);
        }
    }
    default_value(collectors)
}

fn source_value(collector: &DimensionCollector, input: &DimensionCollectInput) -> Option<Value> {
    let path = collector.source_path.as_deref()?;
    let source = match collector.source_type {
        CollectorSource::Request => input.request.as_ref(),
        CollectorSource::Response => input.response.as_ref(),
        CollectorSource::Metadata => input.metadata.as_ref(),
        CollectorSource::Computed => None,
    }?;
    nested_value(source, path).cloned()
}

fn transform_value(collector: &DimensionCollector, raw: Value, dimensions: &BTreeMap<String, Value>) -> Result<Value, String> {
    let value = if let Some(expression) = collector.transform_expression.as_deref() {
        let mut env = dimensions.clone();
        env.insert("value".into(), raw);
        Value::String(SafeExpressionEvaluator::eval_decimal(expression, &env)?.to_string())
    } else {
        raw
    };
    cast_value(&value, &collector.value_type).ok_or_else(|| "invalid dimension value".into())
}

fn nested_value<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for part in path.split('.') {
        current = match current {
            Value::Object(map) => map.get(part)?,
            Value::Array(items) => items.get(part.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(current)
}

fn cast_value(value: &Value, value_type: &DimensionValueType) -> Option<Value> {
    match value_type {
        DimensionValueType::String => Some(Value::String(value.as_str().map(str::to_owned).unwrap_or_else(|| value.to_string()))),
        DimensionValueType::Int => numeric(value).map(|number| Value::from(number.trunc() as i64)),
        DimensionValueType::Float => numeric(value).map(Value::from),
    }
}

fn numeric(value: &Value) -> Option<f64> {
    match value {
        Value::Bool(_) => None,
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse::<f64>().ok(),
        _ => None,
    }
}

fn default_value(collectors: &[DimensionCollector]) -> Option<Value> {
    for collector in collectors {
        if let Some(value) = collector.default_value.as_deref()
            && let Some(value) = cast_value(&Value::String(value.into()), &collector.value_type)
        {
            return Some(value);
        }
    }
    None
}

fn computed_order(grouped: &BTreeMap<String, Vec<DimensionCollector>>, computed_only: &BTreeSet<String>) -> Vec<String> {
    let mut deps = BTreeMap::<String, BTreeSet<String>>::new();
    for name in computed_only {
        let mut names = BTreeSet::new();
        for collector in grouped.get(name).into_iter().flatten() {
            if let Some(expression) = collector.transform_expression.as_deref()
                && let Ok(vars) = SafeExpressionEvaluator::variable_names(expression)
            {
                names.extend(vars.into_iter().filter(|var| computed_only.contains(var) && var != name));
            }
        }
        deps.insert(name.clone(), names);
    }
    let mut output = Vec::new();
    let mut remaining = deps;
    while !remaining.is_empty() {
        let ready = remaining
            .iter()
            .filter(|(_, deps)| deps.iter().all(|dep| output.contains(dep)))
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        if ready.is_empty() {
            output.extend(remaining.keys().cloned());
            break;
        }
        for name in ready {
            remaining.remove(&name);
            output.push(name);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn collector_uses_priority_fallback_and_rejects_bool_numbers() {
        let collectors = vec![collector("input_tokens", "usage.bad", 10), collector("input_tokens", "usage.prompt_tokens", 0)];
        let dims = DimensionCollectorRuntime::collect(
            &collectors,
            DimensionCollectInput {
                response: Some(json!({"usage": {"bad": true, "prompt_tokens": 42}})),
                ..Default::default()
            },
        );
        assert_eq!(dims["input_tokens"], 42);
    }

    #[test]
    fn collector_does_not_overwrite_base_dimension_when_value_is_missing() {
        let collectors = vec![collector("input_tokens", "usage.prompt_tokens", 100)];
        let dims = DimensionCollectorRuntime::collect(
            &collectors,
            DimensionCollectInput {
                response: Some(json!({"usage": {}})),
                base_dimensions: BTreeMap::from([("input_tokens".into(), json!(27))]),
                ..Default::default()
            },
        );

        assert_eq!(dims["input_tokens"], 27);
    }

    #[test]
    fn collector_default_value_can_override_missing_base_dimension() {
        let collectors = vec![DimensionCollector {
            default_value: Some("12".into()),
            ..collector("input_tokens", "usage.prompt_tokens", 100)
        }];
        let dims = DimensionCollectorRuntime::collect(
            &collectors,
            DimensionCollectInput {
                response: Some(json!({"usage": {}})),
                base_dimensions: BTreeMap::from([("input_tokens".into(), json!(27))]),
                ..Default::default()
            },
        );

        assert_eq!(dims["input_tokens"], 12);
    }

    fn collector(name: &str, path: &str, priority: i32) -> DimensionCollector {
        DimensionCollector {
            api_format: "openai_chat".into(),
            task_type: "chat".into(),
            dimension_name: name.into(),
            source_type: CollectorSource::Response,
            source_path: Some(path.into()),
            value_type: DimensionValueType::Int,
            transform_expression: None,
            default_value: None,
            priority,
            is_enabled: true,
        }
    }
}

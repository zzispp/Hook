use regex::Regex;
use serde::Deserialize;
use serde_json::Value;

use super::LlmProxyError;

#[derive(Clone, Copy)]
pub(super) struct HeaderRuleBodies<'a> {
    current: &'a Value,
    original: &'a Value,
}

impl<'a> HeaderRuleBodies<'a> {
    pub(super) const fn new(current: &'a Value, original: &'a Value) -> Self {
        Self { current, original }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum HeaderCondition {
    All { all: Vec<HeaderCondition> },
    Any { any: Vec<HeaderCondition> },
    Leaf(HeaderConditionLeaf),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HeaderConditionLeaf {
    path: String,
    op: ConditionOp,
    #[serde(default)]
    value: Option<Value>,
    #[serde(default)]
    source: ConditionSource,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ConditionOp {
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
    StartsWith,
    EndsWith,
    Contains,
    Matches,
    Exists,
    NotExists,
    In,
    TypeIs,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ConditionSource {
    #[default]
    Current,
    Original,
}

pub(super) fn condition_matches(condition: Option<&HeaderCondition>, bodies: HeaderRuleBodies<'_>) -> Result<bool, LlmProxyError> {
    let Some(condition) = condition else {
        return Ok(true);
    };
    evaluate_condition(condition, bodies)
}

fn evaluate_condition(condition: &HeaderCondition, bodies: HeaderRuleBodies<'_>) -> Result<bool, LlmProxyError> {
    match condition {
        HeaderCondition::All { all } => evaluate_all(all, bodies),
        HeaderCondition::Any { any } => evaluate_any(any, bodies),
        HeaderCondition::Leaf(leaf) => evaluate_leaf(leaf, bodies),
    }
}

fn evaluate_all(conditions: &[HeaderCondition], bodies: HeaderRuleBodies<'_>) -> Result<bool, LlmProxyError> {
    for condition in conditions {
        if !evaluate_condition(condition, bodies)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn evaluate_any(conditions: &[HeaderCondition], bodies: HeaderRuleBodies<'_>) -> Result<bool, LlmProxyError> {
    for condition in conditions {
        if evaluate_condition(condition, bodies)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn evaluate_leaf(leaf: &HeaderConditionLeaf, bodies: HeaderRuleBodies<'_>) -> Result<bool, LlmProxyError> {
    let values = resolve_path(condition_root(leaf, bodies), &leaf.path)?;
    match leaf.op {
        ConditionOp::Exists => Ok(!values.is_empty()),
        ConditionOp::NotExists => Ok(values.is_empty()),
        _ => any_value_matches(&values, leaf),
    }
}

fn any_value_matches(values: &[&Value], leaf: &HeaderConditionLeaf) -> Result<bool, LlmProxyError> {
    for value in values {
        if compare_value(value, leaf)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn condition_root<'a>(leaf: &HeaderConditionLeaf, bodies: HeaderRuleBodies<'a>) -> &'a Value {
    match leaf.source {
        ConditionSource::Current => bodies.current,
        ConditionSource::Original => bodies.original,
    }
}

fn compare_value(actual: &Value, leaf: &HeaderConditionLeaf) -> Result<bool, LlmProxyError> {
    match leaf.op {
        ConditionOp::Eq => Ok(actual == expected_value(leaf)?),
        ConditionOp::Neq => Ok(actual != expected_value(leaf)?),
        ConditionOp::Gt => compare_number(actual, leaf, |left, right| left > right),
        ConditionOp::Lt => compare_number(actual, leaf, |left, right| left < right),
        ConditionOp::Gte => compare_number(actual, leaf, |left, right| left >= right),
        ConditionOp::Lte => compare_number(actual, leaf, |left, right| left <= right),
        ConditionOp::StartsWith => compare_string(actual, leaf, |left, right| left.starts_with(right)),
        ConditionOp::EndsWith => compare_string(actual, leaf, |left, right| left.ends_with(right)),
        ConditionOp::Contains => compare_contains(actual, leaf),
        ConditionOp::Matches => compare_matches(actual, leaf),
        ConditionOp::In => compare_in(actual, leaf),
        ConditionOp::TypeIs => compare_type(actual, leaf),
        ConditionOp::Exists | ConditionOp::NotExists => Ok(false),
    }
}

fn expected_value(leaf: &HeaderConditionLeaf) -> Result<&Value, LlmProxyError> {
    leaf.value
        .as_ref()
        .ok_or_else(|| LlmProxyError::InvalidRequest(format!("provider header condition {:?} requires value", leaf.op)))
}

fn compare_number(actual: &Value, leaf: &HeaderConditionLeaf, compare: impl Fn(f64, f64) -> bool) -> Result<bool, LlmProxyError> {
    let Some(actual) = actual.as_f64() else {
        return Ok(false);
    };
    let Some(expected) = expected_value(leaf)?.as_f64() else {
        return Ok(false);
    };
    Ok(compare(actual, expected))
}

fn compare_string(actual: &Value, leaf: &HeaderConditionLeaf, compare: impl Fn(&str, &str) -> bool) -> Result<bool, LlmProxyError> {
    let Some(actual) = actual.as_str() else {
        return Ok(false);
    };
    let Some(expected) = expected_value(leaf)?.as_str() else {
        return Ok(false);
    };
    Ok(compare(actual, expected))
}

fn compare_contains(actual: &Value, leaf: &HeaderConditionLeaf) -> Result<bool, LlmProxyError> {
    let expected = expected_value(leaf)?;
    match (actual, expected) {
        (Value::String(actual), Value::String(expected)) => Ok(actual.contains(expected)),
        (Value::Array(items), expected) => Ok(items.iter().any(|item| item == expected)),
        _ => Ok(false),
    }
}

fn compare_matches(actual: &Value, leaf: &HeaderConditionLeaf) -> Result<bool, LlmProxyError> {
    let Some(actual) = actual.as_str() else {
        return Ok(false);
    };
    let Some(pattern) = expected_value(leaf)?.as_str() else {
        return Ok(false);
    };
    Regex::new(pattern)
        .map(|regex| regex.is_match(actual))
        .map_err(|error| LlmProxyError::InvalidRequest(format!("invalid provider header condition regex: {error}")))
}

fn compare_in(actual: &Value, leaf: &HeaderConditionLeaf) -> Result<bool, LlmProxyError> {
    let Some(expected_values) = expected_value(leaf)?.as_array() else {
        return Ok(false);
    };
    Ok(expected_values.iter().any(|expected| expected == actual))
}

fn compare_type(actual: &Value, leaf: &HeaderConditionLeaf) -> Result<bool, LlmProxyError> {
    let Some(expected) = expected_value(leaf)?.as_str() else {
        return Ok(false);
    };
    Ok(value_type(actual) == expected)
}

fn value_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn resolve_path<'a>(root: &'a Value, path: &str) -> Result<Vec<&'a Value>, LlmProxyError> {
    let mut values = vec![root];
    for segment in path.trim().trim_start_matches('.').split('.') {
        let tokens = parse_segment(segment)?;
        values = resolve_segment(values, &tokens);
    }
    Ok(values)
}

fn parse_segment(segment: &str) -> Result<Vec<PathToken>, LlmProxyError> {
    if segment.is_empty() {
        return Err(LlmProxyError::InvalidRequest("provider header condition path contains an empty segment".into()));
    }
    let (key, mut rest) = segment.split_once('[').map_or((segment, ""), |(key, rest)| (key, rest));
    let mut tokens = key_token(key);
    while !rest.is_empty() {
        let Some((index, next)) = rest.split_once(']') else {
            return Err(LlmProxyError::InvalidRequest(format!(
                "invalid provider header condition path segment: {segment}"
            )));
        };
        tokens.push(index_token(index)?);
        rest = next.strip_prefix('[').unwrap_or(next);
    }
    Ok(tokens)
}

fn key_token(key: &str) -> Vec<PathToken> {
    if key.is_empty() { Vec::new() } else { vec![PathToken::Key(key.to_owned())] }
}

fn index_token(value: &str) -> Result<PathToken, LlmProxyError> {
    if value == "*" {
        return Ok(PathToken::Wildcard);
    }
    value
        .parse::<usize>()
        .map(PathToken::Index)
        .map_err(|error| LlmProxyError::InvalidRequest(format!("invalid provider header condition array index {value:?}: {error}")))
}

fn resolve_segment<'a>(values: Vec<&'a Value>, tokens: &[PathToken]) -> Vec<&'a Value> {
    let mut current = values;
    for token in tokens {
        current = resolve_token(current, token);
    }
    current
}

fn resolve_token<'a>(values: Vec<&'a Value>, token: &PathToken) -> Vec<&'a Value> {
    values.into_iter().flat_map(|value| token_values(value, token)).collect()
}

fn token_values<'a>(value: &'a Value, token: &PathToken) -> Vec<&'a Value> {
    match token {
        PathToken::Key(key) => value.get(key).into_iter().collect(),
        PathToken::Index(index) => value.get(*index).into_iter().collect(),
        PathToken::Wildcard => value.as_array().map(|items| items.iter().collect()).unwrap_or_default(),
    }
}

enum PathToken {
    Key(String),
    Index(usize),
    Wildcard,
}

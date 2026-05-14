use regex::RegexBuilder;
use serde::Deserialize;
use serde_json::Value;

use super::LlmProxyError;
use super::header_condition::{HeaderCondition, HeaderRuleBodies, condition_matches};

const ORIGINAL_PLACEHOLDER: &str = "{{$original}}";

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
enum BodyRule {
    Set {
        path: String,
        value: Value,
        #[serde(default)]
        condition: Option<HeaderCondition>,
    },
    Drop {
        path: String,
        #[serde(default)]
        condition: Option<HeaderCondition>,
    },
    Rename {
        from: String,
        to: String,
        #[serde(default)]
        condition: Option<HeaderCondition>,
    },
    Append {
        path: String,
        value: Value,
        #[serde(default)]
        condition: Option<HeaderCondition>,
    },
    Insert {
        path: String,
        index: usize,
        value: Value,
        #[serde(default)]
        condition: Option<HeaderCondition>,
    },
    RegexReplace {
        path: String,
        pattern: String,
        replacement: String,
        #[serde(default)]
        flags: Option<String>,
        #[serde(default)]
        count: Option<usize>,
        #[serde(default)]
        condition: Option<HeaderCondition>,
    },
    NameStyle {
        path: String,
        style: NameStyle,
        #[serde(default)]
        condition: Option<HeaderCondition>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize)]
enum NameStyle {
    #[serde(rename = "snake_case")]
    SnakeCase,
    #[serde(rename = "camelCase")]
    CamelCase,
    #[serde(rename = "PascalCase")]
    PascalCase,
    #[serde(rename = "kebab-case")]
    KebabCase,
    #[serde(rename = "capitalize")]
    Capitalize,
}

#[derive(Clone, Debug)]
enum PathSegment {
    Key(String),
    Index(usize),
}

pub(super) fn apply_provider_body_rules(body: &mut Value, rules: &Option<Value>, original_body: &Value) -> Result<(), LlmProxyError> {
    let Some(rules) = rules else {
        return Ok(());
    };
    for rule in parse_rules(rules)? {
        apply_rule(body, rule, original_body)?;
    }
    Ok(())
}

fn parse_rules(rules: &Value) -> Result<Vec<BodyRule>, LlmProxyError> {
    serde_json::from_value(rules.clone()).map_err(|error| LlmProxyError::InvalidRequest(format!("invalid provider body_rules: {error}")))
}

fn apply_rule(body: &mut Value, rule: BodyRule, original_body: &Value) -> Result<(), LlmProxyError> {
    let bodies = HeaderRuleBodies::new(body, original_body);
    match rule {
        BodyRule::Set { path, value, condition } => {
            if !condition_matches(condition.as_ref(), bodies)? {
                return Ok(());
            }
            set_at_path(body, &parse_path(&path)?, resolve_rule_value(&value, original_body))
        }
        BodyRule::Drop { path, condition } => {
            if !condition_matches(condition.as_ref(), bodies)? {
                return Ok(());
            }
            drop_at_path(body, &parse_path(&path)?)
        }
        BodyRule::Rename { from, to, condition } => {
            if !condition_matches(condition.as_ref(), bodies)? {
                return Ok(());
            }
            rename_at_path(body, &parse_path(&from)?, &parse_path(&to)?)
        }
        BodyRule::Append { path, value, condition } => {
            if !condition_matches(condition.as_ref(), bodies)? {
                return Ok(());
            }
            append_at_path(body, &parse_path(&path)?, resolve_rule_value(&value, original_body))
        }
        BodyRule::Insert {
            path,
            index,
            value,
            condition,
        } => {
            if !condition_matches(condition.as_ref(), bodies)? {
                return Ok(());
            }
            insert_at_path(body, &parse_path(&path)?, index, resolve_rule_value(&value, original_body))
        }
        BodyRule::RegexReplace {
            path,
            pattern,
            replacement,
            flags,
            count,
            condition,
        } => {
            if !condition_matches(condition.as_ref(), bodies)? {
                return Ok(());
            }
            regex_replace_at_path(body, &parse_path(&path)?, &pattern, &replacement, flags.as_deref(), count)
        }
        BodyRule::NameStyle { path, style, condition } => {
            if !condition_matches(condition.as_ref(), bodies)? {
                return Ok(());
            }
            apply_name_style_at_path(body, &parse_path(&path)?, style)
        }
    }
}

fn parse_path(path: &str) -> Result<Vec<PathSegment>, LlmProxyError> {
    let normalized = path.trim().trim_start_matches('.');
    if normalized.is_empty() {
        return Err(LlmProxyError::InvalidRequest("provider body rule path cannot be blank".into()));
    }
    let mut output = Vec::new();
    for segment in normalized.split('.') {
        parse_segment(segment, &mut output)?;
    }
    Ok(output)
}

fn parse_segment(segment: &str, output: &mut Vec<PathSegment>) -> Result<(), LlmProxyError> {
    if segment.is_empty() {
        return Err(LlmProxyError::InvalidRequest("provider body rule path contains an empty segment".into()));
    }
    let (key, mut rest) = segment.split_once('[').map_or((segment, ""), |(value, suffix)| (value, suffix));
    if !key.is_empty() {
        output.push(PathSegment::Key(key.to_owned()));
    }
    while !rest.is_empty() {
        let Some((index, next)) = rest.split_once(']') else {
            return Err(LlmProxyError::InvalidRequest(format!("invalid provider body rule path segment: {segment}")));
        };
        if index == "*" {
            return Err(LlmProxyError::InvalidRequest("provider body rule path does not support wildcard indexes".into()));
        }
        let index = index
            .parse::<usize>()
            .map_err(|error| LlmProxyError::InvalidRequest(format!("invalid provider body rule array index {index:?}: {error}")))?;
        output.push(PathSegment::Index(index));
        rest = next.strip_prefix('[').unwrap_or(next);
    }
    Ok(())
}

fn set_at_path(root: &mut Value, path: &[PathSegment], value: Value) -> Result<(), LlmProxyError> {
    let (last, parent_path) = path.split_last().expect("path should not be empty");
    let parent = ensure_parent_path_mut(root, parent_path, last)?;
    match last {
        PathSegment::Key(key) => {
            object_mut(parent)?.insert(key.clone(), value);
        }
        PathSegment::Index(index) => {
            let items = array_mut(parent)?;
            let slot = items
                .get_mut(*index)
                .ok_or_else(|| LlmProxyError::InvalidRequest(format!("provider body rule array index out of bounds: {index}")))?;
            *slot = value;
        }
    }
    Ok(())
}

fn ensure_parent_path_mut<'a>(
    root: &'a mut Value,
    path: &[PathSegment],
    terminal: &PathSegment,
) -> Result<&'a mut Value, LlmProxyError> {
    if path.is_empty() {
        return Ok(root);
    }
    let current = ensure_child_mut(root, &path[0], path.get(1).unwrap_or(terminal))?;
    ensure_parent_path_mut(current, &path[1..], terminal)
}

fn drop_at_path(root: &mut Value, path: &[PathSegment]) -> Result<(), LlmProxyError> {
    take_at_path(root, path).map(|_| ())
}

fn rename_at_path(root: &mut Value, from: &[PathSegment], to: &[PathSegment]) -> Result<(), LlmProxyError> {
    let value = take_at_path(root, from)?;
    set_at_path(root, to, value)
}

fn append_at_path(root: &mut Value, path: &[PathSegment], value: Value) -> Result<(), LlmProxyError> {
    array_mut(value_at_path_mut(root, path)?)?.push(value);
    Ok(())
}

fn insert_at_path(root: &mut Value, path: &[PathSegment], index: usize, value: Value) -> Result<(), LlmProxyError> {
    let items = array_mut(value_at_path_mut(root, path)?)?;
    if index > items.len() {
        return Err(LlmProxyError::InvalidRequest(format!("provider body rule array index out of bounds: {index}")));
    }
    items.insert(index, value);
    Ok(())
}

fn regex_replace_at_path(
    root: &mut Value,
    path: &[PathSegment],
    pattern: &str,
    replacement: &str,
    flags: Option<&str>,
    count: Option<usize>,
) -> Result<(), LlmProxyError> {
    let value = value_at_path_mut(root, path)?;
    let text = string_mut(value)?;
    let regex = build_regex(pattern, flags)?;
    *text = match count {
        Some(limit) => regex.replacen(text.as_str(), limit, replacement).to_string(),
        None => regex.replace_all(text.as_str(), replacement).to_string(),
    };
    Ok(())
}

fn apply_name_style_at_path(root: &mut Value, path: &[PathSegment], style: NameStyle) -> Result<(), LlmProxyError> {
    let value = value_at_path_mut(root, path)?;
    let text = string_mut(value)?;
    *text = rewrite_name_style(text, style);
    Ok(())
}

fn value_at_path_mut<'a>(root: &'a mut Value, path: &[PathSegment]) -> Result<&'a mut Value, LlmProxyError> {
    if path.is_empty() {
        return Ok(root);
    }
    let next = child_mut(root, &path[0])?;
    value_at_path_mut(next, &path[1..])
}

fn child_mut<'a>(value: &'a mut Value, segment: &PathSegment) -> Result<&'a mut Value, LlmProxyError> {
    match segment {
        PathSegment::Key(key) => object_mut(value)?
            .get_mut(key)
            .ok_or_else(|| LlmProxyError::InvalidRequest(format!("provider body rule path key not found: {key}"))),
        PathSegment::Index(index) => array_mut(value)?
            .get_mut(*index)
            .ok_or_else(|| LlmProxyError::InvalidRequest(format!("provider body rule array index out of bounds: {index}"))),
    }
}

fn ensure_child_mut<'a>(value: &'a mut Value, segment: &PathSegment, next: &PathSegment) -> Result<&'a mut Value, LlmProxyError> {
    match segment {
        PathSegment::Key(key) => {
            let object = object_mut(value)?;
            let entry = object.entry(key.clone()).or_insert_with(|| default_container(next));
            container_matches(entry, next)?;
            Ok(entry)
        }
        PathSegment::Index(_) => child_mut(value, segment),
    }
}

fn take_at_path(root: &mut Value, path: &[PathSegment]) -> Result<Value, LlmProxyError> {
    let (last, parent_path) = path.split_last().expect("path should not be empty");
    let parent = value_at_path_mut(root, parent_path)?;
    match last {
        PathSegment::Key(key) => object_mut(parent)?
            .remove(key)
            .ok_or_else(|| LlmProxyError::InvalidRequest(format!("provider body rule path key not found: {key}"))),
        PathSegment::Index(index) => {
            let items = array_mut(parent)?;
            if *index >= items.len() {
                return Err(LlmProxyError::InvalidRequest(format!("provider body rule array index out of bounds: {index}")));
            }
            Ok(items.remove(*index))
        }
    }
}

fn object_mut(value: &mut Value) -> Result<&mut serde_json::Map<String, Value>, LlmProxyError> {
    value
        .as_object_mut()
        .ok_or_else(|| LlmProxyError::InvalidRequest("provider body rule path target must be an object".into()))
}

fn array_mut(value: &mut Value) -> Result<&mut Vec<Value>, LlmProxyError> {
    value
        .as_array_mut()
        .ok_or_else(|| LlmProxyError::InvalidRequest("provider body rule path target must be an array".into()))
}

fn string_mut(value: &mut Value) -> Result<&mut String, LlmProxyError> {
    match value {
        Value::String(text) => Ok(text),
        _ => Err(LlmProxyError::InvalidRequest("provider body rule path target must be a string".into())),
    }
}

fn default_container(next: &PathSegment) -> Value {
    match next {
        PathSegment::Key(_) => Value::Object(serde_json::Map::new()),
        PathSegment::Index(_) => Value::Array(Vec::new()),
    }
}

fn container_matches(value: &Value, next: &PathSegment) -> Result<(), LlmProxyError> {
    match next {
        PathSegment::Key(_) if value.is_object() => Ok(()),
        PathSegment::Index(_) if value.is_array() => Ok(()),
        PathSegment::Key(_) => Err(LlmProxyError::InvalidRequest("provider body rule path target must be an object".into())),
        PathSegment::Index(_) => Err(LlmProxyError::InvalidRequest("provider body rule path target must be an array".into())),
    }
}

fn resolve_rule_value(value: &Value, original_body: &Value) -> Value {
    match value {
        Value::String(text) if text == ORIGINAL_PLACEHOLDER => original_body.clone(),
        Value::Array(items) => Value::Array(items.iter().map(|item| resolve_rule_value(item, original_body)).collect()),
        Value::Object(items) => Value::Object(
            items.iter().map(|(key, item)| (key.clone(), resolve_rule_value(item, original_body))).collect(),
        ),
        other => other.clone(),
    }
}

fn build_regex(pattern: &str, flags: Option<&str>) -> Result<regex::Regex, LlmProxyError> {
    let mut builder = RegexBuilder::new(pattern);
    for flag in flags.unwrap_or_default().chars() {
        match flag {
            'i' => builder.case_insensitive(true),
            'm' => builder.multi_line(true),
            's' => builder.dot_matches_new_line(true),
            other => {
                return Err(LlmProxyError::InvalidRequest(format!(
                    "invalid provider body rule regex flag {other:?}, expected i/m/s"
                )))
            }
        };
    }
    builder
        .build()
        .map_err(|error| LlmProxyError::InvalidRequest(format!("invalid provider body rule regex: {error}")))
}

fn rewrite_name_style(input: &str, style: NameStyle) -> String {
    let words = split_words(input);
    match style {
        NameStyle::SnakeCase => words.join("_"),
        NameStyle::KebabCase => words.join("-"),
        NameStyle::CamelCase => camel_case(&words),
        NameStyle::PascalCase => pascal_case(&words),
        NameStyle::Capitalize => capitalize_phrase(&words),
    }
}

fn split_words(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut previous_lowercase = false;
    for ch in input.chars() {
        if !ch.is_alphanumeric() {
            flush_word(&mut current, &mut words);
            previous_lowercase = false;
            continue;
        }
        if ch.is_uppercase() && previous_lowercase {
            flush_word(&mut current, &mut words);
        }
        current.extend(ch.to_lowercase());
        previous_lowercase = ch.is_lowercase() || ch.is_numeric();
    }
    flush_word(&mut current, &mut words);
    words
}

fn flush_word(current: &mut String, words: &mut Vec<String>) {
    if current.is_empty() {
        return;
    }
    words.push(std::mem::take(current));
}

fn camel_case(words: &[String]) -> String {
    let Some((first, rest)) = words.split_first() else {
        return String::new();
    };
    let mut output = first.clone();
    for word in rest {
        output.push_str(&capitalize_word(word));
    }
    output
}

fn pascal_case(words: &[String]) -> String {
    words.iter().map(|word| capitalize_word(word)).collect()
}

fn capitalize_phrase(words: &[String]) -> String {
    capitalize_word(&words.join(" "))
}

fn capitalize_word(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut output = String::new();
    output.extend(first.to_uppercase());
    output.push_str(chars.as_str());
    output
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::apply_provider_body_rules;

    #[test]
    fn provider_body_rules_apply_in_order() {
        let mut body = json!({
            "model": "gpt-5.5",
            "metadata": { "tenant": "hook" },
            "messages": ["hello"],
            "temp_name": "sample_value",
            "remove_me": true
        });
        let original = body.clone();
        let rules = json!([
            {"action": "set", "path": "reasoning.effort", "value": "high"},
            {"action": "append", "path": "messages", "value": "world"},
            {"action": "rename", "from": "temp_name", "to": "renamed_name"},
            {"action": "regex_replace", "path": "renamed_name", "pattern": "_", "replacement": "-"},
            {"action": "name_style", "path": "renamed_name", "style": "camelCase"},
            {"action": "drop", "path": "remove_me"}
        ]);

        apply_provider_body_rules(&mut body, &Some(rules), &original).unwrap();

        assert_eq!(body["reasoning"]["effort"], "high");
        assert_eq!(body["messages"], json!(["hello", "world"]));
        assert_eq!(body["renamed_name"], "sampleValue");
        assert!(body.get("remove_me").is_none());
    }

    #[test]
    fn provider_body_rules_can_use_original_placeholder_and_condition() {
        let mut body = json!({"payload": null, "metadata": {"tenant": "hook"}});
        let original = json!({"model": "gpt-5.5", "metadata": {"tenant": "hook"}});
        let rules = json!([
            {"action": "set", "path": "payload", "value": {"copied": "{{$original}}"}},
            {"action": "set", "path": "metadata.tenant", "value": "updated", "condition": {"path": "metadata.tenant", "op": "eq", "value": "hook"}}
        ]);

        apply_provider_body_rules(&mut body, &Some(rules), &original).unwrap();

        assert_eq!(body["payload"]["copied"], original);
        assert_eq!(body["metadata"]["tenant"], "updated");
    }
}

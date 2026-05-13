use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;
use serde_json::Value;

use super::LlmProxyError;
use super::header_condition::{HeaderCondition, HeaderRuleBodies, condition_matches};

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
enum HeaderRule {
    Set {
        key: String,
        value: String,
        #[serde(default)]
        condition: Option<HeaderCondition>,
    },
    Drop {
        key: String,
        #[serde(default)]
        condition: Option<HeaderCondition>,
    },
    Rename {
        from: String,
        to: String,
        #[serde(default)]
        condition: Option<HeaderCondition>,
    },
}

pub(super) fn apply_provider_header_rules(
    headers: &mut HeaderMap,
    rules: &Option<Value>,
    current_body: &Value,
    original_body: &Value,
) -> Result<(), LlmProxyError> {
    let Some(rules) = rules else {
        return Ok(());
    };
    let bodies = HeaderRuleBodies::new(current_body, original_body);
    for rule in parse_rules(rules)? {
        apply_rule(headers, rule, bodies)?;
    }
    Ok(())
}

fn parse_rules(rules: &Value) -> Result<Vec<HeaderRule>, LlmProxyError> {
    serde_json::from_value(rules.clone()).map_err(|error| LlmProxyError::InvalidRequest(format!("invalid provider header_rules: {error}")))
}

fn apply_rule(headers: &mut HeaderMap, rule: HeaderRule, bodies: HeaderRuleBodies<'_>) -> Result<(), LlmProxyError> {
    match rule {
        HeaderRule::Set { key, value, condition } => set_header(headers, &key, &value, condition, bodies),
        HeaderRule::Drop { key, condition } => drop_header(headers, &key, condition, bodies),
        HeaderRule::Rename { from, to, condition } => rename_header(headers, &from, &to, condition, bodies),
    }
}

fn set_header(headers: &mut HeaderMap, key: &str, value: &str, condition: Option<HeaderCondition>, bodies: HeaderRuleBodies<'_>) -> Result<(), LlmProxyError> {
    if !condition_matches(condition.as_ref(), bodies)? {
        return Ok(());
    }
    headers.insert(header_name(key)?, header_value(key, value)?);
    Ok(())
}

fn drop_header(headers: &mut HeaderMap, key: &str, condition: Option<HeaderCondition>, bodies: HeaderRuleBodies<'_>) -> Result<(), LlmProxyError> {
    if !condition_matches(condition.as_ref(), bodies)? {
        return Ok(());
    }
    headers.remove(header_name(key)?);
    Ok(())
}

fn rename_header(headers: &mut HeaderMap, from: &str, to: &str, condition: Option<HeaderCondition>, bodies: HeaderRuleBodies<'_>) -> Result<(), LlmProxyError> {
    if !condition_matches(condition.as_ref(), bodies)? {
        return Ok(());
    }
    let Some(value) = headers.remove(header_name(from)?) else {
        return Ok(());
    };
    headers.insert(header_name(to)?, value);
    Ok(())
}

fn header_name(value: &str) -> Result<HeaderName, LlmProxyError> {
    HeaderName::from_bytes(value.trim().as_bytes()).map_err(|error| LlmProxyError::InvalidRequest(format!("invalid provider header name {value:?}: {error}")))
}

fn header_value(key: &str, value: &str) -> Result<HeaderValue, LlmProxyError> {
    HeaderValue::from_str(value).map_err(|error| LlmProxyError::InvalidRequest(format!("invalid provider header value for {key:?}: {error}")))
}

#[cfg(test)]
mod tests {
    use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
    use serde_json::json;

    use super::apply_provider_header_rules;

    #[test]
    fn provider_header_rules_apply_in_order() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer provider-key"));

        let body = json!({"model": "gpt-5.5"});
        apply_provider_header_rules(&mut headers, &Some(header_rules()), &body, &body).unwrap();

        assert_eq!(header(&headers, "authorization"), "Bearer overwritten");
        assert_eq!(header(&headers, "x-provider-overwrite-probe"), "enabled");
        assert_eq!(header(&headers, "x-renamed-to"), "moved");
        assert!(!headers.contains_key("x-remove-me"));
        assert!(!headers.contains_key("x-rename-from"));
    }

    #[test]
    fn provider_header_rules_apply_matching_conditions() {
        let mut headers = HeaderMap::new();
        let body = json!({"model": "gpt-5.5", "metadata": {"tenant": "hook"}});
        let rules = json!([
            {"action": "set", "key": "X-Matched", "value": "yes", "condition": {"path": "metadata.tenant", "op": "eq", "value": "hook"}},
            {"action": "set", "key": "X-Skipped", "value": "no", "condition": {"path": "model", "op": "eq", "value": "other"}}
        ]);

        apply_provider_header_rules(&mut headers, &Some(rules), &body, &body).unwrap();

        assert_eq!(header(&headers, "x-matched"), "yes");
        assert!(!headers.contains_key("x-skipped"));
    }

    fn header_rules() -> serde_json::Value {
        json!([
            {"action": "set", "key": "Authorization", "value": "Bearer overwritten"},
            {"action": "set", "key": "X-Provider-Overwrite-Probe", "value": "enabled"},
            {"action": "set", "key": "X-Remove-Me", "value": "temporary"},
            {"action": "drop", "key": "X-Remove-Me"},
            {"action": "set", "key": "X-Rename-From", "value": "moved"},
            {"action": "rename", "from": "X-Rename-From", "to": "X-Renamed-To"}
        ])
    }

    fn header<'a>(headers: &'a HeaderMap, key: &str) -> &'a str {
        headers.get(key).unwrap().to_str().unwrap()
    }
}

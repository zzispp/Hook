use serde_json::Value;

pub(super) fn json_capability_enabled(capabilities: Option<&Value>, required: &str) -> bool {
    let required = required.trim();
    if required.is_empty() {
        return true;
    }
    let Some(capabilities) = capabilities else {
        return false;
    };
    if let Some(object) = capabilities.as_object() {
        return object
            .iter()
            .any(|(key, value)| key.eq_ignore_ascii_case(required) && capability_value_enabled(value));
    }
    if let Some(items) = capabilities.as_array() {
        return items
            .iter()
            .any(|value| value.as_str().is_some_and(|value| value.eq_ignore_ascii_case(required)));
    }
    false
}

pub(super) fn capability_list_enabled(capabilities: Option<&[String]>, required: &str) -> bool {
    let required = required.trim();
    capabilities.is_some_and(|items| items.iter().any(|value| value.eq_ignore_ascii_case(required)))
}

fn capability_value_enabled(value: &Value) -> bool {
    match value {
        Value::Bool(value) => *value,
        Value::String(value) => value.eq_ignore_ascii_case("true"),
        Value::Number(value) => value.as_i64().is_some_and(|value| value > 0),
        _ => false,
    }
}

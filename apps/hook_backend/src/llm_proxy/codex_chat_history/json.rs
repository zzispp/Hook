use serde_json::Value;

const ENRICHED_CALL_FIELDS: [&str; 5] = ["name", "arguments", "status", "reasoning", "reasoning_content"];

pub(super) fn call_item_needs_cache(item: &Value) -> bool {
    ENRICHED_CALL_FIELDS.iter().any(|key| !item.get(*key).is_some_and(is_present_value))
}

pub(super) fn enrich_call_item_from_cache(item: &mut Value, cached: &Value) -> bool {
    let mut changed = false;
    for key in ENRICHED_CALL_FIELDS {
        if item.get(key).is_some_and(is_present_value) {
            continue;
        }
        let Some(value) = cached.get(key).filter(|value| is_present_value(value)) else {
            continue;
        };
        if let Some(object) = item.as_object_mut() {
            object.insert(key.to_owned(), value.clone());
            changed = true;
        }
    }
    changed
}

fn is_present_value(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(value) => !value.trim().is_empty(),
        Value::Array(values) => !values.is_empty(),
        Value::Object(values) => !values.is_empty(),
        Value::Bool(_) | Value::Number(_) => true,
    }
}

use serde_json::{Map, Value};

use super::record::TranslationEntryRecord;

pub fn resource_json(records: Vec<TranslationEntryRecord>) -> Value {
    let mut root = Map::new();
    for record in records {
        insert_resource_value(&mut root, &record.group_key, &record.item_key, record.value);
    }
    normalize_array_markers(Value::Object(root))
}

fn insert_resource_value(root: &mut Map<String, Value>, group_key: &str, item_key: &str, value: String) {
    let group = root.entry(group_key.to_owned()).or_insert_with(|| Value::Object(Map::new()));
    if let Value::Object(group_map) = group {
        let parts = item_key.split('.').collect::<Vec<_>>();
        insert_nested_value(group_map, &parts, value);
    }
}

fn insert_nested_value(map: &mut Map<String, Value>, parts: &[&str], value: String) {
    let Some((part, rest)) = parts.split_first() else {
        return;
    };
    if let Some(index) = array_index(part) {
        insert_array_value(map, index, rest, value);
        return;
    }
    if !rest.is_empty() {
        let child = map.entry(part.to_owned()).or_insert_with(|| Value::Object(Map::new()));
        if let Value::Object(child_map) = child {
            insert_nested_value(child_map, rest, value);
        }
        return;
    }
    map.insert((*part).to_owned(), Value::String(value));
}

fn insert_array_value(map: &mut Map<String, Value>, index: usize, rest: &[&str], value: String) {
    let array = map.entry("_array".to_owned()).or_insert_with(|| Value::Array(Vec::new()));
    if let Value::Array(items) = array {
        resize_array(items, index);
        items[index] = array_item_value(rest, value);
    }
}

fn array_item_value(rest: &[&str], value: String) -> Value {
    if rest.is_empty() {
        return Value::String(value);
    }
    let mut child = Map::new();
    insert_nested_value(&mut child, rest, value);
    normalize_array_markers(Value::Object(child))
}

fn resize_array(items: &mut Vec<Value>, index: usize) {
    while items.len() <= index {
        items.push(Value::Null);
    }
}

fn normalize_array_markers(value: Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.into_iter().map(normalize_array_markers).collect()),
        Value::Object(map) => normalize_object(map),
        other => other,
    }
}

fn normalize_object(mut map: Map<String, Value>) -> Value {
    if map.len() == 1 && map.contains_key("_array") {
        return normalize_array_markers(map.remove("_array").unwrap_or(Value::Null));
    }
    Value::Object(map.into_iter().map(|(key, value)| (key, normalize_array_markers(value))).collect())
}

fn array_index(part: &str) -> Option<usize> {
    part.parse::<usize>().ok()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn resource_json_rebuilds_string_arrays() {
        let records = vec![
            record("dashboard", "months.0", "Jan"),
            record("dashboard", "months.1", "Feb"),
            record("dashboard", "welcome", "Welcome"),
        ];

        assert_eq!(
            resource_json(records),
            json!({
                "dashboard": {
                    "months": ["Jan", "Feb"],
                    "welcome": "Welcome"
                }
            })
        );
    }

    fn record(group_key: &str, item_key: &str, value: &str) -> TranslationEntryRecord {
        TranslationEntryRecord {
            id: format!("{group_key}-{item_key}"),
            namespace: "admin".into(),
            group_key: group_key.into(),
            item_key: item_key.into(),
            lang_code: "en".into(),
            value: value.into(),
            description: None,
            enabled: true,
            created_at: time::OffsetDateTime::UNIX_EPOCH,
            updated_at: time::OffsetDateTime::UNIX_EPOCH,
        }
    }
}

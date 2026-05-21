use serde_json::{Map, Value};

const CONSTRAINTS: &[(&str, &str)] = &[
    ("minLength", "minLen"),
    ("maxLength", "maxLen"),
    ("pattern", "pattern"),
    ("minimum", "min"),
    ("maximum", "max"),
    ("multipleOf", "multipleOf"),
    ("exclusiveMinimum", "exclMin"),
    ("exclusiveMaximum", "exclMax"),
    ("minItems", "minItems"),
    ("maxItems", "maxItems"),
    ("format", "format"),
];

pub(super) fn move_constraints(object: &mut Map<String, Value>) {
    let hints = CONSTRAINTS
        .iter()
        .filter_map(|(field, label)| object.get(*field).map(|value| format!("{label}: {value}")))
        .collect::<Vec<_>>();
    if !hints.is_empty() {
        append_hint(object, &format!("[Constraint: {}]", hints.join(", ")));
    }
}

pub(super) fn align_required(object: &mut Map<String, Value>) {
    let valid_keys = object
        .get("properties")
        .and_then(Value::as_object)
        .map(|properties| properties.keys().cloned().collect::<Vec<_>>());
    if let Some(Value::Array(required)) = object.get_mut("required") {
        match valid_keys {
            Some(valid) => required.retain(|item| item.as_str().is_some_and(|key| valid.iter().any(|valid_key| valid_key == key))),
            None => required.clear(),
        }
        if required.is_empty() {
            object.remove("required");
        }
    }
}

pub(super) fn infer_type(object: &mut Map<String, Value>) {
    if object.contains_key("type") {
        return;
    }
    let inferred = if object.contains_key("enum") {
        "string"
    } else if object.contains_key("properties") {
        "object"
    } else if object.contains_key("items") {
        "array"
    } else {
        return;
    };
    object.insert("type".into(), Value::String(inferred.into()));
}

pub(super) fn normalize_type(object: &mut Map<String, Value>) -> bool {
    let fallback = if object.contains_key("properties") {
        "object"
    } else if object.contains_key("items") {
        "array"
    } else {
        "string"
    };
    let Some(value) = object.get("type") else { return false };
    let (selected, nullable) = select_type(value, fallback);
    object.insert("type".into(), Value::String(selected));
    nullable
}

fn select_type(value: &Value, fallback: &str) -> (String, bool) {
    match value {
        Value::String(text) if text.eq_ignore_ascii_case("null") => (fallback.to_owned(), true),
        Value::String(text) => (text.to_ascii_lowercase(), false),
        Value::Array(items) => select_type_from_array(items, fallback),
        _ => (fallback.to_owned(), false),
    }
}

fn select_type_from_array(items: &[Value], fallback: &str) -> (String, bool) {
    let mut selected = None;
    let mut nullable = false;
    for item in items {
        let Some(text) = item.as_str() else { continue };
        if text.eq_ignore_ascii_case("null") {
            nullable = true;
        } else if selected.is_none() {
            selected = Some(text.to_ascii_lowercase());
        }
    }
    (selected.unwrap_or_else(|| fallback.to_owned()), nullable)
}

pub(super) fn stringify_enum(object: &mut Map<String, Value>) {
    let Some(Value::Array(items)) = object.get_mut("enum") else { return };
    for item in items {
        if !item.is_string() {
            let text = if item.is_null() { "null".into() } else { item.to_string() };
            *item = Value::String(text);
        }
    }
}

pub(super) fn ensure_empty_object_properties(object: &mut Map<String, Value>) {
    if object.get("type").and_then(Value::as_str) == Some("object") && !object.contains_key("properties") {
        object.insert("properties".into(), Value::Object(Map::new()));
    }
}

pub(super) fn remove_nullable_required(object: &mut Map<String, Value>, nullable_keys: &[String]) {
    if let Some(Value::Array(required)) = object.get_mut("required") {
        required.retain(|item| item.as_str().is_none_or(|key| !nullable_keys.iter().any(|nullable| nullable == key)));
        if required.is_empty() {
            object.remove("required");
        }
    }
}

pub(super) fn clean_payload_children(object: &mut Map<String, Value>) {
    for key in object.keys().cloned().collect::<Vec<_>>() {
        if ["anyOf", "oneOf", "allOf", "enum", "type"].contains(&key.as_str()) {
            continue;
        }
        if let Some(value) = object.get_mut(&key).filter(|value| value.is_object() || value.is_array()) {
            super::gemini::clean_nested_value(value);
        }
    }
}

pub(super) fn branch_score(value: &Value) -> u8 {
    let Some(object) = value.as_object() else { return 0 };
    if object.contains_key("properties") || object.get("type").and_then(Value::as_str) == Some("object") {
        3
    } else if object.contains_key("items") || object.get("type").and_then(Value::as_str) == Some("array") {
        2
    } else if object.get("type").and_then(Value::as_str).is_some_and(|value| value != "null") {
        1
    } else {
        0
    }
}

pub(super) fn merge_object_field(target: &mut Map<String, Value>, source: &mut Map<String, Value>, key: &str) {
    let Some(Value::Object(source_object)) = source.remove(key) else { return };
    let target_value = target.entry(key).or_insert_with(|| Value::Object(Map::new()));
    if let Value::Object(target_object) = target_value {
        for (name, value) in source_object {
            target_object.entry(name).or_insert(value);
        }
    }
}

pub(super) fn merge_array_field(target: &mut Map<String, Value>, source: &mut Map<String, Value>, key: &str) {
    let Some(Value::Array(source_array)) = source.remove(key) else { return };
    let target_value = target.entry(key).or_insert_with(|| Value::Array(Vec::new()));
    if let Value::Array(target_array) = target_value {
        for value in source_array {
            if !target_array.contains(&value) {
                target_array.push(value);
            }
        }
    }
}

pub(super) fn append_hint(object: &mut Map<String, Value>, hint: &str) {
    let current = object.get("description").and_then(Value::as_str).unwrap_or_default();
    if current.contains(hint) {
        return;
    }
    let description = if current.is_empty() { hint.to_owned() } else { format!("{current} {hint}") };
    object.insert("description".into(), Value::String(description));
}

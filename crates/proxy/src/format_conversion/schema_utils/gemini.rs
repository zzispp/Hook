use serde_json::{Map, Value};

use super::gemini_clean::{
    align_required, append_hint, branch_score, clean_payload_children, ensure_empty_object_properties, infer_type, merge_array_field, merge_object_field,
    move_constraints, normalize_type, remove_nullable_required, stringify_enum,
};

const ALLOWED: &[&str] = &["type", "description", "properties", "required", "items", "enum", "title"];

pub(crate) fn clean_gemini_schema(value: &Value) -> Value {
    let mut cloned = value.clone();
    let defs = collect_defs(&cloned);
    flatten_refs(&mut cloned, &defs, &mut Vec::new());
    clean_value(&mut cloned, true);
    cloned
}

pub(super) fn clean_nested_value(value: &mut Value) {
    clean_value(value, false);
}

fn clean_value(value: &mut Value, is_schema: bool) -> bool {
    let Value::Object(object) = value else {
        if let Value::Array(items) = value {
            items.iter_mut().for_each(|item| {
                clean_value(item, is_schema);
            });
        }
        return false;
    };
    merge_all_of(object);
    object_items_to_properties(object);
    let mut nullable = clean_children(object);
    let accepted_types = fold_union(object);
    if !schema_like(object, is_schema) {
        return nullable;
    }
    move_constraints(object);
    object.retain(|key, _| ALLOWED.contains(&key.as_str()));
    ensure_empty_object_properties(object);
    align_required(object);
    infer_type(object);
    nullable |= normalize_type(object);
    if accepted_types.len() > 1 {
        append_hint(object, &format!("Accepts: {}", accepted_types.join(" | ")));
    }
    if nullable {
        append_hint(object, "(nullable)");
    }
    stringify_enum(object);
    nullable
}

fn collect_defs(value: &Value) -> Map<String, Value> {
    let mut defs = Map::new();
    match value {
        Value::Object(object) => {
            collect_object_defs(object, &mut defs);
            for (key, child) in object {
                if key != "$defs" && key != "definitions" {
                    defs.extend(collect_defs(child));
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|item| defs.extend(collect_defs(item))),
        _ => {}
    }
    defs
}

fn collect_object_defs(object: &Map<String, Value>, defs: &mut Map<String, Value>) {
    for key in ["$defs", "definitions"] {
        if let Some(Value::Object(items)) = object.get(key) {
            for (name, schema) in items {
                defs.entry(name.clone()).or_insert_with(|| schema.clone());
            }
        }
    }
}

fn flatten_refs(value: &mut Value, defs: &Map<String, Value>, seen: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            if let Some(Value::String(path)) = object.remove("$ref") {
                merge_ref(object, defs, seen, path);
                flatten_refs(value, defs, seen);
                return;
            }
            object.values_mut().for_each(|child| flatten_refs(child, defs, seen));
        }
        Value::Array(items) => items.iter_mut().for_each(|item| flatten_refs(item, defs, seen)),
        _ => {}
    }
}

fn merge_ref(object: &mut Map<String, Value>, defs: &Map<String, Value>, seen: &mut Vec<String>, path: String) {
    let name = path.rsplit('/').next().unwrap_or_default().to_owned();
    if seen.contains(&name) {
        object.entry("type").or_insert(Value::String("string".into()));
        append_hint(object, &format!("(Circular $ref: {path})"));
        return;
    }
    let Some(Value::Object(definition)) = defs.get(&name) else {
        object.entry("type").or_insert(Value::String("string".into()));
        append_hint(object, &format!("(Unresolved $ref: {path})"));
        return;
    };
    seen.push(name.clone());
    for (key, item) in definition {
        object.entry(key.clone()).or_insert_with(|| item.clone());
    }
    seen.retain(|item| item != &name);
}

fn merge_all_of(object: &mut Map<String, Value>) {
    let Some(Value::Array(items)) = object.remove("allOf") else { return };
    for item in items {
        if let Value::Object(mut schema) = item {
            merge_object_field(object, &mut schema, "properties");
            merge_array_field(object, &mut schema, "required");
            for (key, value) in schema {
                if key != "properties" && key != "required" && key != "allOf" {
                    object.entry(key).or_insert(value);
                }
            }
        }
    }
}

fn object_items_to_properties(object: &mut Map<String, Value>) {
    if !(object.get("type").and_then(Value::as_str) == Some("object") || object.contains_key("properties")) {
        return;
    }
    let Some(Value::Object(items)) = object.remove("items") else { return };
    let properties = object.entry("properties").or_insert_with(|| Value::Object(Map::new()));
    if let Value::Object(properties) = properties {
        properties.extend(items);
    }
}

fn clean_children(object: &mut Map<String, Value>) -> bool {
    let mut nullable_keys = Vec::new();
    if let Some(Value::Object(properties)) = object.get_mut("properties") {
        for (key, value) in properties {
            if clean_value(value, true) {
                nullable_keys.push(key.clone());
            }
        }
        object.entry("type").or_insert(Value::String("object".into()));
    }
    remove_nullable_required(object, &nullable_keys);
    if let Some(items) = object.get_mut("items") {
        if items.is_object() {
            clean_value(items, true);
            object.entry("type").or_insert(Value::String("array".into()));
        }
    }
    if !object.contains_key("properties") && !object.contains_key("items") {
        clean_payload_children(object);
    }
    false
}

fn fold_union(object: &mut Map<String, Value>) -> Vec<String> {
    clean_union_branches(object);
    if object.get("type").is_some_and(|value| value != "object") {
        object.remove("anyOf");
        object.remove("oneOf");
        return Vec::new();
    }
    let union = object.remove("anyOf").or_else(|| object.remove("oneOf"));
    let Some(Value::Array(items)) = union else { return Vec::new() };
    let accepted_types = accepted_types(&items);
    if let Some(Value::Object(best)) = items.into_iter().max_by_key(branch_score) {
        merge_best_branch(object, best);
    }
    accepted_types
}

fn clean_union_branches(object: &mut Map<String, Value>) {
    for key in ["anyOf", "oneOf"] {
        if let Some(Value::Array(items)) = object.get_mut(key) {
            items.iter_mut().filter(|item| item.is_object()).for_each(|item| {
                clean_value(item, true);
            });
        }
    }
}

fn schema_like(object: &mut Map<String, Value>, is_schema: bool) -> bool {
    if object.contains_key("functionCall") || object.contains_key("functionResponse") {
        return false;
    }
    let has_standard = object.keys().any(|key| ALLOWED.contains(&key.as_str()));
    if is_schema && !has_standard && !object.is_empty() {
        let properties = std::mem::take(object);
        object.insert("type".into(), Value::String("object".into()));
        object.insert("properties".into(), Value::Object(properties));
        if let Some(Value::Object(properties)) = object.get_mut("properties") {
            properties.values_mut().for_each(|value| {
                clean_value(value, true);
            });
        }
        return true;
    }
    is_schema || has_standard
}

fn accepted_types(items: &[Value]) -> Vec<String> {
    let mut output = Vec::new();
    for item in items {
        let Some(name) = type_name(item) else { continue };
        if !output.iter().any(|existing| existing == &name) {
            output.push(name);
        }
    }
    output
}

fn type_name(value: &Value) -> Option<String> {
    let object = value.as_object()?;
    if let Some(text) = object.get("type").and_then(Value::as_str) {
        return Some(text.to_owned());
    }
    if object.contains_key("properties") {
        return Some("object".into());
    }
    if object.contains_key("items") {
        return Some("array".into());
    }
    None
}

fn merge_best_branch(object: &mut Map<String, Value>, mut best: Map<String, Value>) {
    merge_object_field(object, &mut best, "properties");
    merge_array_field(object, &mut best, "required");
    for (key, value) in best {
        object.entry(key).or_insert(value);
    }
}

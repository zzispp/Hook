use serde_json::{Map, Value};

pub(crate) fn openai_schema_with_object_fixes(value: &Value) -> Value {
    let mut cloned = value.clone();
    ensure_object_properties(&mut cloned);
    cloned
}

fn ensure_object_properties(value: &mut Value) {
    match value {
        Value::Object(object) => {
            if type_includes_object(object.get("type")) && !object.get("properties").is_some_and(Value::is_object) {
                object.insert("properties".into(), Value::Object(Map::new()));
            }
            object.values_mut().for_each(ensure_object_properties);
        }
        Value::Array(items) => items.iter_mut().for_each(ensure_object_properties),
        _ => {}
    }
}

fn type_includes_object(value: Option<&Value>) -> bool {
    match value {
        Some(Value::String(text)) => text.eq_ignore_ascii_case("object"),
        Some(Value::Array(items)) => items.iter().any(|item| item.as_str().is_some_and(|text| text.eq_ignore_ascii_case("object"))),
        _ => false,
    }
}

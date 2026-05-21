use serde_json::Value;

pub(super) fn compact_contents(contents: Vec<Value>) -> Vec<Value> {
    let mut output: Vec<Value> = Vec::new();
    for content in contents.into_iter().filter(has_valid_parts) {
        if merge_same_role(&mut output, &content) {
            continue;
        }
        output.push(content);
    }
    output
}

fn has_valid_parts(content: &Value) -> bool {
    content
        .get("parts")
        .and_then(Value::as_array)
        .is_some_and(|parts| parts.iter().any(is_valid_part))
}

fn is_valid_part(part: &Value) -> bool {
    let Some(object) = part.as_object() else {
        return false;
    };
    ["text", "inlineData", "functionCall", "functionResponse", "fileData"]
        .iter()
        .any(|key| object.contains_key(*key))
}

fn merge_same_role(output: &mut [Value], content: &Value) -> bool {
    let Some(previous) = output.last_mut() else {
        return false;
    };
    if previous.get("role") != content.get("role") {
        return false;
    }
    let (Some(previous_parts), Some(parts)) = (
        previous.get_mut("parts").and_then(Value::as_array_mut),
        content.get("parts").and_then(Value::as_array),
    ) else {
        return false;
    };
    previous_parts.extend(parts.iter().filter(|part| is_valid_part(part)).cloned());
    true
}

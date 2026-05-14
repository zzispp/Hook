use serde_json::Value;

use super::LlmProxyError;

pub(super) fn rewrite_response_model_bytes(bytes: &[u8], requested_model_name: &str) -> Result<Vec<u8>, LlmProxyError> {
    let mut value: Value = serde_json::from_slice(bytes).map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
    rewrite_response_model_value(&mut value, requested_model_name);
    serde_json::to_vec(&value).map_err(|error| LlmProxyError::Infrastructure(error.to_string()))
}

pub(super) fn rewrite_response_model_value(value: &mut Value, requested_model_name: &str) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    rewrite_key(object, "model", requested_model_name);
    rewrite_key(object, "modelVersion", requested_model_name);
    if let Some(response) = object.get_mut("response").and_then(Value::as_object_mut) {
        rewrite_key(response, "model", requested_model_name);
    }
}

fn rewrite_key(object: &mut serde_json::Map<String, Value>, key: &str, requested_model_name: &str) {
    if object.contains_key(key) {
        object.insert(key.to_owned(), Value::String(requested_model_name.to_owned()));
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::rewrite_response_model_value;

    #[test]
    fn response_model_rewrite_updates_top_level_and_nested_fields() {
        let mut value = json!({
            "model": "upstream-model",
            "modelVersion": "gemini-upstream",
            "response": { "model": "responses-upstream" }
        });

        rewrite_response_model_value(&mut value, "gpt-5.5");

        assert_eq!(value["model"], "gpt-5.5");
        assert_eq!(value["modelVersion"], "gpt-5.5");
        assert_eq!(value["response"]["model"], "gpt-5.5");
    }
}

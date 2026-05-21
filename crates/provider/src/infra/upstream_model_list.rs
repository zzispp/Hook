use req::Url;
use serde_json::Value;

use crate::application::{ProviderError, ProviderResult};

pub fn extract_model_names(value: &Value, api_format: &str) -> Vec<String> {
    let mut names = std::collections::BTreeSet::new();
    for item in model_items(value) {
        if let Some(name) = model_name(item, api_format) {
            names.insert(name);
        }
    }
    names.into_iter().collect()
}

pub fn openai_models_url(base_url: &str) -> ProviderResult<Url> {
    parsed_url(models_url(base_url, "/v1"))
}

pub fn gemini_models_url(base_url: &str, api_key: &str) -> ProviderResult<Url> {
    let mut url = parsed_url(models_url(base_url, "/v1beta"))?;
    url.query_pairs_mut().append_pair("key", api_key);
    Ok(url)
}

fn model_items(value: &Value) -> Vec<&Value> {
    match value {
        Value::Array(items) => items.iter().collect(),
        Value::Object(map) => map
            .get("data")
            .or_else(|| map.get("models"))
            .and_then(Value::as_array)
            .map(|items| items.iter().collect())
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn model_name(item: &Value, api_format: &str) -> Option<String> {
    match item {
        Value::String(value) => normalize_model_name(value, api_format),
        Value::Object(map) => map
            .get("id")
            .and_then(Value::as_str)
            .or_else(|| map.get("name").and_then(Value::as_str))
            .and_then(|value| normalize_model_name(value, api_format)),
        _ => None,
    }
}

fn normalize_model_name(value: &str, api_format: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let normalized = match api_format {
        "gemini:chat" | "gemini:cli" => trimmed.rsplit('/').next().unwrap_or(trimmed),
        _ => trimmed,
    };
    Some(normalized.to_owned())
}

fn models_url(base_url: &str, version_path: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    if trimmed.ends_with(version_path) {
        return format!("{trimmed}/models");
    }
    format!("{trimmed}{version_path}/models")
}

fn parsed_url(value: String) -> ProviderResult<Url> {
    Url::parse(&value).map_err(|error| ProviderError::InvalidInput(format!("invalid provider base_url: {error}")))
}

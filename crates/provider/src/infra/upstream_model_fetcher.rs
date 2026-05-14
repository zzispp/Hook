use std::{collections::BTreeSet, time::Duration};

use async_trait::async_trait;
use reqwest::{Response, Url};
use serde_json::Value;
use types::provider::{ProviderEndpoint, ProviderUpstreamModelsResponse};

use crate::application::{ProviderError, ProviderResult, UpstreamModelFetcher};

const ANTHROPIC_VERSION: &str = "2023-06-01";
const FETCH_TIMEOUT_SECONDS: u64 = 30;
const MAX_ERROR_BODY_CHARS: usize = 300;

#[derive(Clone)]
pub struct ReqwestUpstreamModelFetcher {
    http: reqwest::Client,
}

impl ReqwestUpstreamModelFetcher {
    pub fn new() -> ProviderResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(FETCH_TIMEOUT_SECONDS))
            .build()
            .map_err(reqwest_error)?;
        Ok(Self { http })
    }
}

#[async_trait]
impl UpstreamModelFetcher for ReqwestUpstreamModelFetcher {
    async fn fetch_upstream_models(&self, endpoint: &ProviderEndpoint, api_key: &str) -> ProviderResult<ProviderUpstreamModelsResponse> {
        let request = build_request(&self.http, endpoint, api_key)?;
        let response = self.http.execute(request).await.map_err(reqwest_error)?;
        parse_models_response(response, &endpoint.api_format).await
    }
}

fn build_request(client: &reqwest::Client, endpoint: &ProviderEndpoint, api_key: &str) -> ProviderResult<reqwest::Request> {
    match endpoint.api_format.as_str() {
        "openai_chat" | "openai_cli" | "openai_compact" => client
            .get(openai_models_url(&endpoint.base_url)?)
            .bearer_auth(api_key)
            .build()
            .map_err(reqwest_error),
        "claude_chat" | "claude_messages" => client
            .get(openai_models_url(&endpoint.base_url)?)
            .header("x-api-key", api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .build()
            .map_err(reqwest_error),
        "gemini_chat" | "gemini_cli" => client.get(gemini_models_url(&endpoint.base_url, api_key)?).build().map_err(reqwest_error),
        other => Err(ProviderError::InvalidInput(format!(
            "api_format does not support upstream model fetch: {other}"
        ))),
    }
}

async fn parse_models_response(response: Response, api_format: &str) -> ProviderResult<ProviderUpstreamModelsResponse> {
    let status = response.status();
    let text = response.text().await.map_err(reqwest_error)?;
    if !status.is_success() {
        return Err(ProviderError::Infrastructure(format!("upstream returned {status}: {}", clipped_text(&text))));
    }
    let value = if text.trim().is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&text).map_err(json_error)?
    };
    Ok(ProviderUpstreamModelsResponse {
        models: extract_model_names(&value, api_format),
    })
}

fn extract_model_names(value: &Value, api_format: &str) -> Vec<String> {
    let mut names = BTreeSet::new();
    for item in model_items(value) {
        if let Some(name) = model_name(item, api_format) {
            names.insert(name);
        }
    }
    names.into_iter().collect()
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
        "gemini_chat" | "gemini_cli" => trimmed.rsplit('/').next().unwrap_or(trimmed),
        _ => trimmed,
    };
    Some(normalized.to_owned())
}

fn openai_models_url(base_url: &str) -> ProviderResult<Url> {
    parsed_url(models_url(base_url, "/v1"))
}

fn gemini_models_url(base_url: &str, api_key: &str) -> ProviderResult<Url> {
    let mut url = parsed_url(models_url(base_url, "/v1beta"))?;
    url.query_pairs_mut().append_pair("key", api_key);
    Ok(url)
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

fn clipped_text(value: &str) -> String {
    let clipped = value.chars().take(MAX_ERROR_BODY_CHARS).collect::<String>();
    if clipped.is_empty() { "(empty)".into() } else { clipped }
}

fn reqwest_error(error: reqwest::Error) -> ProviderError {
    ProviderError::Infrastructure(error.to_string())
}

fn json_error(error: serde_json::Error) -> ProviderError {
    ProviderError::Infrastructure(format!("invalid upstream models payload: {error}"))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::extract_model_names;

    #[test]
    fn extract_model_names_supports_openai_payloads() {
        let value = json!({
            "data": [
                { "id": "gpt-5.4" },
                { "id": "gpt-5.4-mini" }
            ]
        });

        assert_eq!(extract_model_names(&value, "openai_chat"), vec!["gpt-5.4", "gpt-5.4-mini"]);
    }

    #[test]
    fn extract_model_names_supports_gemini_payloads() {
        let value = json!({
            "models": [
                { "name": "models/gemini-2.5-pro" },
                { "name": "models/gemini-2.5-flash" }
            ]
        });

        assert_eq!(extract_model_names(&value, "gemini_chat"), vec!["gemini-2.5-flash", "gemini-2.5-pro"]);
    }
}

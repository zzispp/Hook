use std::time::Duration;

use async_trait::async_trait;
use req::{Request, ReqwestClient, Response};
use serde_json::Value;
use types::provider::{ProviderEndpoint, ProviderUpstreamModelsResponse};

use crate::application::{ProviderError, ProviderResult, UpstreamModelFetcher};

use super::upstream_model_list::{extract_model_names, gemini_models_url, openai_models_url};

const ANTHROPIC_VERSION: &str = "2023-06-01";
const FETCH_TIMEOUT_SECONDS: u64 = 30;
const MAX_ERROR_BODY_CHARS: usize = 300;

#[derive(Clone)]
pub struct ReqwestUpstreamModelFetcher {
    http: ReqwestClient,
}

impl ReqwestUpstreamModelFetcher {
    pub fn new() -> ProviderResult<Self> {
        let http = ReqwestClient::from_builder(req::builder().timeout(Duration::from_secs(FETCH_TIMEOUT_SECONDS))).map_err(client_error)?;
        Ok(Self { http })
    }
}

#[async_trait]
impl UpstreamModelFetcher for ReqwestUpstreamModelFetcher {
    async fn fetch_upstream_models(&self, endpoint: &ProviderEndpoint, api_key: &str) -> ProviderResult<ProviderUpstreamModelsResponse> {
        let request = build_request(&self.http, endpoint, api_key)?;
        let response = self.http.execute(request).await.map_err(client_error)?;
        parse_models_response(response, &endpoint.api_format).await
    }
}

fn build_request(client: &ReqwestClient, endpoint: &ProviderEndpoint, api_key: &str) -> ProviderResult<Request> {
    let request = match endpoint.api_format.as_str() {
        "openai:chat" | "openai:cli" | "openai:compact" => client.get(openai_models_url(&endpoint.base_url)?).bearer_auth(api_key),
        "claude:chat" => client
            .get(openai_models_url(&endpoint.base_url)?)
            .header("x-api-key", api_key)
            .header("anthropic-version", ANTHROPIC_VERSION),
        "gemini:chat" | "gemini:cli" => client.get(gemini_models_url(&endpoint.base_url, api_key)?),
        other => Err(ProviderError::InvalidInput(format!(
            "api_format does not support upstream model fetch: {other}"
        )))?,
    };
    client.build_request(request).map_err(client_error)
}

async fn parse_models_response(response: Response, api_format: &str) -> ProviderResult<ProviderUpstreamModelsResponse> {
    let status = response.status();
    let text = req::response_text(response).await.map_err(client_error)?;
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

fn clipped_text(value: &str) -> String {
    let clipped = value.chars().take(MAX_ERROR_BODY_CHARS).collect::<String>();
    if clipped.is_empty() { "(empty)".into() } else { clipped }
}

fn client_error(error: req::ClientError) -> ProviderError {
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

        assert_eq!(extract_model_names(&value, "openai:chat"), vec!["gpt-5.4", "gpt-5.4-mini"]);
    }

    #[test]
    fn extract_model_names_supports_gemini_payloads() {
        let value = json!({
            "models": [
                { "name": "models/gemini-2.5-pro" },
                { "name": "models/gemini-2.5-flash" }
            ]
        });

        assert_eq!(extract_model_names(&value, "gemini:chat"), vec!["gemini-2.5-flash", "gemini-2.5-pro"]);
    }
}

use async_trait::async_trait;
use serde_json::{Map, Value};

use crate::application::{ExternalModelCatalog, ModelError, ModelResult};

const MODELS_DEV_URL: &str = "https://models.dev/api.json";
const OFFICIAL_PROVIDERS: &[&str] = &[
    "anthropic",
    "openai",
    "google",
    "google-vertex",
    "azure",
    "amazon-bedrock",
    "xai",
    "meta",
    "deepseek",
    "mistral",
    "cohere",
    "zhipuai",
    "alibaba",
    "minimax",
    "moonshot",
    "baichuan",
    "ai21",
];

#[derive(Clone)]
pub struct ModelsDevClient {
    http: reqwest::Client,
}

impl ModelsDevClient {
    pub fn new() -> Self {
        Self { http: reqwest::Client::new() }
    }
}

impl Default for ModelsDevClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExternalModelCatalog for ModelsDevClient {
    async fn models_dev(&self) -> ModelResult<Value> {
        let response = self.http.get(MODELS_DEV_URL).send().await.map_err(external_error)?;
        let status = response.status();
        if !status.is_success() {
            return Err(ModelError::External(format!("models.dev returned HTTP {status}")));
        }
        let data = response.json::<Value>().await.map_err(external_error)?;
        mark_official_providers(data)
    }
}

fn mark_official_providers(data: Value) -> ModelResult<Value> {
    let Value::Object(map) = data else {
        return Err(ModelError::External("models.dev response must be a provider object".into()));
    };
    let marked = map.into_iter().map(mark_provider).collect();
    Ok(Value::Object(marked))
}

fn mark_provider((provider_id, provider_data): (String, Value)) -> (String, Value) {
    let official = OFFICIAL_PROVIDERS.contains(&provider_id.as_str());
    match provider_data {
        Value::Object(mut provider) => {
            provider.insert("official".into(), Value::Bool(official));
            (provider_id, Value::Object(provider))
        }
        value => {
            let mut provider = Map::new();
            provider.insert("official".into(), Value::Bool(official));
            provider.insert("value".into(), value);
            (provider_id, Value::Object(provider))
        }
    }
}

fn external_error(error: reqwest::Error) -> ModelError {
    ModelError::External(error.to_string())
}

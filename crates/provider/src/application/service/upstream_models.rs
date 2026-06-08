use types::provider::{ProviderEndpoint, ProviderUpstreamModelsResponse};

use crate::application::{ProviderApiKeySecret, ProviderError, ProviderRepository, ProviderResult, SecretCipher, UpstreamModelFetcher};

pub struct FetchUpstreamModels<'a, R, C, F> {
    pub repository: &'a R,
    pub cipher: &'a C,
    pub fetcher: &'a F,
    pub provider_id: &'a str,
}

pub async fn fetch_upstream_models<R, C, F>(args: FetchUpstreamModels<'_, R, C, F>) -> ProviderResult<ProviderUpstreamModelsResponse>
where
    R: ProviderRepository,
    C: SecretCipher,
    F: UpstreamModelFetcher,
{
    let endpoints = active_endpoints(args.repository.list_endpoints(args.provider_id).await?);
    if endpoints.is_empty() {
        return Err(ProviderError::InvalidInput("provider has no active endpoint".into()));
    }
    let keys = active_api_key_secrets(args.repository.list_api_key_secrets(args.provider_id).await?);
    if keys.is_empty() {
        return Err(ProviderError::InvalidInput("provider has no active API key".into()));
    }
    fetch_first_success(args, endpoints, keys).await
}

async fn fetch_first_success<R, C, F>(
    args: FetchUpstreamModels<'_, R, C, F>,
    endpoints: Vec<ProviderEndpoint>,
    keys: Vec<ProviderApiKeySecret>,
) -> ProviderResult<ProviderUpstreamModelsResponse>
where
    R: ProviderRepository,
    C: SecretCipher,
    F: UpstreamModelFetcher,
{
    let mut errors = Vec::new();
    for endpoint in endpoints {
        for key in keys.iter().filter(|key| key_supports_endpoint(key, &endpoint.api_format)) {
            let decrypted = args.cipher.decrypt_provider_key(&key.encrypted_api_key)?;
            match args.fetcher.fetch_upstream_models(&endpoint, &decrypted).await {
                Ok(response) => return Ok(response),
                Err(error) => errors.push(format!("{} / {}: {error}", endpoint.api_format, key.name)),
            }
        }
    }
    Err(ProviderError::Infrastructure(format!(
        "failed to fetch upstream models: {}",
        errors.join(" | ")
    )))
}

fn active_endpoints(endpoints: Vec<ProviderEndpoint>) -> Vec<ProviderEndpoint> {
    endpoints.into_iter().filter(|endpoint| endpoint.is_active).collect()
}

fn active_api_key_secrets(mut keys: Vec<ProviderApiKeySecret>) -> Vec<ProviderApiKeySecret> {
    keys.retain(|key| key.is_active);
    keys
}

fn key_supports_endpoint(key: &ProviderApiKeySecret, api_format: &str) -> bool {
    key.api_formats.iter().any(|format| format == api_format)
}

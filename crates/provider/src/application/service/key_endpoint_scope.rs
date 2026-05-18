use std::collections::HashSet;

use types::provider::ProviderEndpoint;

use crate::application::{ProviderError, ProviderRepository, ProviderResult};

pub async fn ensure_api_formats_bound<R>(repository: &R, provider_id: &str, api_formats: &[String]) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    let endpoints = repository.list_endpoints(provider_id).await?;
    validate_api_formats_bound(api_formats, &endpoints)
}

pub fn validate_api_formats_bound(api_formats: &[String], endpoints: &[ProviderEndpoint]) -> ProviderResult<()> {
    let endpoint_formats = endpoint_format_set(endpoints);
    let unbound = unbound_api_formats(api_formats, &endpoint_formats);

    if unbound.is_empty() {
        return Ok(());
    }

    Err(ProviderError::InvalidInput(format!(
        "api_formats must be bound to provider endpoints: {}",
        unbound.join(", ")
    )))
}

fn endpoint_format_set(endpoints: &[ProviderEndpoint]) -> HashSet<&str> {
    endpoints.iter().map(|endpoint| endpoint.api_format.as_str()).collect()
}

fn unbound_api_formats(api_formats: &[String], endpoint_formats: &HashSet<&str>) -> Vec<String> {
    api_formats
        .iter()
        .filter(|api_format| !endpoint_formats.contains(api_format.as_str()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_api_formats_bound_accepts_endpoint_formats() {
        let api_formats = vec!["openai_chat".to_owned(), "gemini_cli".to_owned()];
        let endpoints = vec![endpoint("openai_chat"), endpoint("gemini_cli")];

        let result = validate_api_formats_bound(&api_formats, &endpoints);

        assert!(result.is_ok());
    }

    #[test]
    fn validate_api_formats_bound_rejects_unbound_formats() {
        let api_formats = vec!["openai_chat".to_owned(), "claude_chat".to_owned()];
        let endpoints = vec![endpoint("openai_chat")];

        let error = validate_api_formats_bound(&api_formats, &endpoints).unwrap_err();

        assert_eq!(error.to_string(), "invalid input: api_formats must be bound to provider endpoints: claude_chat");
    }

    fn endpoint(api_format: &str) -> ProviderEndpoint {
        ProviderEndpoint {
            id: format!("endpoint-{api_format}"),
            provider_id: "provider-1".to_owned(),
            api_format: api_format.to_owned(),
            base_url: "https://api.example.com".to_owned(),
            custom_path: None,
            max_retries: None,
            is_active: true,
            format_acceptance_config: None,
            header_rules: None,
            body_rules: None,
            created_at: "2026-05-18T00:00:00Z".to_owned(),
            updated_at: "2026-05-18T00:00:00Z".to_owned(),
        }
    }
}

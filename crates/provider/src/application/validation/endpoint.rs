use types::provider::{ProviderEndpointCreate, ProviderEndpointUpdate};

use crate::application::{ProviderError, ProviderResult};

use super::{MAX_API_FORMAT_LENGTH, MAX_URL_LENGTH, trim_optional, trim_patch, validate_text};

pub fn sanitize_endpoint(input: ProviderEndpointCreate) -> ProviderEndpointCreate {
    ProviderEndpointCreate {
        api_format: input.api_format.trim().to_ascii_lowercase(),
        base_url: normalize_base_url(input.base_url),
        custom_path: input.custom_path.and_then(trim_optional),
        ..input
    }
}

pub fn sanitize_endpoint_update(input: ProviderEndpointUpdate) -> ProviderEndpointUpdate {
    ProviderEndpointUpdate {
        api_format: input.api_format.map(|value| value.trim().to_ascii_lowercase()),
        base_url: input.base_url.map(normalize_base_url),
        custom_path: trim_patch(input.custom_path),
        ..input
    }
}

pub fn validate_endpoint(input: &ProviderEndpointCreate) -> ProviderResult<()> {
    validate_text("api_format", &input.api_format, MAX_API_FORMAT_LENGTH)?;
    validate_text("base_url", &input.base_url, MAX_URL_LENGTH)
}

pub fn validate_endpoint_update(input: &ProviderEndpointUpdate) -> ProviderResult<()> {
    if endpoint_update_is_empty(input) {
        return Err(ProviderError::InvalidInput("endpoint update payload is empty".into()));
    }
    if let Some(api_format) = input.api_format.as_deref() {
        validate_text("api_format", api_format, MAX_API_FORMAT_LENGTH)?;
    }
    if let Some(base_url) = input.base_url.as_deref() {
        validate_text("base_url", base_url, MAX_URL_LENGTH)?;
    }
    Ok(())
}

fn normalize_base_url(value: String) -> String {
    let trimmed = value.trim();
    if trimmed.ends_with("://") {
        return trimmed.to_owned();
    }
    trimmed.trim_end_matches('/').to_owned()
}

fn endpoint_update_is_empty(input: &ProviderEndpointUpdate) -> bool {
    input.api_format.is_none()
        && input.base_url.is_none()
        && input.custom_path.is_missing()
        && input.max_retries.is_missing()
        && input.is_active.is_none()
        && input.format_acceptance_config.is_missing()
        && input.header_rules.is_missing()
        && input.body_rules.is_missing()
}

#[cfg(test)]
mod tests {
    use types::model::PatchField;

    use super::*;

    #[test]
    fn sanitize_endpoint_removes_base_url_trailing_slashes() {
        let input = ProviderEndpointCreate {
            api_format: " OpenAI_CHAT ".to_owned(),
            base_url: " https://api.example.com/// ".to_owned(),
            custom_path: None,
            max_retries: None,
            is_active: None,
            format_acceptance_config: None,
            header_rules: None,
            body_rules: None,
        };

        let output = sanitize_endpoint(input);

        assert_eq!(output.api_format, "openai_chat");
        assert_eq!(output.base_url, "https://api.example.com");
    }

    #[test]
    fn sanitize_endpoint_update_removes_base_url_trailing_slashes() {
        let input = ProviderEndpointUpdate {
            base_url: Some(" https://api.example.com/ ".to_owned()),
            custom_path: PatchField::Missing,
            ..Default::default()
        };

        let output = sanitize_endpoint_update(input);

        assert_eq!(output.base_url.as_deref(), Some("https://api.example.com"));
    }
}

use std::sync::LazyLock;

use regex::Regex;
use req::Url;
use types::{
    model::PatchField,
    provider::{ProviderEndpointCreate, ProviderEndpointUpdate},
};

use crate::application::{ProviderError, ProviderResult};

use super::{MAX_API_FORMAT_LENGTH, MAX_URL_LENGTH, trim_optional, trim_patch, validate_text};

const BASE_URL_PATTERN: &str = r"^https?://(?:localhost|[A-Za-z0-9.-]+|\[[0-9A-Fa-f:.]+\])(?::[0-9]{1,5})?(?:/[A-Za-z0-9._~%!$&'()*+,;=:@/-]*)?$";
const CUSTOM_PATH_PATTERN: &str = r"^/[A-Za-z0-9._~%!$&'()*+,;=:@/{}-]*$";

static BASE_URL_REGEX: LazyLock<Result<Regex, regex::Error>> = LazyLock::new(|| Regex::new(BASE_URL_PATTERN));
static CUSTOM_PATH_REGEX: LazyLock<Result<Regex, regex::Error>> = LazyLock::new(|| Regex::new(CUSTOM_PATH_PATTERN));

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
    validate_text("base_url", &input.base_url, MAX_URL_LENGTH)?;
    validate_base_url(&input.base_url)?;
    validate_custom_path(input.custom_path.as_deref())
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
        validate_base_url(base_url)?;
    }
    if let PatchField::Value(custom_path) = &input.custom_path {
        validate_custom_path(Some(custom_path))?;
    }
    Ok(())
}

fn validate_base_url(value: &str) -> ProviderResult<()> {
    let url = Url::parse(value).map_err(|_| invalid_base_url())?;
    if url.query().is_some() || url.fragment().is_some() {
        return Err(ProviderError::InvalidInput("base_url must not contain query or fragment".into()));
    }
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() {
        return Err(invalid_base_url());
    }
    if !base_url_regex()?.is_match(value) {
        return Err(invalid_base_url());
    }
    Ok(())
}

fn validate_custom_path(value: Option<&str>) -> ProviderResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    validate_text("custom_path", value, MAX_URL_LENGTH)?;
    if !custom_path_regex()?.is_match(value) {
        return Err(ProviderError::InvalidInput(
            "custom_path must start with / and contain only URL path characters".into(),
        ));
    }
    Ok(())
}

fn base_url_regex() -> ProviderResult<&'static Regex> {
    BASE_URL_REGEX
        .as_ref()
        .map_err(|error| ProviderError::Infrastructure(format!("invalid base_url validation regex: {error}")))
}

fn custom_path_regex() -> ProviderResult<&'static Regex> {
    CUSTOM_PATH_REGEX
        .as_ref()
        .map_err(|error| ProviderError::Infrastructure(format!("invalid custom_path validation regex: {error}")))
}

fn invalid_base_url() -> ProviderError {
    ProviderError::InvalidInput("base_url must be a valid HTTP or HTTPS URL".into())
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

    #[test]
    fn validate_endpoint_rejects_invalid_base_url() {
        let input = endpoint_create("not-a-url", None);

        let error = validate_endpoint(&input).unwrap_err();

        assert_eq!(error.to_string(), "invalid input: base_url must be a valid HTTP or HTTPS URL");
    }

    #[test]
    fn validate_endpoint_rejects_base_url_with_query() {
        let input = endpoint_create("https://api.example.com/v1?token=secret", None);

        let error = validate_endpoint(&input).unwrap_err();

        assert_eq!(error.to_string(), "invalid input: base_url must not contain query or fragment");
    }

    #[test]
    fn validate_endpoint_rejects_invalid_custom_path() {
        let input = endpoint_create("https://api.example.com", Some("v1/chat completions"));

        let error = validate_endpoint(&input).unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid input: custom_path must start with / and contain only URL path characters"
        );
    }

    #[test]
    fn validate_endpoint_update_rejects_invalid_custom_path() {
        let input = ProviderEndpointUpdate {
            custom_path: PatchField::Value("/v1/chat completions".to_owned()),
            ..Default::default()
        };

        let error = validate_endpoint_update(&input).unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid input: custom_path must start with / and contain only URL path characters"
        );
    }

    #[test]
    fn validate_endpoint_accepts_localhost_base_url_and_template_path() {
        let input = endpoint_create("http://localhost:11434/v1", Some("/v1beta/models/{model}:{action}"));

        validate_endpoint(&input).unwrap();
    }

    fn endpoint_create(base_url: &str, custom_path: Option<&str>) -> ProviderEndpointCreate {
        ProviderEndpointCreate {
            api_format: "openai_chat".to_owned(),
            base_url: base_url.to_owned(),
            custom_path: custom_path.map(str::to_owned),
            max_retries: None,
            is_active: None,
            format_acceptance_config: None,
            header_rules: None,
            body_rules: None,
        }
    }
}

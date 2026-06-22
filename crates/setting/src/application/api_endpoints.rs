use types::system_setting::{ApiEndpoint, public_base_url_is_valid};

use super::{SettingError, SettingResult};

const MAX_API_ENDPOINT_FIELD_LENGTH: usize = 255;

pub(super) fn sanitize_api_endpoints(endpoints: Vec<ApiEndpoint>) -> Vec<ApiEndpoint> {
    endpoints
        .into_iter()
        .map(|endpoint| ApiEndpoint {
            id: endpoint.id.trim().to_owned(),
            name: endpoint.name.trim().to_owned(),
            url: endpoint.url.trim().trim_end_matches('/').to_owned(),
            description: endpoint.description.trim().to_owned(),
        })
        .collect()
}

pub(super) fn validate_api_endpoints(endpoints: Option<&[ApiEndpoint]>) -> SettingResult<()> {
    let Some(endpoints) = endpoints else {
        return Ok(());
    };
    for endpoint in endpoints {
        validate_api_endpoint(endpoint)?;
    }
    Ok(())
}

fn validate_api_endpoint(endpoint: &ApiEndpoint) -> SettingResult<()> {
    validate_required_length("api_endpoints.id", &endpoint.id)?;
    validate_required_length("api_endpoints.name", &endpoint.name)?;
    validate_required_length("api_endpoints.url", &endpoint.url)?;
    validate_optional_length("api_endpoints.description", &endpoint.description)?;
    validate_api_endpoint_url(&endpoint.url)
}

fn validate_api_endpoint_url(value: &str) -> SettingResult<()> {
    let is_valid =
        public_base_url_is_valid(value).map_err(|error| SettingError::Infrastructure(format!("invalid api_endpoints.url validation regex: {error}")))?;
    if is_valid {
        return Ok(());
    }
    Err(SettingError::InvalidInput("api_endpoints.url must be a valid HTTP or HTTPS URL".into()))
}

fn validate_required_length(field: &str, value: &str) -> SettingResult<()> {
    if value.is_empty() || value.len() > MAX_API_ENDPOINT_FIELD_LENGTH {
        return Err(SettingError::InvalidInput(format!(
            "{field} length must be between 1 and {MAX_API_ENDPOINT_FIELD_LENGTH}"
        )));
    }
    Ok(())
}

fn validate_optional_length(field: &str, value: &str) -> SettingResult<()> {
    if value.len() > MAX_API_ENDPOINT_FIELD_LENGTH {
        return Err(SettingError::InvalidInput(format!(
            "{field} length must be at most {MAX_API_ENDPOINT_FIELD_LENGTH}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use types::system_setting::ApiEndpoint;

    use super::{sanitize_api_endpoints, validate_api_endpoints};

    #[test]
    fn sanitize_api_endpoints_trims_fields_and_trailing_slash() {
        let sanitized = sanitize_api_endpoints(vec![api_endpoint(
            "  endpoint-1  ",
            "  Global  ",
            "  https://api.example.com/  ",
            "  primary  ",
        )]);

        assert_eq!(sanitized[0], api_endpoint("endpoint-1", "Global", "https://api.example.com", "primary"));
    }

    #[test]
    fn validate_api_endpoints_accepts_empty_list() {
        assert!(validate_api_endpoints(Some(&[])).is_ok());
    }

    #[test]
    fn validate_api_endpoints_rejects_missing_name() {
        let endpoints = vec![api_endpoint("endpoint-1", "", "https://api.example.com", "")];
        let error = validate_api_endpoints(Some(&endpoints)).unwrap_err();

        assert_eq!(error.to_string(), "invalid input: api_endpoints.name length must be between 1 and 255");
    }

    #[test]
    fn validate_api_endpoints_rejects_invalid_url() {
        let endpoints = vec![api_endpoint("endpoint-1", "Global", "ftp://api.example.com", "")];
        let error = validate_api_endpoints(Some(&endpoints)).unwrap_err();

        assert_eq!(error.to_string(), "invalid input: api_endpoints.url must be a valid HTTP or HTTPS URL");
    }

    fn api_endpoint(id: &str, name: &str, url: &str, description: &str) -> ApiEndpoint {
        ApiEndpoint {
            id: id.into(),
            name: name.into(),
            url: url.into(),
            description: description.into(),
        }
    }
}

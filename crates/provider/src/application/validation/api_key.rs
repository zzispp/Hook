use types::{
    model::PatchField,
    provider::{ProviderApiKeyCreate, ProviderApiKeyUpdate, parse_provider_key_time_range_minute},
};

use crate::application::{ProviderError, ProviderResult};

use super::{MAX_API_FORMAT_LENGTH, MAX_MODEL_ID_LENGTH, MAX_NAME_LENGTH, trim_optional, trim_patch, validate_text};

const TIME_RANGE_REQUIRED_MESSAGE: &str = "time_range_start and time_range_end are required when time_range_enabled is true";
const TIME_RANGE_SAME_VALUE_MESSAGE: &str = "time_range_start and time_range_end cannot be equal";
const TIME_RANGE_UPDATE_ENABLED_MESSAGE: &str = "time_range_enabled must be provided when changing time range";

pub fn sanitize_api_key(input: ProviderApiKeyCreate) -> ProviderApiKeyCreate {
    ProviderApiKeyCreate {
        name: input.name.trim().to_owned(),
        api_key: input.api_key.trim().to_owned(),
        api_formats: normalize_api_formats(input.api_formats),
        allowed_model_ids: normalize_ids(input.allowed_model_ids),
        note: input.note.and_then(trim_optional),
        time_range_start: input.time_range_start.and_then(trim_optional),
        time_range_end: input.time_range_end.and_then(trim_optional),
        ..input
    }
}

pub fn sanitize_api_key_update(input: ProviderApiKeyUpdate) -> ProviderApiKeyUpdate {
    ProviderApiKeyUpdate {
        name: input.name.map(|value| value.trim().to_owned()),
        api_key: input.api_key.map(|value| value.trim().to_owned()),
        api_formats: input.api_formats.map(normalize_api_formats),
        allowed_model_ids: input.allowed_model_ids.map(normalize_ids),
        note: trim_patch(input.note),
        time_range_start: trim_patch(input.time_range_start),
        time_range_end: trim_patch(input.time_range_end),
        ..input
    }
}

pub fn validate_api_key(input: &ProviderApiKeyCreate) -> ProviderResult<()> {
    validate_text("name", &input.name, MAX_NAME_LENGTH)?;
    if input.api_key.is_empty() {
        return Err(ProviderError::InvalidInput("api_key cannot be blank".into()));
    }
    validate_api_formats(&input.api_formats)?;
    validate_ids("allowed_model_ids", &input.allowed_model_ids)?;
    validate_create_time_range(input)
}

pub fn validate_api_key_update(input: &ProviderApiKeyUpdate) -> ProviderResult<()> {
    if api_key_update_is_empty(input) {
        return Err(ProviderError::InvalidInput("api key update payload is empty".into()));
    }
    if let Some(name) = input.name.as_deref() {
        validate_text("name", name, MAX_NAME_LENGTH)?;
    }
    if input.api_key.as_deref().is_some_and(str::is_empty) {
        return Err(ProviderError::InvalidInput("api_key cannot be blank".into()));
    }
    if let Some(api_formats) = &input.api_formats {
        validate_api_formats(api_formats)?;
    }
    if let Some(allowed_model_ids) = &input.allowed_model_ids {
        validate_ids("allowed_model_ids", allowed_model_ids)?;
    }
    validate_update_time_range(input)
}

fn validate_create_time_range(input: &ProviderApiKeyCreate) -> ProviderResult<()> {
    let start = validate_optional_time_range("time_range_start", input.time_range_start.as_deref())?;
    let end = validate_optional_time_range("time_range_end", input.time_range_end.as_deref())?;
    if !input.time_range_enabled.unwrap_or(false) {
        return Ok(());
    }
    let (Some(start), Some(end)) = (start, end) else {
        return Err(ProviderError::InvalidInput(TIME_RANGE_REQUIRED_MESSAGE.into()));
    };
    validate_distinct_time_range(start, end)
}

fn validate_update_time_range(input: &ProviderApiKeyUpdate) -> ProviderResult<()> {
    if input.time_range_enabled.is_none() && (!input.time_range_start.is_missing() || !input.time_range_end.is_missing()) {
        return Err(ProviderError::InvalidInput(TIME_RANGE_UPDATE_ENABLED_MESSAGE.into()));
    }
    match input.time_range_enabled {
        Some(true) => validate_enabled_update_time_range(input),
        Some(false) => {
            validate_patch_time_range("time_range_start", &input.time_range_start)?;
            validate_patch_time_range("time_range_end", &input.time_range_end)
        }
        None => Ok(()),
    }
}

fn validate_enabled_update_time_range(input: &ProviderApiKeyUpdate) -> ProviderResult<()> {
    let PatchField::Value(start) = &input.time_range_start else {
        return Err(ProviderError::InvalidInput(TIME_RANGE_REQUIRED_MESSAGE.into()));
    };
    let PatchField::Value(end) = &input.time_range_end else {
        return Err(ProviderError::InvalidInput(TIME_RANGE_REQUIRED_MESSAGE.into()));
    };
    let start = validate_time_range("time_range_start", start)?;
    let end = validate_time_range("time_range_end", end)?;
    validate_distinct_time_range(start, end)
}

fn validate_patch_time_range(field: &str, value: &PatchField<String>) -> ProviderResult<()> {
    if let PatchField::Value(value) = value {
        validate_time_range(field, value)?;
    }
    Ok(())
}

fn validate_optional_time_range(field: &str, value: Option<&str>) -> ProviderResult<Option<u16>> {
    value.map(|value| validate_time_range(field, value)).transpose()
}

fn validate_time_range(field: &str, value: &str) -> ProviderResult<u16> {
    parse_provider_key_time_range_minute(value).ok_or_else(|| ProviderError::InvalidInput(format!("{field} must use HH:mm format")))
}

fn validate_distinct_time_range(start: u16, end: u16) -> ProviderResult<()> {
    if start == end {
        return Err(ProviderError::InvalidInput(TIME_RANGE_SAME_VALUE_MESSAGE.into()));
    }
    Ok(())
}

fn validate_api_formats(api_formats: &[String]) -> ProviderResult<()> {
    if api_formats.is_empty() {
        return Err(ProviderError::InvalidInput("api_formats cannot be empty".into()));
    }
    for api_format in api_formats {
        validate_text("api_formats", api_format, MAX_API_FORMAT_LENGTH)?;
        validate_canonical_chat_cli_format(api_format)?;
    }
    Ok(())
}

fn validate_canonical_chat_cli_format(value: &str) -> ProviderResult<()> {
    if matches!(
        value,
        "openai_chat" | "openai_cli" | "openai_compact" | "claude_chat" | "claude_cli" | "gemini_chat" | "gemini_cli"
    ) {
        return Err(ProviderError::InvalidInput(format!(
            "api_formats must use canonical family:kind format: {value}"
        )));
    }
    Ok(())
}

fn validate_ids(field: &str, values: &[String]) -> ProviderResult<()> {
    for value in values {
        validate_text(field, value, MAX_MODEL_ID_LENGTH)?;
    }
    Ok(())
}

fn normalize_api_formats(values: Vec<String>) -> Vec<String> {
    let mut output = Vec::new();
    for value in values {
        let value = value.trim().to_ascii_lowercase();
        if !value.is_empty() && !output.contains(&value) {
            output.push(value);
        }
    }
    output
}

fn normalize_ids(values: Vec<String>) -> Vec<String> {
    let mut output = Vec::new();
    for value in values {
        let value = value.trim().to_owned();
        if !value.is_empty() && !output.contains(&value) {
            output.push(value);
        }
    }
    output
}

fn api_key_update_is_empty(input: &ProviderApiKeyUpdate) -> bool {
    input.name.is_none()
        && input.api_key.is_none()
        && input.api_formats.is_none()
        && input.allowed_model_ids.is_none()
        && input.note.is_missing()
        && input.internal_priority.is_none()
        && input.rpm_limit.is_missing()
        && input.cache_ttl_minutes.is_none()
        && input.max_probe_interval_minutes.is_none()
        && input.time_range_enabled.is_none()
        && input.time_range_start.is_missing()
        && input.time_range_end.is_missing()
        && input.is_active.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_api_key_requires_times_when_time_range_enabled() {
        let input = api_key_create(true, None, Some("18:00"));

        let error = validate_api_key(&input).unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid input: time_range_start and time_range_end are required when time_range_enabled is true"
        );
    }

    #[test]
    fn validate_api_key_rejects_invalid_time_range_format() {
        let input = api_key_create(true, Some("8:00"), Some("18:00"));

        let error = validate_api_key(&input).unwrap_err();

        assert_eq!(error.to_string(), "invalid input: time_range_start must use HH:mm format");
    }

    #[test]
    fn validate_api_key_update_requires_enabled_flag_when_changing_time_range() {
        let input = ProviderApiKeyUpdate {
            time_range_start: PatchField::Value("08:00".to_owned()),
            ..Default::default()
        };

        let error = validate_api_key_update(&input).unwrap_err();

        assert_eq!(error.to_string(), "invalid input: time_range_enabled must be provided when changing time range");
    }

    #[test]
    fn validate_api_key_update_accepts_cross_midnight_time_range() {
        let input = ProviderApiKeyUpdate {
            time_range_enabled: Some(true),
            time_range_start: PatchField::Value("22:00".to_owned()),
            time_range_end: PatchField::Value("06:00".to_owned()),
            ..Default::default()
        };

        validate_api_key_update(&input).unwrap();
    }

    #[test]
    fn validate_api_key_rejects_legacy_chat_cli_format() {
        let mut input = api_key_create(false, None, None);
        input.api_formats = vec!["openai_chat".to_owned()];

        let error = validate_api_key(&input).unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid input: api_formats must use canonical family:kind format: openai_chat"
        );
    }

    fn api_key_create(enabled: bool, start: Option<&str>, end: Option<&str>) -> ProviderApiKeyCreate {
        ProviderApiKeyCreate {
            name: "key-a".to_owned(),
            api_key: "sk-test".to_owned(),
            api_formats: vec!["openai:chat".to_owned()],
            allowed_model_ids: Vec::new(),
            note: None,
            internal_priority: None,
            rpm_limit: None,
            cache_ttl_minutes: None,
            max_probe_interval_minutes: None,
            time_range_enabled: Some(enabled),
            time_range_start: start.map(str::to_owned),
            time_range_end: end.map(str::to_owned),
            is_active: None,
        }
    }
}

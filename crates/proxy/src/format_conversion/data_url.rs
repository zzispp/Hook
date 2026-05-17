use super::FormatConversionError;

const BASE64_MARKER: &str = ";base64";

pub(crate) fn parse_base64_data_url(value: &str, format: &'static str, path: &str) -> Result<Option<(String, String)>, FormatConversionError> {
    let Some(body) = value.strip_prefix("data:") else {
        return Ok(None);
    };
    let Some((metadata, data)) = body.split_once(',') else {
        return Err(FormatConversionError::invalid_payload(format, path));
    };
    let Some(media_type) = metadata.strip_suffix(BASE64_MARKER) else {
        return Err(FormatConversionError::unsupported_content(
            format,
            format!("{path}: data URL must be base64 encoded"),
        ));
    };
    if media_type.is_empty() || data.is_empty() {
        return Err(FormatConversionError::invalid_payload(format, path));
    }
    Ok(Some((media_type.to_owned(), data.to_owned())))
}

pub(crate) fn format_base64_data_url(media_type: Option<&str>, data: &str, format: &'static str) -> Result<String, FormatConversionError> {
    let Some(media_type) = media_type.filter(|value| !value.trim().is_empty()) else {
        return Err(FormatConversionError::unsupported_content(format, "base64 content requires media_type"));
    };
    Ok(format!("data:{media_type}{BASE64_MARKER},{data}"))
}

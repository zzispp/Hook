use types::system_setting::{SystemSettingsResponse, SystemSettingsUpdate, public_base_url_is_valid};

use super::{SettingError, SettingResult};

pub fn validate_oauth_provider_prerequisites(input: &SystemSettingsUpdate, current: &SystemSettingsResponse) -> SettingResult<()> {
    if !effective_oauth_enabled(input, current) {
        return Ok(());
    }
    let public_base_url = input.public_base_url.as_deref().unwrap_or(&current.public_base_url).trim();
    if public_base_url.is_empty() {
        return Err(SettingError::InvalidInput("public_base_url is required before enabling OAuth provider".into()));
    }
    let is_valid = public_base_url_is_valid(public_base_url)
        .map_err(|error| SettingError::Infrastructure(format!("invalid public_base_url validation regex: {error}")))?;
    if !is_valid {
        return Err(SettingError::InvalidInput(
            "public_base_url must be a valid HTTP or HTTPS URL before enabling OAuth provider".into(),
        ));
    }
    Ok(())
}

fn effective_oauth_enabled(input: &SystemSettingsUpdate, current: &SystemSettingsResponse) -> bool {
    input.auth_github_enabled.unwrap_or(current.auth_github_enabled) || input.auth_google_enabled.unwrap_or(current.auth_google_enabled)
}

use types::system_setting::{SystemSettingsResponse, SystemSettingsUpdate, public_base_url_is_valid};

use super::{SettingError, SettingResult};

const ALLOWED_EVM_CHAIN_IDS: [u64; 3] = [1, 56, 42161];

pub fn validate_wallet_provider_prerequisites(input: &SystemSettingsUpdate, current: &SystemSettingsResponse) -> SettingResult<()> {
    if !effective_wallet_enabled(input, current) {
        return Ok(());
    }
    validate_public_base_url(input, current)?;
    validate_evm_provider(input, current)
}

fn effective_wallet_enabled(input: &SystemSettingsUpdate, current: &SystemSettingsResponse) -> bool {
    input.auth_evm_enabled.unwrap_or(current.auth_evm_enabled)
}

fn validate_public_base_url(input: &SystemSettingsUpdate, current: &SystemSettingsResponse) -> SettingResult<()> {
    let public_base_url = input.public_base_url.as_deref().unwrap_or(&current.public_base_url).trim();
    if public_base_url.is_empty() {
        return Err(SettingError::InvalidInput("public_base_url is required before enabling wallet provider".into()));
    }
    let is_valid = public_base_url_is_valid(public_base_url)
        .map_err(|error| SettingError::Infrastructure(format!("invalid public_base_url validation regex: {error}")))?;
    if !is_valid {
        return Err(SettingError::InvalidInput(
            "public_base_url must be a valid HTTP or HTTPS URL before enabling wallet provider".into(),
        ));
    }
    Ok(())
}

fn validate_evm_provider(input: &SystemSettingsUpdate, current: &SystemSettingsResponse) -> SettingResult<()> {
    if !input.auth_evm_enabled.unwrap_or(current.auth_evm_enabled) {
        return Ok(());
    }
    let chain_ids = input.auth_evm_chain_ids.as_deref().unwrap_or(&current.auth_evm_chain_ids);
    let chain_ids = parse_evm_chain_ids(chain_ids)?;
    if chain_ids.is_empty() {
        return Err(SettingError::InvalidInput(
            "auth_evm_chain_ids is required before enabling EVM wallet provider".into(),
        ));
    }
    if chain_ids.iter().any(|id| !ALLOWED_EVM_CHAIN_IDS.contains(id)) {
        return Err(SettingError::InvalidInput("auth_evm_chain_ids contains unsupported EVM network".into()));
    }
    let statement = input.auth_evm_statement.as_deref().unwrap_or(&current.auth_evm_statement).trim();
    if statement.is_empty() {
        return Err(SettingError::InvalidInput(
            "auth_evm_statement is required before enabling EVM wallet provider".into(),
        ));
    }
    Ok(())
}

fn parse_evm_chain_ids(value: &str) -> SettingResult<Vec<u64>> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(parse_evm_chain_id)
        .collect()
}

fn parse_evm_chain_id(value: &str) -> SettingResult<u64> {
    value.parse().map_err(|_| SettingError::InvalidInput(format!("invalid EVM chain id: {value}")))
}

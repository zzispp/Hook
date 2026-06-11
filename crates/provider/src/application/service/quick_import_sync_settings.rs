use rust_decimal::Decimal;
use time::format_description::well_known::Rfc3339;
use types::provider::{
    ProviderOrigin, ProviderQuickImportSyncConfig, ProviderQuickImportSyncSettingsResponse, ProviderQuickImportSyncSettingsUpdate,
    ProviderQuickImportSyncStatus,
};

use crate::application::{ProviderError, ProviderQuickImportSyncSource, ProviderQuickImportSyncSourcePatch, ProviderRepository, ProviderResult, SecretCipher};

pub async fn quick_import_sync_settings<R>(repository: &R, provider_id: &str) -> ProviderResult<ProviderQuickImportSyncSettingsResponse>
where
    R: ProviderRepository,
{
    ensure_quick_import_provider(repository, provider_id).await?;
    let Some(source) = repository.quick_import_sync_source(provider_id).await? else {
        return Ok(unconfigured_response(provider_id));
    };
    settings_response(source)
}

pub async fn update_quick_import_sync_settings<R, C>(
    repository: &R,
    cipher: &C,
    provider_id: &str,
    input: ProviderQuickImportSyncSettingsUpdate,
) -> ProviderResult<ProviderQuickImportSyncSettingsResponse>
where
    R: ProviderRepository,
    C: SecretCipher,
{
    ensure_quick_import_provider(repository, provider_id).await?;
    validate_update(&input)?;
    let patch = source_patch(cipher, input)?;
    let source = repository.update_quick_import_sync_source(provider_id, patch).await?;
    settings_response(source)
}

async fn ensure_quick_import_provider<R>(repository: &R, provider_id: &str) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    let provider = repository.find_provider(provider_id).await?.ok_or(ProviderError::NotFound)?;
    if provider.provider_origin != ProviderOrigin::QuickImport {
        return Err(ProviderError::InvalidInput("provider is not a quick import provider".into()));
    }
    Ok(())
}

fn validate_update(input: &ProviderQuickImportSyncSettingsUpdate) -> ProviderResult<()> {
    validate_text(&input.base_url, "base_url")?;
    validate_text(&input.user_id, "user_id")?;
    validate_text(&input.system_access_token, "system_access_token")?;
    if let Some(value) = input.recharge_multiplier {
        validate_positive_decimal(value, "recharge_multiplier")?;
    }
    if let Some(config) = &input.sync_config
        && config.fetch_failure_disable_threshold == 0
    {
        return Err(ProviderError::InvalidInput("fetch_failure_disable_threshold must be greater than 0".into()));
    }
    Ok(())
}

fn source_patch<C>(cipher: &C, input: ProviderQuickImportSyncSettingsUpdate) -> ProviderResult<ProviderQuickImportSyncSourcePatch>
where
    C: SecretCipher,
{
    Ok(ProviderQuickImportSyncSourcePatch {
        base_url: input.base_url.map(trimmed),
        user_id: input.user_id.map(trimmed),
        encrypted_system_access_token: input.system_access_token.map(|token| cipher.encrypt_provider_key(token.trim())).transpose()?,
        recharge_multiplier: input.recharge_multiplier,
        sync_config: input.sync_config,
    })
}

fn settings_response(source: ProviderQuickImportSyncSource) -> ProviderResult<ProviderQuickImportSyncSettingsResponse> {
    Ok(ProviderQuickImportSyncSettingsResponse {
        provider_id: source.provider_id,
        source_kind: Some(source.source_kind),
        base_url: Some(source.base_url),
        user_id: Some(source.user_id),
        has_system_access_token: !source.encrypted_system_access_token.is_empty(),
        recharge_multiplier: Some(source.recharge_multiplier),
        sync_config: source.sync_config,
        last_status: source.last_status,
        last_error: source.last_error,
        last_synced_at: source.last_synced_at.map(format_timestamp).transpose()?,
        consecutive_failures: source.consecutive_failures,
    })
}

fn unconfigured_response(provider_id: &str) -> ProviderQuickImportSyncSettingsResponse {
    ProviderQuickImportSyncSettingsResponse {
        provider_id: provider_id.to_owned(),
        source_kind: None,
        base_url: None,
        user_id: None,
        has_system_access_token: false,
        recharge_multiplier: None,
        sync_config: ProviderQuickImportSyncConfig {
            auto_sync_enabled: false,
            ..ProviderQuickImportSyncConfig::default()
        },
        last_status: Some(ProviderQuickImportSyncStatus::SourceNotConfigured),
        last_error: None,
        last_synced_at: None,
        consecutive_failures: 0,
    }
}

fn validate_text(value: &Option<String>, field: &str) -> ProviderResult<()> {
    if value.as_ref().is_some_and(|item| item.trim().is_empty()) {
        return Err(ProviderError::InvalidInput(format!("{field} cannot be blank")));
    }
    Ok(())
}

fn validate_positive_decimal(value: Decimal, field: &str) -> ProviderResult<()> {
    if value <= Decimal::ZERO {
        return Err(ProviderError::InvalidInput(format!("{field} must be greater than 0")));
    }
    Ok(())
}

fn trimmed(value: String) -> String {
    value.trim().to_owned()
}

fn format_timestamp(value: time::OffsetDateTime) -> ProviderResult<String> {
    value
        .format(&Rfc3339)
        .map_err(|error| ProviderError::Infrastructure(format!("quick import sync timestamp format failed: {error}")))
}

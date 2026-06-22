use rust_decimal::Decimal;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use types::provider::{
    ProviderOrigin, ProviderQuickImportSourceKind, ProviderQuickImportSyncConfig, ProviderQuickImportSyncSettingsResponse,
    ProviderQuickImportSyncSettingsUpdate, ProviderQuickImportSyncStatus,
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
    let source = repository.quick_import_sync_source(provider_id).await?.ok_or(ProviderError::NotFound)?;
    let source_kind = source.source_kind.clone();
    validate_update(source_kind.clone(), &input)?;
    let patch = source_patch(cipher, source_kind, input)?;
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

fn validate_update(source_kind: ProviderQuickImportSourceKind, input: &ProviderQuickImportSyncSettingsUpdate) -> ProviderResult<()> {
    validate_text(&input.base_url, "base_url")?;
    if let Some(value) = input.recharge_multiplier {
        validate_positive_decimal(value, "recharge_multiplier")?;
    }
    if let Some(config) = &input.sync_config
        && config.fetch_failure_disable_threshold == 0
    {
        return Err(ProviderError::InvalidInput("fetch_failure_disable_threshold must be greater than 0".into()));
    }
    match source_kind {
        ProviderQuickImportSourceKind::Newapi => {
            validate_text(&input.user_id, "user_id")?;
            validate_text(&input.auth_token, "auth_token")?;
            validate_text(&input.refresh_token, "refresh_token")?;
            validate_timestamp(&input.token_expires_at)?;
            forbid_fields(input.auth_token.as_ref(), "auth_token", "newapi")?;
            forbid_fields(input.refresh_token.as_ref(), "refresh_token", "newapi")?;
            forbid_fields(input.token_expires_at.as_ref(), "token_expires_at", "newapi")?;
        }
        ProviderQuickImportSourceKind::Sub2api => {
            let uses_password = input.email.is_some() || input.password.is_some();
            let uses_token = input.auth_token.is_some() || input.refresh_token.is_some() || input.token_expires_at.is_some();
            if uses_password && uses_token {
                return Err(ProviderError::InvalidInput(
                    "sub2api sync settings cannot update password and token auth in the same request".into(),
                ));
            }
            validate_text(&input.email, "email")?;
            validate_text(&input.password, "password")?;
            validate_text(&input.auth_token, "auth_token")?;
            validate_text(&input.refresh_token, "refresh_token")?;
            validate_timestamp(&input.token_expires_at)?;
            forbid_fields(input.user_id.as_ref(), "user_id", "sub2api")?;
            forbid_fields(input.system_access_token.as_ref(), "system_access_token", "sub2api")?;
        }
    }
    Ok(())
}

fn source_patch<C>(
    cipher: &C,
    source_kind: ProviderQuickImportSourceKind,
    input: ProviderQuickImportSyncSettingsUpdate,
) -> ProviderResult<ProviderQuickImportSyncSourcePatch>
where
    C: SecretCipher,
{
    let base_url = input.base_url.map(trimmed);
    let recharge_multiplier = input.recharge_multiplier;
    let sync_config = input.sync_config;
    match source_kind {
        ProviderQuickImportSourceKind::Newapi => Ok(ProviderQuickImportSyncSourcePatch {
            base_url,
            user_id: input.user_id.map(trimmed),
            encrypted_system_access_token: encrypt_optional_secret(cipher, input.system_access_token)?,
            recharge_multiplier,
            sync_config,
            ..ProviderQuickImportSyncSourcePatch::default()
        }),
        ProviderQuickImportSourceKind::Sub2api => {
            let uses_password = input.email.is_some() || input.password.is_some();
            let uses_token = input.auth_token.is_some() || input.refresh_token.is_some() || input.token_expires_at.is_some();
            if uses_password {
                Ok(ProviderQuickImportSyncSourcePatch {
                    base_url,
                    email: input.email.map(trimmed),
                    encrypted_password: encrypt_optional_secret(cipher, input.password)?,
                    encrypted_auth_token: Some(String::new()),
                    encrypted_refresh_token: Some(String::new()),
                    token_expires_at: Some(None),
                    recharge_multiplier,
                    sync_config,
                    ..ProviderQuickImportSyncSourcePatch::default()
                })
            } else if uses_token {
                Ok(ProviderQuickImportSyncSourcePatch {
                    base_url,
                    email: Some(String::new()),
                    encrypted_password: Some(String::new()),
                    encrypted_auth_token: encrypt_optional_secret(cipher, input.auth_token)?,
                    encrypted_refresh_token: encrypt_optional_secret(cipher, input.refresh_token)?,
                    token_expires_at: optional_timestamp(input.token_expires_at)?,
                    recharge_multiplier,
                    sync_config,
                    ..ProviderQuickImportSyncSourcePatch::default()
                })
            } else {
                Ok(ProviderQuickImportSyncSourcePatch {
                    base_url,
                    recharge_multiplier,
                    sync_config,
                    ..ProviderQuickImportSyncSourcePatch::default()
                })
            }
        }
    }
}

fn settings_response(source: ProviderQuickImportSyncSource) -> ProviderResult<ProviderQuickImportSyncSettingsResponse> {
    let user_id = user_id_for_source(&source);
    let email = email_for_source(&source);
    let token_expires_at = source.token_expires_at.map(format_timestamp).transpose()?;
    let has_system_access_token = !source.encrypted_system_access_token.is_empty();
    let has_password = !source.encrypted_password.is_empty();
    let has_auth_token = !source.encrypted_auth_token.is_empty();
    let has_refresh_token = !source.encrypted_refresh_token.is_empty();
    let last_synced_at = source.last_synced_at.map(format_timestamp).transpose()?;
    Ok(ProviderQuickImportSyncSettingsResponse {
        provider_id: source.provider_id,
        source_kind: Some(source.source_kind),
        base_url: Some(source.base_url),
        user_id,
        email,
        token_expires_at,
        has_system_access_token,
        has_password,
        has_auth_token,
        has_refresh_token,
        recharge_multiplier: Some(source.recharge_multiplier),
        sync_config: source.sync_config,
        last_status: source.last_status,
        last_error: source.last_error,
        last_synced_at,
        consecutive_failures: source.consecutive_failures,
    })
}

fn unconfigured_response(provider_id: &str) -> ProviderQuickImportSyncSettingsResponse {
    ProviderQuickImportSyncSettingsResponse {
        provider_id: provider_id.to_owned(),
        source_kind: None,
        base_url: None,
        user_id: None,
        email: None,
        token_expires_at: None,
        has_system_access_token: false,
        has_password: false,
        has_auth_token: false,
        has_refresh_token: false,
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

fn forbid_fields(value: Option<&String>, field: &str, source_kind: &str) -> ProviderResult<()> {
    if value.is_some() {
        return Err(ProviderError::InvalidInput(format!("{field} is not allowed for {source_kind} sync settings")));
    }
    Ok(())
}

fn encrypt_optional_secret<C>(cipher: &C, value: Option<String>) -> ProviderResult<Option<String>>
where
    C: SecretCipher,
{
    let Some(value) = value.map(|item| item.trim().to_owned()).filter(|item| !item.is_empty()) else {
        return Ok(None);
    };
    cipher.encrypt_provider_key(&value).map(Some)
}

fn user_id_for_source(source: &ProviderQuickImportSyncSource) -> Option<String> {
    match source.source_kind {
        ProviderQuickImportSourceKind::Newapi => Some(source.user_id.clone()),
        ProviderQuickImportSourceKind::Sub2api => None,
    }
}

fn email_for_source(source: &ProviderQuickImportSyncSource) -> Option<String> {
    match source.source_kind {
        ProviderQuickImportSourceKind::Newapi => None,
        ProviderQuickImportSourceKind::Sub2api => (!source.email.trim().is_empty()).then(|| source.email.clone()),
    }
}

fn validate_timestamp(value: &Option<String>) -> ProviderResult<()> {
    if let Some(value) = value.as_ref() {
        parse_timestamp(value)?;
    }
    Ok(())
}

fn optional_timestamp(value: Option<String>) -> ProviderResult<Option<Option<OffsetDateTime>>> {
    match value {
        Some(value) => Ok(Some(Some(parse_timestamp(&value)?))),
        None => Ok(None),
    }
}

fn parse_timestamp(value: &str) -> ProviderResult<OffsetDateTime> {
    let trimmed = value.trim();
    if let Ok(milliseconds) = trimmed.parse::<i128>() {
        let seconds = milliseconds.div_euclid(1000) as i64;
        let nanos = (milliseconds.rem_euclid(1000) as i64) * 1_000_000;
        return OffsetDateTime::from_unix_timestamp(seconds)
            .map(|value| value + time::Duration::nanoseconds(nanos))
            .map_err(|error| ProviderError::InvalidInput(format!("invalid token_expires_at milliseconds: {error}")));
    }
    OffsetDateTime::parse(trimmed, &Rfc3339).map_err(|error| ProviderError::InvalidInput(format!("invalid token_expires_at: {error}")))
}

fn format_timestamp(value: time::OffsetDateTime) -> ProviderResult<String> {
    value
        .format(&Rfc3339)
        .map_err(|error| ProviderError::Infrastructure(format!("quick import sync timestamp format failed: {error}")))
}

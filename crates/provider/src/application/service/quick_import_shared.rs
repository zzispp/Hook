use std::collections::BTreeMap;

use rust_decimal::Decimal;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use types::{
    model::GlobalModelResponse,
    provider::{
        ProviderCreate, ProviderQuickImportProviderConfig, ProviderQuickImportSourceConfig, ProviderQuickImportSourceKind, Sub2ApiPasswordQuickImportConfig,
        Sub2ApiQuickImportConfig, Sub2ApiTokenQuickImportConfig,
    },
};

use crate::application::{ProviderError, ProviderQuickImportSyncSource, ProviderQuickImportSyncSourcePatch, ProviderResult, SecretCipher};

const PROVIDER_TYPE_CUSTOM: &str = "custom";
const DEFAULT_MAX_RETRIES: i32 = 2;
const DEFAULT_REQUEST_TIMEOUT_SECONDS: f64 = 300.0;
const DEFAULT_STREAM_FIRST_BYTE_TIMEOUT_SECONDS: f64 = 12.0;
const DEFAULT_STREAM_FIRST_OUTPUT_TIMEOUT_SECONDS: f64 = 45.0;
const DEFAULT_STREAM_IDLE_TIMEOUT_SECONDS: f64 = 300.0;
const DEFAULT_PRIORITY: i32 = 100;

pub fn provider_create(name: &str, config: &ProviderQuickImportProviderConfig) -> ProviderCreate {
    ProviderCreate {
        name: name.trim().to_owned(),
        provider_type: PROVIDER_TYPE_CUSTOM.into(),
        max_retries: Some(config.max_retries.unwrap_or(DEFAULT_MAX_RETRIES)),
        request_timeout_seconds: Some(config.request_timeout_seconds.unwrap_or(DEFAULT_REQUEST_TIMEOUT_SECONDS)),
        stream_first_byte_timeout_seconds: Some(config.stream_first_byte_timeout_seconds.unwrap_or(DEFAULT_STREAM_FIRST_BYTE_TIMEOUT_SECONDS)),
        stream_first_output_timeout_seconds: Some(
            config
                .stream_first_output_timeout_seconds
                .unwrap_or(DEFAULT_STREAM_FIRST_OUTPUT_TIMEOUT_SECONDS),
        ),
        stream_idle_timeout_seconds: Some(config.stream_idle_timeout_seconds.unwrap_or(DEFAULT_STREAM_IDLE_TIMEOUT_SECONDS)),
        priority: Some(config.priority.unwrap_or(DEFAULT_PRIORITY)),
        keep_priority_on_conversion: Some(config.keep_priority_on_conversion.unwrap_or(false)),
        enable_format_conversion: Some(config.enable_format_conversion.unwrap_or(true)),
        is_active: Some(config.is_active.unwrap_or(true)),
    }
}

pub fn validate_common(
    source_kind: ProviderQuickImportSourceKind,
    source: &ProviderQuickImportSourceConfig,
    provider_name: &str,
    recharge_multiplier: Decimal,
) -> ProviderResult<()> {
    if source_kind != source.kind() {
        return Err(ProviderError::InvalidInput("source_kind does not match source.kind".into()));
    }
    if provider_name.trim().is_empty() {
        return Err(ProviderError::InvalidInput("provider_name cannot be blank".into()));
    }
    if recharge_multiplier <= Decimal::ZERO {
        return Err(ProviderError::InvalidInput("recharge_multiplier must be greater than 0".into()));
    }
    validate_source(source)
}

pub fn source_base_url(source: &ProviderQuickImportSourceConfig) -> String {
    match source {
        ProviderQuickImportSourceConfig::Newapi(config) => config.base_url.trim().trim_end_matches('/').to_owned(),
        ProviderQuickImportSourceConfig::Sub2api(Sub2ApiQuickImportConfig::Password(config)) => config.base_url.trim().trim_end_matches('/').to_owned(),
        ProviderQuickImportSourceConfig::Sub2api(Sub2ApiQuickImportConfig::Token(config)) => config.base_url.trim().trim_end_matches('/').to_owned(),
    }
}

pub fn globals_by_name(models: &[GlobalModelResponse]) -> BTreeMap<String, &GlobalModelResponse> {
    models.iter().map(|model| (model.name.clone(), model)).collect()
}

pub fn globals_by_id(models: &[GlobalModelResponse]) -> BTreeMap<String, &GlobalModelResponse> {
    models.iter().map(|model| (model.id.clone(), model)).collect()
}

pub fn global_model<'a>(models: &'a BTreeMap<String, &'a GlobalModelResponse>, id: &str) -> ProviderResult<&'a GlobalModelResponse> {
    models
        .get(id)
        .copied()
        .ok_or_else(|| ProviderError::InvalidInput(format!("global model does not exist or is inactive: {id}")))
}

pub fn refreshed_source_patch<C>(cipher: &C, source: &ProviderQuickImportSourceConfig) -> ProviderResult<ProviderQuickImportSyncSourcePatch>
where
    C: SecretCipher,
{
    match source {
        ProviderQuickImportSourceConfig::Newapi(config) => Ok(ProviderQuickImportSyncSourcePatch {
            base_url: Some(source_base_url(source)),
            encrypted_system_access_token: Some(cipher.encrypt_provider_key(config.system_access_token.trim())?),
            user_id: Some(config.user_id.trim().to_owned()),
            ..ProviderQuickImportSyncSourcePatch::default()
        }),
        ProviderQuickImportSourceConfig::Sub2api(Sub2ApiQuickImportConfig::Password(config)) => Ok(ProviderQuickImportSyncSourcePatch {
            base_url: Some(source_base_url(source)),
            email: Some(config.email.trim().to_owned()),
            encrypted_password: Some(cipher.encrypt_provider_key(config.password.trim())?),
            encrypted_auth_token: Some(String::new()),
            encrypted_refresh_token: Some(String::new()),
            token_expires_at: Some(None),
            ..ProviderQuickImportSyncSourcePatch::default()
        }),
        ProviderQuickImportSourceConfig::Sub2api(Sub2ApiQuickImportConfig::Token(config)) => Ok(ProviderQuickImportSyncSourcePatch {
            base_url: Some(source_base_url(source)),
            email: Some(String::new()),
            encrypted_password: Some(String::new()),
            encrypted_auth_token: Some(cipher.encrypt_provider_key(config.auth_token.trim())?),
            encrypted_refresh_token: Some(cipher.encrypt_provider_key(config.refresh_token.trim())?),
            token_expires_at: Some(Some(parse_token_expires_at(&config.token_expires_at)?)),
            ..ProviderQuickImportSyncSourcePatch::default()
        }),
    }
}

pub fn refreshed_sub2api_token_patch<C>(cipher: &C, source: &ProviderQuickImportSourceConfig) -> ProviderResult<ProviderQuickImportSyncSourcePatch>
where
    C: SecretCipher,
{
    let ProviderQuickImportSourceConfig::Sub2api(Sub2ApiQuickImportConfig::Token(config)) = source else {
        return Err(ProviderError::InvalidInput("sub2api token source is required".into()));
    };
    Ok(ProviderQuickImportSyncSourcePatch {
        encrypted_auth_token: Some(cipher.encrypt_provider_key(config.auth_token.trim())?),
        encrypted_refresh_token: Some(cipher.encrypt_provider_key(config.refresh_token.trim())?),
        token_expires_at: Some(Some(parse_token_expires_at(&config.token_expires_at)?)),
        ..ProviderQuickImportSyncSourcePatch::default()
    })
}

pub fn restore_source_config<C>(cipher: &C, source: &ProviderQuickImportSyncSource) -> ProviderResult<ProviderQuickImportSourceConfig>
where
    C: SecretCipher,
{
    match source.source_kind {
        ProviderQuickImportSourceKind::Newapi => Ok(ProviderQuickImportSourceConfig::Newapi(types::provider::NewApiQuickImportConfig {
            base_url: source.base_url.clone(),
            system_access_token: cipher.decrypt_provider_key(&source.encrypted_system_access_token)?,
            user_id: source.user_id.clone(),
        })),
        ProviderQuickImportSourceKind::Sub2api => Ok(ProviderQuickImportSourceConfig::Sub2api(restore_sub2api_auth(cipher, source)?)),
    }
}

fn validate_source(source: &ProviderQuickImportSourceConfig) -> ProviderResult<()> {
    match source {
        ProviderQuickImportSourceConfig::Newapi(config) => {
            if config.base_url.trim().is_empty() || config.system_access_token.trim().is_empty() || config.user_id.trim().is_empty() {
                return Err(ProviderError::InvalidInput("newapi source fields cannot be blank".into()));
            }
            Ok(())
        }
        ProviderQuickImportSourceConfig::Sub2api(config) => match config {
            Sub2ApiQuickImportConfig::Password(config) => {
                if config.base_url.trim().is_empty() || config.email.trim().is_empty() || config.password.trim().is_empty() {
                    return Err(ProviderError::InvalidInput("sub2api password source fields cannot be blank".into()));
                }
                Ok(())
            }
            Sub2ApiQuickImportConfig::Token(config) => {
                if config.base_url.trim().is_empty()
                    || config.auth_token.trim().is_empty()
                    || config.refresh_token.trim().is_empty()
                    || config.token_expires_at.trim().is_empty()
                {
                    return Err(ProviderError::InvalidInput("sub2api token source fields cannot be blank".into()));
                }
                Ok(())
            }
        },
    }
}

fn restore_sub2api_auth<C>(cipher: &C, source: &ProviderQuickImportSyncSource) -> ProviderResult<Sub2ApiQuickImportConfig>
where
    C: SecretCipher,
{
    if !source.email.trim().is_empty() || !source.encrypted_password.is_empty() {
        return Ok(Sub2ApiQuickImportConfig::Password(Sub2ApiPasswordQuickImportConfig {
            base_url: source.base_url.clone(),
            email: source.email.clone(),
            password: cipher.decrypt_provider_key(&source.encrypted_password)?,
        }));
    }
    Ok(Sub2ApiQuickImportConfig::Token(Sub2ApiTokenQuickImportConfig {
        base_url: source.base_url.clone(),
        auth_token: cipher.decrypt_provider_key(&source.encrypted_auth_token)?,
        refresh_token: cipher.decrypt_provider_key(&source.encrypted_refresh_token)?,
        token_expires_at: source
            .token_expires_at
            .ok_or_else(|| ProviderError::InvalidInput("sub2api token_expires_at is missing".into()))?
            .format(&Rfc3339)
            .map_err(|error| ProviderError::Infrastructure(format!("quick import sync timestamp format failed: {error}")))?,
    }))
}

fn parse_token_expires_at(value: &str) -> ProviderResult<OffsetDateTime> {
    if let Ok(milliseconds) = value.trim().parse::<i128>() {
        let seconds = milliseconds.div_euclid(1000) as i64;
        let nanos = (milliseconds.rem_euclid(1000) as i64) * 1_000_000;
        return OffsetDateTime::from_unix_timestamp(seconds)
            .map(|value| value + time::Duration::nanoseconds(nanos))
            .map_err(|error| ProviderError::InvalidInput(format!("invalid token_expires_at milliseconds: {error}")));
    }
    OffsetDateTime::parse(value.trim(), &Rfc3339).map_err(|error| ProviderError::InvalidInput(format!("invalid token_expires_at: {error}")))
}

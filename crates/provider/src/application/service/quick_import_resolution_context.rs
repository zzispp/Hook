use types::provider::{NewApiQuickImportConfig, ProviderApiKey, ProviderOrigin, ProviderQuickImportSourceConfig};

use crate::application::{
    ProviderError, ProviderQuickImportSyncKey, ProviderQuickImportSyncSource, ProviderRepository, ProviderResult, SecretCipher, UpstreamImportData,
    UpstreamImportToken,
};

pub(super) struct KeyContext {
    pub(super) provider_name: String,
    pub(super) source: ProviderQuickImportSyncSource,
    pub(super) key: ProviderQuickImportSyncKey,
    pub(super) api_key: ProviderApiKey,
}

pub(super) async fn key_context<R>(repository: &R, provider_id: &str, key_id: &str) -> ProviderResult<KeyContext>
where
    R: ProviderRepository,
{
    let provider = repository.find_provider(provider_id).await?.ok_or(ProviderError::NotFound)?;
    if provider.provider_origin != ProviderOrigin::QuickImport {
        return Err(ProviderError::InvalidInput("provider is not a quick import provider".into()));
    }
    let source = repository
        .quick_import_sync_source(provider_id)
        .await?
        .ok_or_else(|| ProviderError::InvalidInput("quick import sync source is not configured".into()))?;
    let key = repository.quick_import_sync_key(provider_id, key_id).await?.ok_or(ProviderError::NotFound)?;
    let api_key = api_key_by_id(repository, provider_id, key_id).await?;
    Ok(KeyContext {
        provider_name: provider.name,
        source,
        key,
        api_key,
    })
}

pub(super) fn source_config<C>(cipher: &C, source: &ProviderQuickImportSyncSource) -> ProviderResult<ProviderQuickImportSourceConfig>
where
    C: SecretCipher,
{
    Ok(ProviderQuickImportSourceConfig::Newapi(NewApiQuickImportConfig {
        base_url: source.base_url.clone(),
        system_access_token: cipher.decrypt_provider_key(&source.encrypted_system_access_token)?,
        user_id: source.user_id.clone(),
    }))
}

pub(super) async fn reject_duplicate_relink<R>(repository: &R, context: &KeyContext, upstream_token_id: &str) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    for key in repository.list_quick_import_sync_keys(&context.source.id).await? {
        if key.key_id != context.key.key_id && key.upstream_token_id == upstream_token_id {
            return Err(ProviderError::InvalidInput(format!("upstream token is already linked: {upstream_token_id}")));
        }
    }
    Ok(())
}

pub(super) fn token_from_data<'a>(data: &'a UpstreamImportData, upstream_token_id: &str) -> ProviderResult<&'a UpstreamImportToken> {
    data.tokens
        .iter()
        .find(|token| token.id == upstream_token_id)
        .ok_or_else(|| ProviderError::InvalidInput(format!("upstream token does not exist: {upstream_token_id}")))
}

async fn api_key_by_id<R>(repository: &R, provider_id: &str, key_id: &str) -> ProviderResult<ProviderApiKey>
where
    R: ProviderRepository,
{
    repository
        .list_api_keys(provider_id)
        .await?
        .into_iter()
        .find(|key| key.id == key_id)
        .ok_or(ProviderError::NotFound)
}

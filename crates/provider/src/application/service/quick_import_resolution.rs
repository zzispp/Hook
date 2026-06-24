use std::collections::{BTreeMap, BTreeSet};

use types::model::GlobalModelResponse;
use types::provider::{
    ProviderApiKey, ProviderQuickImportModelAssociationsResponse, ProviderQuickImportModelAssociationsUpdate, ProviderQuickImportPreviewResponse,
    ProviderQuickImportRelinkRequest, ProviderQuickImportResolutionResponse,
};

use crate::application::{
    GlobalModelCatalog, ProviderError, ProviderQuickImportSyncKey, ProviderRepository, ProviderResult, SecretCipher, UpstreamImportData,
    UpstreamImportToken, UpstreamProviderImportSource,
};

use super::{
    quick_import::QuickImportArgs,
    quick_import_commit::{QuickImportKeyReplacementDraft, quick_import_key_replacement},
    quick_import_preview::{AppendPreviewInput, append_preview_response},
    quick_import_resolution_context::{KeyContext, key_context, reject_duplicate_relink, source_config, token_from_data},
    quick_import_resolution_models::{
        associations, associations_response, current_mappings_for_token, resolve_mappings, token_from_key, validate_associated_models,
        validate_existing_mappings, validate_token,
    },
    quick_import_shared::refreshed_source_patch,
};

pub async fn quick_import_resolution<R, M, C, I>(
    args: QuickImportArgs<'_, R, M, C, I>,
    provider_id: &str,
    key_id: &str,
) -> ProviderResult<ProviderQuickImportResolutionResponse>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    let context = key_context(args.repository, provider_id, key_id).await?;
    let source = refreshed_source(&args, provider_id, &context).await?;
    let data = args.importer.fetch_import_data(&source).await?;
    let globals = args.models.list_global_models().await?;
    let imported = linked_token_ids(args.repository, &context).await?;
    let preview = resolution_preview(&context, data, &globals, &imported);
    let associated = associations(&context.key, &globals)?;
    Ok(ProviderQuickImportResolutionResponse {
        provider_id: provider_id.to_owned(),
        key_id: key_id.to_owned(),
        key_name: context.api_key.name,
        source_kind: context.source.source_kind.clone(),
        current_upstream_token_id: context.key.upstream_token_id,
        current_upstream_group: context.key.upstream_group,
        current_effective_cost_multiplier: context.key.effective_cost_multiplier,
        statuses: context.key.statuses,
        tokens: preview.tokens,
        model_mappings: preview.model_mappings,
        associated_models: associated,
    })
}

pub async fn accept_quick_import_current<R, M, C, I>(args: QuickImportArgs<'_, R, M, C, I>, provider_id: &str, key_id: &str) -> ProviderResult<ProviderApiKey>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    let context = key_context(args.repository, provider_id, key_id).await?;
    let source = refreshed_source(&args, provider_id, &context).await?;
    let data = args.importer.fetch_import_data(&source).await?;
    let token = token_from_data(&data, &context.key.upstream_token_id)?;
    let mappings = accept_current_mappings(&context.key, &context.api_key.allowed_model_ids, token);
    let replacement = replacement(&args, &context, token, mappings).await?;
    Ok(args.repository.replace_quick_import_key(replacement).await?.api_key)
}

pub async fn relink_quick_import_key<R, M, C, I>(
    args: QuickImportArgs<'_, R, M, C, I>,
    provider_id: &str,
    key_id: &str,
    input: ProviderQuickImportRelinkRequest,
) -> ProviderResult<ProviderApiKey>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    let context = key_context(args.repository, provider_id, key_id).await?;
    let source = refreshed_source(&args, provider_id, &context).await?;
    let data = args.importer.fetch_import_data(&source).await?;
    reject_duplicate_relink(args.repository, &context, &input.upstream_token_id).await?;
    let token = token_from_data(&data, &input.upstream_token_id)?;
    let globals = args.models.list_global_models().await?;
    let mappings = resolve_mappings(token, &globals, input.selected_model_ids, input.model_mappings)?;
    let replacement = replacement_with_globals(&args, &context, token, mappings, globals).await?;
    Ok(args.repository.replace_quick_import_key(replacement).await?.api_key)
}

pub async fn quick_import_model_associations<R, M, C, I>(
    args: QuickImportArgs<'_, R, M, C, I>,
    provider_id: &str,
    key_id: &str,
) -> ProviderResult<ProviderQuickImportModelAssociationsResponse>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    let context = key_context(args.repository, provider_id, key_id).await?;
    let globals = args.models.list_global_models().await?;
    let source = refreshed_source(&args, provider_id, &context).await?;
    let upstream_models = args.importer.fetch_sync_token_models(&source, &context.key.upstream_token_id).await?;
    let bindings = args.repository.list_model_bindings(provider_id).await?;
    associations_response(&context, &globals, &upstream_models, &bindings)
}

pub async fn update_quick_import_model_associations<R, M, C, I>(
    args: QuickImportArgs<'_, R, M, C, I>,
    provider_id: &str,
    key_id: &str,
    input: ProviderQuickImportModelAssociationsUpdate,
) -> ProviderResult<ProviderQuickImportModelAssociationsResponse>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    if input.model_mappings.is_empty() {
        return Err(ProviderError::InvalidInput("model_mappings cannot be empty".into()));
    }
    let context = key_context(args.repository, provider_id, key_id).await?;
    let source = refreshed_source(&args, provider_id, &context).await?;
    let upstream_models = args.importer.fetch_sync_token_models(&source, &context.key.upstream_token_id).await?;
    let globals = args.models.list_global_models().await?;
    let token = token_from_key(&context.key, upstream_models.clone());
    let selected_ids = input.model_mappings.iter().map(|item| item.upstream_model_id.clone()).collect();
    let mappings = resolve_mappings(&token, &globals, selected_ids, input.model_mappings)?;
    let replacement = replacement_with_globals(&args, &context, &token, mappings, globals.clone()).await?;
    args.repository.replace_quick_import_key(replacement).await?;
    let bindings = args.repository.list_model_bindings(provider_id).await?;
    associations_response(&context, &globals, &upstream_models, &bindings)
}

async fn refreshed_source<R, M, C, I>(
    args: &QuickImportArgs<'_, R, M, C, I>,
    provider_id: &str,
    context: &KeyContext,
) -> ProviderResult<types::provider::ProviderQuickImportSourceConfig>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    let source_config = source_config(args.cipher, &context.source)?;
    let refreshed = args.importer.refreshed_source_config(&source_config).await?.unwrap_or(source_config);
    args.repository
        .update_quick_import_sync_source(provider_id, refreshed_source_patch(args.cipher, &refreshed)?)
        .await?;
    Ok(refreshed)
}

async fn replacement<R, M, C, I>(
    args: &QuickImportArgs<'_, R, M, C, I>,
    context: &KeyContext,
    token: &UpstreamImportToken,
    mappings: BTreeMap<String, String>,
) -> ProviderResult<crate::application::ProviderQuickImportKeyReplacement>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    let globals = args.models.list_global_models().await?;
    replacement_with_globals(args, context, token, mappings, globals).await
}

async fn replacement_with_globals<R, M, C, I>(
    args: &QuickImportArgs<'_, R, M, C, I>,
    context: &KeyContext,
    token: &UpstreamImportToken,
    mappings: BTreeMap<String, String>,
    globals: Vec<GlobalModelResponse>,
) -> ProviderResult<crate::application::ProviderQuickImportKeyReplacement>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    validate_token(token)?;
    validate_associated_models(token, &mappings)?;
    let bindings = args.repository.list_model_bindings(&context.source.provider_id).await?;
    validate_existing_mappings(&bindings, &mappings)?;
    let existing_global_ids = bindings.iter().map(|item| item.global_model_id.clone()).collect();
    let encrypted = token.api_key.as_deref().map(|key| args.cipher.encrypt_provider_key(key)).transpose()?;
    quick_import_key_replacement(QuickImportKeyReplacementDraft {
        provider_id: context.source.provider_id.clone(),
        source_id: context.source.id.clone(),
        key_id: context.key.key_id.clone(),
        token,
        effective_cost_multiplier: token.group_ratio / context.source.recharge_multiplier,
        globals: &globals,
        mappings,
        encrypted_api_key: encrypted,
        existing_global_model_ids: &existing_global_ids,
    })
}

async fn linked_token_ids<R>(repository: &R, context: &KeyContext) -> ProviderResult<BTreeSet<String>>
where
    R: ProviderRepository,
{
    Ok(repository
        .list_quick_import_sync_keys(&context.source.id)
        .await?
        .into_iter()
        .filter(|key| key.key_id != context.key.key_id)
        .map(|key| key.upstream_token_id)
        .collect())
}

fn resolution_preview(
    context: &KeyContext,
    data: UpstreamImportData,
    globals: &[GlobalModelResponse],
    imported: &BTreeSet<String>,
) -> ProviderQuickImportPreviewResponse {
    append_preview_response(AppendPreviewInput {
        provider_id: context.source.provider_id.clone(),
        provider_name: context.provider_name.clone(),
        source_kind: context.source.source_kind.clone(),
        recharge_multiplier: context.source.recharge_multiplier,
        data,
        globals,
        imported_token_ids: imported,
        linked_keys: &BTreeMap::new(),
        include_linked_tokens: true,
    })
}

pub(super) fn accept_current_mappings(
    key: &ProviderQuickImportSyncKey,
    allowed_model_ids: &[String],
    token: &UpstreamImportToken,
) -> BTreeMap<String, String> {
    current_mappings_for_token(key, allowed_model_ids, token)
}

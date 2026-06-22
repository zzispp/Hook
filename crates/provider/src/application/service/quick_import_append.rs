use std::collections::{BTreeMap, BTreeSet};

use types::provider::{
    Provider, ProviderApiKey, ProviderEndpoint, ProviderModelBinding, ProviderOrigin, ProviderQuickImportAppendCommitRequest,
    ProviderQuickImportAppendPreviewRequest, ProviderQuickImportCommitResponse, ProviderQuickImportLinkedKeyPreview, ProviderQuickImportModelMappingInput,
    ProviderQuickImportPreviewResponse, ProviderQuickImportSourceConfig,
};

use crate::application::{
    GlobalModelCatalog, ProviderError, ProviderQuickImportAppend, ProviderQuickImportSyncKey, ProviderQuickImportSyncSource, ProviderRepository,
    ProviderResult, SecretCipher, UpstreamProviderImportSource,
};

use super::{
    quick_import::QuickImportArgs,
    quick_import_commit::{QuickImportAppendDraft, quick_import_append, resolved_mappings, selected_tokens},
    quick_import_preview::{AppendPreviewInput, append_preview_response},
    quick_import_shared::{refreshed_source_patch, restore_source_config},
};

pub async fn preview_quick_import_append<R, M, C, I>(
    args: QuickImportArgs<'_, R, M, C, I>,
    provider_id: &str,
    input: ProviderQuickImportAppendPreviewRequest,
) -> ProviderResult<ProviderQuickImportPreviewResponse>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    let context = append_context(args.repository, provider_id).await?;
    let source_config = source_config(args.cipher, &context.source)?;
    let refreshed = args.importer.refreshed_source_config(&source_config).await?.unwrap_or(source_config.clone());
    args.repository
        .update_quick_import_sync_source(provider_id, refreshed_source_patch(args.cipher, &refreshed)?)
        .await?;
    let data = args.importer.fetch_import_data(&refreshed).await?;
    let globals = args.models.list_global_models().await?;
    let api_keys = args.repository.list_api_keys(&context.provider.id).await?;
    let imported_ids = imported_token_ids(&context.keys);
    let linked_keys = linked_keys(&context.keys, &api_keys);
    Ok(append_preview_response(AppendPreviewInput {
        provider_id: context.provider.id,
        provider_name: context.provider.name,
        source_kind: context.source.source_kind,
        recharge_multiplier: context.source.recharge_multiplier,
        data,
        globals: &globals,
        imported_token_ids: &imported_ids,
        linked_keys: &linked_keys,
        include_linked_tokens: input.include_linked_tokens,
    }))
}

pub async fn commit_quick_import_append<R, M, C, I>(
    args: QuickImportArgs<'_, R, M, C, I>,
    provider_id: &str,
    input: ProviderQuickImportAppendCommitRequest,
) -> ProviderResult<ProviderQuickImportCommitResponse>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    let context = append_context(args.repository, provider_id).await?;
    let imported_ids = imported_token_ids(&context.keys);
    reject_imported_tokens(&input, &imported_ids)?;
    let source_config = source_config(args.cipher, &context.source)?;
    let refreshed = args.importer.refreshed_source_config(&source_config).await?.unwrap_or(source_config.clone());
    args.repository
        .update_quick_import_sync_source(provider_id, refreshed_source_patch(args.cipher, &refreshed)?)
        .await?;
    let data = args.importer.fetch_import_data(&refreshed).await?;
    let globals = args.models.list_global_models().await?;
    let selected = selected_tokens(&data, &input.selected_tokens)?;
    let mappings = resolved_mappings(&selected, &globals, input.selected_model_ids, input.model_mappings)?;
    let existing_endpoints = args.repository.list_endpoints(&context.provider.id).await?;
    let existing_bindings = args.repository.list_model_bindings(&context.provider.id).await?;
    let draft = quick_import_append(QuickImportAppendDraft {
        provider_id: context.provider.id.clone(),
        source_id: context.source.id,
        source: &refreshed,
        selected,
        globals: &globals,
        mappings,
        cipher: args.cipher,
    })?;
    let draft = filter_existing_resources(draft, &existing_endpoints, &existing_bindings);
    let output = args.repository.append_quick_import(draft).await?;
    Ok(ProviderQuickImportCommitResponse {
        imported_token_count: output.api_keys.len(),
        imported_model_count: output.model_bindings.len(),
        provider: context.provider,
        endpoints: output.endpoints,
        api_keys: output.api_keys,
        model_bindings: output.model_bindings,
        model_costs: output.model_costs,
    })
}

struct AppendContext {
    provider: Provider,
    source: ProviderQuickImportSyncSource,
    keys: Vec<ProviderQuickImportSyncKey>,
}

async fn append_context<R>(repository: &R, provider_id: &str) -> ProviderResult<AppendContext>
where
    R: ProviderRepository,
{
    let provider = repository.find_provider(provider_id).await?.ok_or(ProviderError::NotFound)?;
    if provider.provider_origin != ProviderOrigin::QuickImport {
        return Err(ProviderError::InvalidInput("provider is not a quick import provider".into()));
    }
    let Some(source) = repository.quick_import_sync_source(provider_id).await? else {
        return Err(ProviderError::InvalidInput("quick import sync source is not configured".into()));
    };
    let keys = repository.list_quick_import_sync_keys(&source.id).await?;
    Ok(AppendContext { provider, source, keys })
}

fn source_config<C>(cipher: &C, source: &ProviderQuickImportSyncSource) -> ProviderResult<ProviderQuickImportSourceConfig>
where
    C: SecretCipher,
{
    restore_source_config(cipher, source)
}

fn imported_token_ids(keys: &[ProviderQuickImportSyncKey]) -> BTreeSet<String> {
    keys.iter().map(|key| key.upstream_token_id.clone()).collect()
}

fn linked_keys(keys: &[ProviderQuickImportSyncKey], api_keys: &[ProviderApiKey]) -> BTreeMap<String, ProviderQuickImportLinkedKeyPreview> {
    let api_keys_by_id = api_keys.iter().map(|key| (key.id.as_str(), key)).collect::<BTreeMap<_, _>>();
    keys.iter()
        .filter_map(|key| linked_key_preview(key, api_keys_by_id.get(key.key_id.as_str()).copied()))
        .collect()
}

fn linked_key_preview(key: &ProviderQuickImportSyncKey, api_key: Option<&ProviderApiKey>) -> Option<(String, ProviderQuickImportLinkedKeyPreview)> {
    let api_key = api_key?;
    Some((
        key.upstream_token_id.clone(),
        ProviderQuickImportLinkedKeyPreview {
            key_id: key.key_id.clone(),
            name: api_key.name.clone(),
            endpoint_formats: api_key.api_formats.clone(),
            effective_cost_multiplier: key.effective_cost_multiplier,
            model_mappings: key
                .model_mappings
                .iter()
                .map(|mapping| ProviderQuickImportModelMappingInput {
                    upstream_model_id: mapping.upstream_model_name.clone(),
                    global_model_id: mapping.global_model_id.clone(),
                })
                .collect(),
        },
    ))
}

fn reject_imported_tokens(input: &ProviderQuickImportAppendCommitRequest, imported_ids: &BTreeSet<String>) -> ProviderResult<()> {
    for token in &input.selected_tokens {
        if imported_ids.contains(token.upstream_token_id.trim()) {
            return Err(ProviderError::InvalidInput(format!(
                "upstream token is already linked: {}",
                token.upstream_token_id
            )));
        }
    }
    Ok(())
}

fn filter_existing_resources(
    mut draft: ProviderQuickImportAppend,
    endpoints: &[ProviderEndpoint],
    bindings: &[ProviderModelBinding],
) -> ProviderQuickImportAppend {
    let existing_formats = endpoints.iter().map(|item| item.api_format.as_str()).collect::<BTreeSet<_>>();
    let existing_models = bindings.iter().map(|item| item.global_model_id.as_str()).collect::<BTreeSet<_>>();
    draft.endpoints.retain(|item| !existing_formats.contains(item.api_format.as_str()));
    draft.model_bindings.retain(|item| !existing_models.contains(item.global_model_id.as_str()));
    draft
}

use std::collections::{BTreeMap, BTreeSet};

use types::model::GlobalModelResponse;
use types::provider::{
    ProviderKeyModelMapping, ProviderKeyModelMappingCandidate, ProviderKeyModelMappingInput, ProviderKeyModelMappingsForKeyResponse,
    ProviderKeyModelMappingsResponse, ProviderKeyModelMappingsUpdate,
};

use crate::application::ports::ProviderKeyModelMappingWrite;
use crate::application::{
    GlobalModelCatalog, ProviderError, ProviderRepository, ProviderResult, SecretCipher, UpstreamImportModel, UpstreamProviderImportSource,
};

use super::quick_import::QuickImportArgs;
use super::quick_import_costs::has_default_cost;
use super::quick_import_resolution_context::{key_context, source_config};
use super::quick_import_shared::globals_by_name;

pub async fn key_model_mappings<R>(repository: &R, provider_id: &str) -> ProviderResult<ProviderKeyModelMappingsResponse>
where
    R: ProviderRepository,
{
    repository.find_provider(provider_id).await?.ok_or(ProviderError::NotFound)?;
    let keys = repository.key_model_mappings(provider_id).await?;
    Ok(ProviderKeyModelMappingsResponse {
        provider_id: provider_id.to_owned(),
        keys: keys
            .into_iter()
            .map(|item| types::provider::ProviderKeyModelMappingsByKey {
                provider_id: item.provider_id,
                key_id: item.key_id,
                key_name: item.key_name,
                is_quick_import_key: item.is_quick_import_key,
                mappings: item.mappings,
            })
            .collect(),
    })
}

pub async fn key_model_mappings_for_key<R, M, C, I>(
    args: QuickImportArgs<'_, R, M, C, I>,
    provider_id: &str,
    key_id: &str,
) -> ProviderResult<ProviderKeyModelMappingsForKeyResponse>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    let record = args
        .repository
        .key_model_mappings_for_key(provider_id, key_id)
        .await?
        .ok_or(ProviderError::NotFound)?;
    let globals = args.models.list_global_models().await?;
    let candidates = key_mapping_candidates(args, provider_id, key_id, &record.mappings, &globals).await?;
    Ok(ProviderKeyModelMappingsForKeyResponse {
        provider_id: record.provider_id,
        key_id: record.key_id,
        key_name: record.key_name,
        is_quick_import_key: record.is_quick_import_key,
        mappings: record.mappings,
        candidates,
    })
}

pub async fn update_key_model_mappings<R, M, C, I>(
    args: QuickImportArgs<'_, R, M, C, I>,
    provider_id: &str,
    key_id: &str,
    input: ProviderKeyModelMappingsUpdate,
) -> ProviderResult<ProviderKeyModelMappingsForKeyResponse>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    let record = args
        .repository
        .key_model_mappings_for_key(provider_id, key_id)
        .await?
        .ok_or(ProviderError::NotFound)?;
    let bindings = args.repository.list_model_bindings(provider_id).await?;
    let writes = resolve_key_mapping_writes(provider_id, key_id, &bindings, input.model_mappings)?;
    let mappings = args.repository.replace_key_model_mappings(provider_id, key_id, writes).await?;
    let globals = args.models.list_global_models().await?;
    let candidates = key_mapping_candidates(args, provider_id, key_id, &mappings, &globals).await?;
    Ok(ProviderKeyModelMappingsForKeyResponse {
        provider_id: record.provider_id,
        key_id: record.key_id,
        key_name: record.key_name,
        is_quick_import_key: record.is_quick_import_key,
        mappings,
        candidates,
    })
}

fn resolve_key_mapping_writes(
    provider_id: &str,
    key_id: &str,
    bindings: &[types::provider::ProviderModelBinding],
    inputs: Vec<ProviderKeyModelMappingInput>,
) -> ProviderResult<Vec<ProviderKeyModelMappingWrite>> {
    let binding_ids = bindings
        .iter()
        .map(|binding| (binding.global_model_id.as_str(), binding.id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut seen_global_ids = BTreeSet::new();
    let mut writes = Vec::with_capacity(inputs.len());
    for input in inputs {
        let global_model_id = input.global_model_id.trim();
        let upstream_model_name = input.upstream_model_name.trim();
        if global_model_id.is_empty() || upstream_model_name.is_empty() {
            return Err(ProviderError::InvalidInput("global_model_id and upstream_model_name cannot be blank".into()));
        }
        if !seen_global_ids.insert(global_model_id.to_owned()) {
            return Err(ProviderError::InvalidInput(format!(
                "multiple upstream models map to the same global model: {global_model_id}"
            )));
        }
        let Some(provider_model_id) = binding_ids.get(global_model_id) else {
            return Err(ProviderError::InvalidInput(format!(
                "provider model binding does not exist for global model: {global_model_id}"
            )));
        };
        writes.push(ProviderKeyModelMappingWrite {
            provider_id: provider_id.to_owned(),
            key_id: key_id.to_owned(),
            provider_model_id: (*provider_model_id).to_owned(),
            upstream_model_name: upstream_model_name.to_owned(),
            reasoning_effort: input.reasoning_effort.filter(|value| !value.trim().is_empty()),
        });
    }
    Ok(writes)
}

async fn key_mapping_candidates<R, M, C, I>(
    args: QuickImportArgs<'_, R, M, C, I>,
    provider_id: &str,
    key_id: &str,
    mappings: &[ProviderKeyModelMapping],
    globals: &[GlobalModelResponse],
) -> ProviderResult<Vec<ProviderKeyModelMappingCandidate>>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    let Ok(context) = key_context(args.repository, provider_id, key_id).await else {
        return Ok(Vec::new());
    };
    let source = source_config(args.cipher, &context.source)?;
    let upstream_models = args.importer.fetch_sync_token_models(&source, &context.key.upstream_token_id).await?;
    Ok(candidates(mappings, globals, &upstream_models))
}

fn candidates(
    mappings: &[ProviderKeyModelMapping],
    globals: &[GlobalModelResponse],
    upstream_models: &[UpstreamImportModel],
) -> Vec<ProviderKeyModelMappingCandidate> {
    let by_name = globals_by_name(globals);
    let associated = mappings.iter().map(|item| item.upstream_model_name.as_str()).collect::<BTreeSet<_>>();
    upstream_models
        .iter()
        .filter(|model| !associated.contains(model.id.as_str()))
        .filter_map(|model| {
            let global = by_name.get(&model.id)?;
            if !has_default_cost(global) {
                return None;
            }
            Some(ProviderKeyModelMappingCandidate {
                upstream_model_name: model.id.clone(),
                suggested_global_model_id: Some(global.id.clone()),
                reason: "exact_name_match".into(),
            })
        })
        .collect()
}

use std::collections::{BTreeMap, BTreeSet};

use types::model::GlobalModelResponse;
use types::provider::{
    ProviderModelBinding, ProviderQuickImportModelAssociation, ProviderQuickImportModelAssociationCandidate, ProviderQuickImportModelAssociationsResponse,
    ProviderQuickImportModelMappingInput, ProviderQuickImportSyncStatus,
};

use crate::application::{ProviderError, ProviderQuickImportSyncKey, ProviderResult, UpstreamImportModel, UpstreamImportToken};

use super::{quick_import_costs::has_default_cost, quick_import_resolution_context::KeyContext, quick_import_shared::globals_by_name};

pub(super) fn associations_response(
    context: &KeyContext,
    globals: &[GlobalModelResponse],
    upstream_models: &[UpstreamImportModel],
    bindings: &[ProviderModelBinding],
) -> ProviderResult<ProviderQuickImportModelAssociationsResponse> {
    Ok(ProviderQuickImportModelAssociationsResponse {
        provider_id: context.source.provider_id.clone(),
        key_id: context.key.key_id.clone(),
        key_name: context.api_key.name.clone(),
        source_kind: context.source.source_kind.clone(),
        upstream_token_id: context.key.upstream_token_id.clone(),
        associations: associations(&context.key, globals)?,
        candidates: candidates(&context.key, globals, upstream_models, bindings),
    })
}

pub(super) fn associations(key: &ProviderQuickImportSyncKey, globals: &[GlobalModelResponse]) -> ProviderResult<Vec<ProviderQuickImportModelAssociation>> {
    let by_id = globals.iter().map(|model| (model.id.as_str(), model)).collect::<BTreeMap<_, _>>();
    key.model_mappings.iter().map(|mapping| association(mapping, &by_id)).collect()
}

pub(super) fn token_from_key(key: &ProviderQuickImportSyncKey, models: Vec<UpstreamImportModel>) -> UpstreamImportToken {
    UpstreamImportToken {
        id: key.upstream_token_id.clone(),
        name: key.upstream_token_name.clone(),
        masked_key: String::new(),
        status: 1,
        group: key.upstream_group.clone(),
        group_ratio: key.upstream_group_ratio,
        api_key: None,
        models,
    }
}

pub(super) fn resolve_mappings(
    token: &UpstreamImportToken,
    globals: &[GlobalModelResponse],
    selected_model_ids: Vec<String>,
    inputs: Vec<ProviderQuickImportModelMappingInput>,
) -> ProviderResult<BTreeMap<String, String>> {
    let selected = selected_model_ids
        .into_iter()
        .map(|id| id.trim().to_owned())
        .filter(|id| !id.is_empty())
        .collect::<BTreeSet<_>>();
    if selected.is_empty() {
        return Err(ProviderError::InvalidInput("selected_model_ids cannot be empty".into()));
    }
    let available = token.models.iter().map(|model| model.id.clone()).collect::<BTreeSet<_>>();
    let submitted = submitted_mappings(inputs);
    let by_name = globals_by_name(globals);
    let by_id = globals.iter().map(|model| (model.id.as_str(), model)).collect::<BTreeMap<_, _>>();
    let mut output = BTreeMap::new();
    for upstream_id in selected {
        validate_available_model(&available, &upstream_id)?;
        let global_id = submitted
            .get(&upstream_id)
            .cloned()
            .or_else(|| by_name.get(&upstream_id).map(|model| model.id.clone()));
        let Some(global_id) = global_id else {
            return Err(ProviderError::InvalidInput(format!("model mapping is required: {upstream_id}")));
        };
        if !by_id.contains_key(global_id.as_str()) {
            return Err(ProviderError::InvalidInput(format!("global model does not exist or is inactive: {global_id}")));
        }
        output.insert(upstream_id, global_id);
    }
    assert_no_mapping_conflicts(&output)?;
    Ok(output)
}

pub(super) fn existing_mappings(key: &ProviderQuickImportSyncKey) -> BTreeMap<String, String> {
    key.model_mappings
        .iter()
        .map(|mapping| (mapping.upstream_model_id.clone(), mapping.global_model_id.clone()))
        .collect()
}

pub(super) fn validate_token(token: &UpstreamImportToken) -> ProviderResult<()> {
    if token.status != 1 {
        return Err(ProviderError::InvalidInput(format!("upstream token is disabled: {}", token.id)));
    }
    if token.group.is_none() {
        return Err(ProviderError::InvalidInput(format!("upstream token group is missing: {}", token.id)));
    }
    Ok(())
}

pub(super) fn validate_associated_models(token: &UpstreamImportToken, mappings: &BTreeMap<String, String>) -> ProviderResult<()> {
    let available = token.models.iter().map(|model| model.id.as_str()).collect::<BTreeSet<_>>();
    for upstream_id in mappings.keys() {
        if !available.contains(upstream_id.as_str()) {
            return Err(ProviderError::InvalidInput(format!("associated upstream model is missing: {upstream_id}")));
        }
    }
    Ok(())
}

pub(super) fn validate_existing_mappings(bindings: &[ProviderModelBinding], mappings: &BTreeMap<String, String>) -> ProviderResult<()> {
    for (upstream_id, global_id) in mappings {
        if mapping_conflicts(bindings, global_id, upstream_id) {
            return Err(ProviderError::InvalidInput(format!(
                "provider model mapping conflict for global model: {global_id}"
            )));
        }
    }
    Ok(())
}

pub(super) fn has_hard_quick_import_status(statuses: &[ProviderQuickImportSyncStatus]) -> bool {
    statuses.iter().any(|status| {
        matches!(
            status,
            ProviderQuickImportSyncStatus::SourceFetchFailed
                | ProviderQuickImportSyncStatus::UpstreamTokenDeleted
                | ProviderQuickImportSyncStatus::UpstreamTokenDisabled
                | ProviderQuickImportSyncStatus::UpstreamGroupRemoved
                | ProviderQuickImportSyncStatus::UpstreamGroupChanged
                | ProviderQuickImportSyncStatus::UpstreamModelRemoved
                | ProviderQuickImportSyncStatus::NoAssociatedModels
        )
    })
}

fn association(
    mapping: &crate::application::ProviderQuickImportSyncKeyModel,
    by_id: &BTreeMap<&str, &GlobalModelResponse>,
) -> ProviderResult<ProviderQuickImportModelAssociation> {
    let global = by_id
        .get(mapping.global_model_id.as_str())
        .ok_or_else(|| ProviderError::InvalidInput(format!("global model is missing: {}", mapping.global_model_id)))?;
    Ok(ProviderQuickImportModelAssociation {
        upstream_model_id: mapping.upstream_model_id.clone(),
        global_model_id: global.id.clone(),
        global_model_name: global.name.clone(),
        global_model_display_name: global.display_name.clone(),
    })
}

fn candidates(
    key: &ProviderQuickImportSyncKey,
    globals: &[GlobalModelResponse],
    upstream_models: &[UpstreamImportModel],
    bindings: &[ProviderModelBinding],
) -> Vec<ProviderQuickImportModelAssociationCandidate> {
    let by_name = globals_by_name(globals);
    let associated = key.model_mappings.iter().map(|item| item.upstream_model_id.as_str()).collect::<BTreeSet<_>>();
    upstream_models
        .iter()
        .filter(|model| !associated.contains(model.id.as_str()))
        .filter_map(|model| candidate(model, &by_name, bindings))
        .collect()
}

fn candidate(
    model: &UpstreamImportModel,
    by_name: &BTreeMap<String, &GlobalModelResponse>,
    bindings: &[ProviderModelBinding],
) -> Option<ProviderQuickImportModelAssociationCandidate> {
    let global = by_name.get(&model.id)?;
    if !has_default_cost(global) || mapping_conflicts(bindings, &global.id, &model.id) {
        return None;
    }
    Some(ProviderQuickImportModelAssociationCandidate {
        upstream_model_id: model.id.clone(),
        suggested_global_model_id: Some(global.id.clone()),
        reason: "exact_name_match".into(),
    })
}

fn submitted_mappings(inputs: Vec<ProviderQuickImportModelMappingInput>) -> BTreeMap<String, String> {
    inputs
        .into_iter()
        .map(|input| (input.upstream_model_id.trim().to_owned(), input.global_model_id.trim().to_owned()))
        .filter(|(upstream_id, global_id)| !upstream_id.is_empty() && !global_id.is_empty())
        .collect()
}

fn validate_available_model(available: &BTreeSet<String>, upstream_id: &str) -> ProviderResult<()> {
    if available.contains(upstream_id) {
        return Ok(());
    }
    Err(ProviderError::InvalidInput(format!(
        "selected model does not exist on upstream token: {upstream_id}"
    )))
}

fn mapping_conflicts(bindings: &[ProviderModelBinding], global_id: &str, upstream_id: &str) -> bool {
    bindings
        .iter()
        .find(|binding| binding.global_model_id == global_id)
        .is_some_and(|binding| upstream_model_name(binding) != upstream_id)
}

fn upstream_model_name(binding: &ProviderModelBinding) -> &str {
    binding
        .provider_model_mapping
        .as_ref()
        .map_or(binding.provider_model_name.as_str(), |mapping| mapping.name.as_str())
}

fn assert_no_mapping_conflicts(mappings: &BTreeMap<String, String>) -> ProviderResult<()> {
    let mut seen = BTreeMap::<&str, &str>::new();
    for (upstream_id, global_id) in mappings {
        if let Some(previous) = seen.insert(global_id, upstream_id)
            && previous != upstream_id
        {
            return Err(ProviderError::InvalidInput(format!(
                "multiple upstream models map to the same global model: {global_id}"
            )));
        }
    }
    Ok(())
}

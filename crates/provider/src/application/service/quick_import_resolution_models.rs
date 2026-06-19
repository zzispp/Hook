use std::collections::{BTreeMap, BTreeSet};

use types::model::GlobalModelResponse;
use types::provider::{ProviderKeyModelMapping, ProviderQuickImportModelMappingInput, ProviderQuickImportSyncStatus};

use crate::application::{ProviderError, ProviderQuickImportSyncKey, ProviderResult, UpstreamImportToken};

use super::quick_import_shared::globals_by_name;

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

pub(super) fn current_mappings(key: &ProviderQuickImportSyncKey, allowed_model_ids: &[String]) -> BTreeMap<String, String> {
    let allowed = allowed_model_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    key.model_mappings
        .iter()
        .filter(|mapping| allowed.is_empty() || allowed.contains(mapping.global_model_id.as_str()))
        .map(|mapping| (mapping.upstream_model_name.clone(), mapping.global_model_id.clone()))
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

pub(super) fn validate_existing_mappings(_bindings: &[types::provider::ProviderModelBinding], _mappings: &BTreeMap<String, String>) -> ProviderResult<()> {
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

pub(super) fn associations(key: &ProviderQuickImportSyncKey, by_id: &BTreeMap<&str, &GlobalModelResponse>) -> ProviderResult<Vec<ProviderKeyModelMapping>> {
    key.model_mappings.iter().map(|mapping| association(key, mapping, by_id)).collect()
}

fn association(
    key: &ProviderQuickImportSyncKey,
    mapping: &crate::application::ProviderQuickImportSyncKeyModel,
    by_id: &BTreeMap<&str, &GlobalModelResponse>,
) -> ProviderResult<ProviderKeyModelMapping> {
    let global = by_id
        .get(mapping.global_model_id.as_str())
        .ok_or_else(|| ProviderError::InvalidInput(format!("global model is missing: {}", mapping.global_model_id)))?;
    Ok(ProviderKeyModelMapping {
        id: format!("{}:{}", key.key_id, mapping.provider_model_id),
        provider_id: key.provider_id.clone(),
        key_id: key.key_id.clone(),
        provider_model_id: mapping.provider_model_id.clone(),
        global_model_id: global.id.clone(),
        upstream_model_name: mapping.upstream_model_name.clone(),
        reasoning_effort: mapping.reasoning_effort.clone(),
        created_at: String::new(),
        updated_at: String::new(),
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

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use types::provider::ProviderQuickImportSyncStatus;

    use crate::application::{ProviderQuickImportSyncKey, ProviderQuickImportSyncKeyModel, UpstreamImportModel, UpstreamImportToken};

    use super::{current_mappings, validate_associated_models};

    #[test]
    fn current_mappings_respect_non_empty_key_allowed_models() {
        let key = key_with_mappings(vec![("gpt-5.4", "global-gpt-54"), ("gpt-5.4-mini", "global-gpt-54-mini")]);
        let mappings = current_mappings(&key, &["global-gpt-54".to_owned()]);

        assert_eq!(mappings, std::collections::BTreeMap::from([("gpt-5.4".to_owned(), "global-gpt-54".to_owned())]));
    }

    #[test]
    fn current_mappings_keep_existing_associations_when_key_allowed_models_are_unrestricted() {
        let key = key_with_mappings(vec![("gpt-5.4", "global-gpt-54"), ("gpt-5.4-mini", "global-gpt-54-mini")]);
        let mappings = current_mappings(&key, &[]);

        assert_eq!(mappings.len(), 2);
        assert_eq!(mappings["gpt-5.4"], "global-gpt-54");
        assert_eq!(mappings["gpt-5.4-mini"], "global-gpt-54-mini");
    }

    #[test]
    fn current_mappings_ignore_manually_removed_missing_upstream_models() {
        let key = key_with_mappings(vec![("gpt-5.4", "global-gpt-54"), ("gpt-5.4-mini", "global-gpt-54-mini")]);
        let token = token_with_models(vec!["gpt-5.4"]);
        let mappings = current_mappings(&key, &["global-gpt-54".to_owned()]);

        let result = validate_associated_models(&token, &mappings);

        assert!(result.is_ok());
    }

    fn key_with_mappings(mappings: Vec<(&str, &str)>) -> ProviderQuickImportSyncKey {
        ProviderQuickImportSyncKey {
            provider_id: "provider-1".to_owned(),
            source_id: "source-1".to_owned(),
            key_id: "key-1".to_owned(),
            local_key_name: "codex".to_owned(),
            upstream_token_id: "373".to_owned(),
            upstream_token_name: "codex".to_owned(),
            upstream_group: Some("low-cost".to_owned()),
            upstream_group_ratio: Decimal::ONE,
            effective_cost_multiplier: Decimal::ONE,
            statuses: vec![ProviderQuickImportSyncStatus::UpstreamModelRemoved],
            model_mappings: mappings
                .into_iter()
                .map(|(upstream_model_name, global_model_id)| ProviderQuickImportSyncKeyModel {
                    provider_model_id: format!("provider-{global_model_id}"),
                    global_model_id: global_model_id.to_owned(),
                    upstream_model_name: upstream_model_name.to_owned(),
                    reasoning_effort: None,
                })
                .collect(),
        }
    }

    fn token_with_models(models: Vec<&str>) -> UpstreamImportToken {
        UpstreamImportToken {
            id: "373".to_owned(),
            name: "codex".to_owned(),
            masked_key: "sk-***".to_owned(),
            status: 1,
            group: Some("low-cost".to_owned()),
            group_ratio: Decimal::ONE,
            api_key: Some("sk-test".to_owned()),
            models: models
                .into_iter()
                .map(|id| UpstreamImportModel {
                    id: id.to_owned(),
                    supported_endpoint_types: vec![],
                })
                .collect(),
        }
    }
}

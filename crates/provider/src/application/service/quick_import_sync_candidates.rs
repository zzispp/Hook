use std::collections::{BTreeMap, BTreeSet};

use types::model::GlobalModelResponse;

use crate::application::{ProviderQuickImportSyncKey, UpstreamImportModel};

use super::{quick_import_costs::has_default_cost, quick_import_sync_bindings::BindingInfo};

pub(super) fn candidate_model_ids(
    globals: &BTreeMap<String, GlobalModelResponse>,
    bindings: &BTreeMap<String, BindingInfo>,
    key: &ProviderQuickImportSyncKey,
    upstream_models: &[UpstreamImportModel],
) -> Vec<String> {
    let associated = key.model_mappings.iter().map(|item| item.upstream_model_id.as_str()).collect::<BTreeSet<_>>();
    upstream_models
        .iter()
        .filter(|model| !associated.contains(model.id.as_str()))
        .filter(|model| exact_name_candidate(globals, bindings, model))
        .map(|model| model.id.clone())
        .collect()
}

fn exact_name_candidate(
    globals: &BTreeMap<String, GlobalModelResponse>,
    bindings: &BTreeMap<String, BindingInfo>,
    upstream_model: &UpstreamImportModel,
) -> bool {
    let Some(global) = globals.values().find(|model| model.name == upstream_model.id) else {
        return false;
    };
    has_default_cost(global) && !mapping_conflicts(bindings, &global.id, &upstream_model.id)
}

fn mapping_conflicts(bindings: &BTreeMap<String, BindingInfo>, global_id: &str, upstream_id: &str) -> bool {
    bindings.get(global_id).is_some_and(|binding| binding.upstream_model_id != upstream_id)
}

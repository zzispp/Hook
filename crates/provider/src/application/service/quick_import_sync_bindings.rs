use std::collections::BTreeMap;

use types::provider::ProviderModelBinding;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct BindingInfo {
    pub(super) id: String,
}

pub(super) fn bindings_by_global(bindings: Vec<ProviderModelBinding>) -> BTreeMap<String, BindingInfo> {
    bindings.into_iter().map(binding_info).collect()
}

fn binding_info(binding: ProviderModelBinding) -> (String, BindingInfo) {
    (binding.global_model_id, BindingInfo { id: binding.id })
}

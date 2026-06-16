use std::collections::BTreeMap;

use rust_decimal::Decimal;
use types::{model::GlobalModelResponse, provider::ProviderModelCostUpsert};

use crate::application::{ProviderError, ProviderQuickImportSyncKey, ProviderResult};

use super::{quick_import_costs::model_cost, quick_import_sync_bindings::BindingInfo};

pub(super) fn costs_for_key(
    globals: &BTreeMap<String, GlobalModelResponse>,
    bindings: &BTreeMap<String, BindingInfo>,
    key: &ProviderQuickImportSyncKey,
    multiplier: Decimal,
) -> ProviderResult<Vec<ProviderModelCostUpsert>> {
    key.model_mappings
        .iter()
        .map(|mapping| {
            let global = globals
                .get(&mapping.global_model_id)
                .ok_or_else(|| ProviderError::InvalidInput(format!("global model is missing: {}", mapping.global_model_id)))?;
            let provider_model_id = bindings
                .get(&mapping.global_model_id)
                .ok_or_else(|| ProviderError::InvalidInput(format!("provider model binding is missing: {}", mapping.global_model_id)))?
                .id
                .clone();
            let mut cost = model_cost(global, multiplier)?;
            cost.provider_model_id = provider_model_id;
            Ok(cost)
        })
        .collect()
}

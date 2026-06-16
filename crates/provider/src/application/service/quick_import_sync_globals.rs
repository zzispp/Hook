use std::collections::BTreeMap;

use types::model::GlobalModelResponse;

pub(super) fn globals_by_id(models: Vec<GlobalModelResponse>) -> BTreeMap<String, GlobalModelResponse> {
    models.into_iter().map(|model| (model.id.clone(), model)).collect()
}

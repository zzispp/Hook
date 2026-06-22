use std::collections::BTreeSet;

use types::provider::ProviderQuickImportSourceConfig;

use crate::application::{ProviderQuickImportSyncKey, ProviderResult, UpstreamImportModel, UpstreamProviderImportSource};

pub(super) enum ModelCheck {
    Available {
        upstream_models: Vec<UpstreamImportModel>,
    },
    Removed {
        missing_upstream_model_ids: Vec<String>,
        upstream_models: Vec<UpstreamImportModel>,
    },
}

pub(super) async fn check_models<I>(
    importer: &I,
    source_config: &ProviderQuickImportSourceConfig,
    key: &ProviderQuickImportSyncKey,
) -> ProviderResult<ModelCheck>
where
    I: UpstreamProviderImportSource,
{
    let models = importer.fetch_sync_token_models(source_config, &key.upstream_token_id).await?;
    let available = models.iter().map(|model| model.id.as_str()).collect::<BTreeSet<_>>();
    let missing_upstream_model_ids = key
        .model_mappings
        .iter()
        .filter(|model| !available.contains(model.upstream_model_name.as_str()))
        .map(|model| model.upstream_model_name.clone())
        .collect::<Vec<_>>();
    Ok(if missing_upstream_model_ids.is_empty() {
        ModelCheck::Available { upstream_models: models }
    } else {
        ModelCheck::Removed {
            missing_upstream_model_ids,
            upstream_models: models,
        }
    })
}

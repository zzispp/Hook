use std::collections::BTreeSet;

use types::provider::ProviderQuickImportSourceConfig;

use crate::application::{ProviderQuickImportSyncKey, ProviderResult, UpstreamImportModel, UpstreamProviderImportSource};

pub(super) enum ModelCheck {
    Available(Vec<UpstreamImportModel>),
    Removed,
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
    let missing = key.model_mappings.iter().any(|model| !available.contains(model.upstream_model_id.as_str()));
    Ok(if missing { ModelCheck::Removed } else { ModelCheck::Available(models) })
}

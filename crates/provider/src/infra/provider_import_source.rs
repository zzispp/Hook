use async_trait::async_trait;
use types::provider::ProviderQuickImportSourceConfig;

use crate::application::{ProviderResult, UpstreamImportData, UpstreamImportModel, UpstreamProviderImportSource, UpstreamSyncSnapshot};

use super::{NewApiImportSource, Sub2ApiImportSource};

#[derive(Clone)]
pub struct ProviderImportSource {
    newapi: NewApiImportSource,
    sub2api: Sub2ApiImportSource,
}

impl ProviderImportSource {
    pub fn new() -> ProviderResult<Self> {
        Ok(Self {
            newapi: NewApiImportSource::new()?,
            sub2api: Sub2ApiImportSource::new()?,
        })
    }
}

#[async_trait]
impl UpstreamProviderImportSource for ProviderImportSource {
    async fn fetch_import_data(&self, source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamImportData> {
        match source {
            ProviderQuickImportSourceConfig::Newapi(_) => self.newapi.fetch_import_data(source).await,
            ProviderQuickImportSourceConfig::Sub2api(_) => self.sub2api.fetch_import_data(source).await,
        }
    }

    async fn fetch_sync_snapshot(&self, source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamSyncSnapshot> {
        match source {
            ProviderQuickImportSourceConfig::Newapi(_) => self.newapi.fetch_sync_snapshot(source).await,
            ProviderQuickImportSourceConfig::Sub2api(_) => self.sub2api.fetch_sync_snapshot(source).await,
        }
    }

    async fn fetch_sync_token_models(&self, source: &ProviderQuickImportSourceConfig, upstream_token_id: &str) -> ProviderResult<Vec<UpstreamImportModel>> {
        match source {
            ProviderQuickImportSourceConfig::Newapi(_) => self.newapi.fetch_sync_token_models(source, upstream_token_id).await,
            ProviderQuickImportSourceConfig::Sub2api(_) => self.sub2api.fetch_sync_token_models(source, upstream_token_id).await,
        }
    }
}

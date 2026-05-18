use crate::{Database, StorageResult};

#[derive(Clone)]
pub struct DashboardStore {
    database: Database,
}

impl DashboardStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub(crate) fn database(&self) -> &Database {
        &self.database
    }

    pub async fn overview(&self, query: super::DashboardStoreOverviewQuery) -> StorageResult<types::dashboard::DashboardOverviewResponse> {
        super::overview::overview(self, query).await
    }

    pub async fn activity(&self, query: super::DashboardStoreActivityQuery) -> StorageResult<types::dashboard::DashboardActivityResponse> {
        super::activity::activity(self, query).await
    }

    pub async fn filter_options(&self, query: super::DashboardStoreFilterOptionsQuery) -> StorageResult<types::dashboard::DashboardFilterOptionsResponse> {
        super::filters::filter_options(self, query).await
    }
}

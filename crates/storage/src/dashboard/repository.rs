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

    pub async fn user_stats_leaderboard(
        &self,
        query: super::DashboardUserStatsLeaderboardQuery,
    ) -> StorageResult<types::dashboard::DashboardUserStatsLeaderboardResponse> {
        super::user_stats::leaderboard(self, query).await
    }

    pub async fn user_usage_stats(&self, query: super::DashboardUserUsageStatsQuery) -> StorageResult<types::dashboard::DashboardUserUsageStatsResponse> {
        super::user_stats::summary(self, query).await
    }

    pub async fn user_stats_time_series(
        &self,
        query: super::DashboardUserStatsTimeSeriesQuery,
    ) -> StorageResult<Vec<types::dashboard::DashboardUserStatsTimeSeriesPoint>> {
        super::user_stats::time_series(self, query).await
    }

    pub async fn cost_forecast(&self, query: super::DashboardCostForecastQuery) -> StorageResult<types::dashboard::DashboardCostForecastResponse> {
        super::cost_analysis::forecast(self, query).await
    }

    pub async fn cost_savings(&self, query: super::DashboardCostSavingsQuery) -> StorageResult<types::dashboard::DashboardCostSavingsResponse> {
        super::cost_analysis::savings(self, query).await
    }

    pub async fn api_key_leaderboard(
        &self,
        query: super::DashboardApiKeyLeaderboardQuery,
    ) -> StorageResult<types::dashboard::DashboardApiKeyLeaderboardResponse> {
        super::cost_analysis::api_key_leaderboard(self, query).await
    }

    pub async fn provider_aggregation(
        &self,
        query: super::DashboardProviderAggregationQuery,
    ) -> StorageResult<Vec<types::dashboard::DashboardProviderAggregationItem>> {
        super::cost_analysis::provider_aggregation(self, query).await
    }
}

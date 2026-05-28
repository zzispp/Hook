use async_trait::async_trait;
use storage::{
    Database, StorageError,
    dashboard::{
        DashboardBucketFilter, DashboardScopeFilter, DashboardStore, DashboardStoreActivityQuery, DashboardStoreFilterOptionsQuery, DashboardStoreOverviewQuery,
    },
};
use types::dashboard::{DashboardActivityResponse, DashboardFilterOptionsResponse, DashboardOverviewResponse};

use crate::application::{
    DashboardActivityQuery, DashboardError, DashboardFilterOptionsQuery, DashboardOverviewQuery, DashboardRepository, DashboardResult,
    DashboardUserStatsLeaderboardQuery, DashboardUserStatsTimeSeriesQuery, DashboardUserUsageStatsQuery,
};

#[derive(Clone)]
pub struct StorageDashboardRepository {
    store: DashboardStore,
}

impl StorageDashboardRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: DashboardStore::new(database),
        }
    }
}

#[async_trait]
impl DashboardRepository for StorageDashboardRepository {
    async fn overview(&self, query: DashboardOverviewQuery) -> DashboardResult<DashboardOverviewResponse> {
        self.store.overview(store_overview_query(query)).await.map_err(storage_error)
    }

    async fn activity(&self, query: DashboardActivityQuery) -> DashboardResult<DashboardActivityResponse> {
        self.store.activity(store_activity_query(query)).await.map_err(storage_error)
    }

    async fn filter_options(&self, query: DashboardFilterOptionsQuery) -> DashboardResult<DashboardFilterOptionsResponse> {
        self.store.filter_options(store_filter_query(query)).await.map_err(storage_error)
    }

    async fn user_stats_leaderboard(
        &self,
        query: DashboardUserStatsLeaderboardQuery,
    ) -> DashboardResult<types::dashboard::DashboardUserStatsLeaderboardResponse> {
        self.store.user_stats_leaderboard(store_user_stats_leaderboard_query(query)).await.map_err(storage_error)
    }

    async fn user_usage_stats(
        &self,
        query: DashboardUserUsageStatsQuery,
    ) -> DashboardResult<types::dashboard::DashboardUserUsageStatsResponse> {
        self.store.user_usage_stats(store_user_usage_stats_query(query)).await.map_err(storage_error)
    }

    async fn user_stats_time_series(
        &self,
        query: DashboardUserStatsTimeSeriesQuery,
    ) -> DashboardResult<Vec<types::dashboard::DashboardUserStatsTimeSeriesPoint>> {
        self.store.user_stats_time_series(store_user_stats_time_series_query(query)).await.map_err(storage_error)
    }
}

fn store_overview_query(query: DashboardOverviewQuery) -> DashboardStoreOverviewQuery {
    DashboardStoreOverviewQuery {
        preset: query.preset,
        scope: store_scope(query.scope),
        started_at: query.window.started_at,
        ended_at: query.window.ended_at,
        today_started_at: query.today_window.started_at,
        today_ended_at: query.today_window.ended_at,
        monthly_started_at: query.monthly_window.started_at,
        monthly_ended_at: query.monthly_window.ended_at,
        bucket: store_bucket(query.bucket),
        include_admin_breakdowns: query.admin,
        include_admin_costs: query.admin,
        tz_offset_minutes: query.tz_offset_minutes,
        daily_page: query.daily_page,
    }
}

fn store_activity_query(query: DashboardActivityQuery) -> DashboardStoreActivityQuery {
    DashboardStoreActivityQuery {
        scope: store_scope(query.scope),
        start_date: query.start_date,
        end_date: query.end_date,
        started_at: query.started_at,
        ended_at: query.ended_at,
        include_admin_costs: query.admin,
        tz_offset_minutes: query.tz_offset_minutes,
    }
}

fn store_filter_query(query: DashboardFilterOptionsQuery) -> DashboardStoreFilterOptionsQuery {
    DashboardStoreFilterOptionsQuery {
        scope: store_scope(query.scope),
    }
}

fn store_user_stats_leaderboard_query(query: DashboardUserStatsLeaderboardQuery) -> storage::dashboard::DashboardUserStatsLeaderboardQuery {
    storage::dashboard::DashboardUserStatsLeaderboardQuery {
        window: store_user_stats_window(query.window),
        metric: query.metric,
        limit: query.limit,
        offset: query.offset,
    }
}

fn store_user_usage_stats_query(query: DashboardUserUsageStatsQuery) -> storage::dashboard::DashboardUserUsageStatsQuery {
    storage::dashboard::DashboardUserUsageStatsQuery {
        window: store_user_stats_window(query.window),
        user_id: query.user_id,
    }
}

fn store_user_stats_time_series_query(query: DashboardUserStatsTimeSeriesQuery) -> storage::dashboard::DashboardUserStatsTimeSeriesQuery {
    storage::dashboard::DashboardUserStatsTimeSeriesQuery {
        window: store_user_stats_window(query.window),
        bucket: store_user_stats_bucket(query.bucket),
        user_id: query.user_id,
    }
}

fn store_user_stats_window(window: crate::application::DashboardUserStatsWindow) -> storage::dashboard::DashboardUserStatsStoreWindow {
    storage::dashboard::DashboardUserStatsStoreWindow {
        start_date: window.start_date,
        end_date: window.end_date,
        started_at: window.started_at,
        ended_at: window.ended_at,
    }
}

fn store_scope(scope: crate::application::DashboardScope) -> DashboardScopeFilter {
    match scope {
        crate::application::DashboardScope::Me { user_id } => DashboardScopeFilter::Me { user_id },
        crate::application::DashboardScope::Global => DashboardScopeFilter::Global,
        crate::application::DashboardScope::User { user_id } => DashboardScopeFilter::User { user_id },
        crate::application::DashboardScope::Token { token_id } => DashboardScopeFilter::Token { token_id },
    }
}

fn store_bucket(bucket: crate::application::DashboardBucket) -> DashboardBucketFilter {
    match bucket {
        crate::application::DashboardBucket::Hour => DashboardBucketFilter::Hour,
        crate::application::DashboardBucket::Day => DashboardBucketFilter::Day,
    }
}

fn store_user_stats_bucket(bucket: crate::application::DashboardUserStatsBucket) -> storage::dashboard::DashboardUserStatsBucket {
    match bucket {
        crate::application::DashboardUserStatsBucket::Hour => storage::dashboard::DashboardUserStatsBucket::Hour,
        crate::application::DashboardUserStatsBucket::Day => storage::dashboard::DashboardUserStatsBucket::Day,
    }
}

fn storage_error(error: StorageError) -> DashboardError {
    DashboardError::Infrastructure(error.to_string())
}

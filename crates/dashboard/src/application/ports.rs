use async_trait::async_trait;
use types::{
    dashboard::{
        DashboardActivityResponse, DashboardFilterOptionsResponse, DashboardOverviewResponse, DashboardPreset, DashboardUserStatsLeaderboardResponse,
        DashboardUserStatsMetric, DashboardUserStatsTimeSeriesPoint, DashboardUserUsageStatsResponse,
    },
    pagination::PageRequest,
};

use super::DashboardResult;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardActor {
    pub user_id: String,
    pub role: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DashboardScope {
    Me { user_id: String },
    Global,
    User { user_id: String },
    Token { token_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DashboardBucket {
    Hour,
    Day,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardOverviewQuery {
    pub preset: DashboardPreset,
    pub scope: DashboardScope,
    pub window: DashboardWindowBounds,
    pub today_window: DashboardWindowBounds,
    pub monthly_window: DashboardWindowBounds,
    pub bucket: DashboardBucket,
    pub admin: bool,
    pub tz_offset_minutes: i32,
    pub daily_page: PageRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardActivityQuery {
    pub scope: DashboardScope,
    pub start_date: time::Date,
    pub end_date: time::Date,
    pub started_at: time::OffsetDateTime,
    pub ended_at: time::OffsetDateTime,
    pub admin: bool,
    pub tz_offset_minutes: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardFilterOptionsQuery {
    pub scope: DashboardScope,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DashboardUserStatsBucket {
    Hour,
    Day,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardUserStatsWindow {
    pub start_date: time::Date,
    pub end_date: time::Date,
    pub started_at: time::OffsetDateTime,
    pub ended_at: time::OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardUserStatsLeaderboardQuery {
    pub window: DashboardUserStatsWindow,
    pub metric: DashboardUserStatsMetric,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardUserUsageStatsQuery {
    pub window: DashboardUserStatsWindow,
    pub user_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardUserStatsTimeSeriesQuery {
    pub window: DashboardUserStatsWindow,
    pub bucket: DashboardUserStatsBucket,
    pub user_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardWindowBounds {
    pub started_at: time::OffsetDateTime,
    pub ended_at: time::OffsetDateTime,
}

/// Reads request-record aggregates for dashboard screens.
#[async_trait]
pub trait DashboardRepository: Send + Sync + 'static {
    async fn overview(&self, query: DashboardOverviewQuery) -> DashboardResult<DashboardOverviewResponse>;
    async fn activity(&self, query: DashboardActivityQuery) -> DashboardResult<DashboardActivityResponse>;
    async fn filter_options(&self, query: DashboardFilterOptionsQuery) -> DashboardResult<DashboardFilterOptionsResponse>;
    async fn user_stats_leaderboard(&self, query: DashboardUserStatsLeaderboardQuery) -> DashboardResult<DashboardUserStatsLeaderboardResponse>;
    async fn user_usage_stats(&self, query: DashboardUserUsageStatsQuery) -> DashboardResult<DashboardUserUsageStatsResponse>;
    async fn user_stats_time_series(&self, query: DashboardUserStatsTimeSeriesQuery) -> DashboardResult<Vec<DashboardUserStatsTimeSeriesPoint>>;
}

#[async_trait]
pub trait DashboardUseCase: Send + Sync + 'static {
    async fn overview(&self, actor: DashboardActor, request: types::dashboard::DashboardOverviewRequest) -> DashboardResult<DashboardOverviewResponse>;
    async fn activity(&self, actor: DashboardActor, request: types::dashboard::DashboardActivityRequest) -> DashboardResult<DashboardActivityResponse>;
    async fn filter_options(
        &self,
        actor: DashboardActor,
        request: types::dashboard::DashboardFilterOptionsRequest,
    ) -> DashboardResult<DashboardFilterOptionsResponse>;
    async fn user_stats_leaderboard(
        &self,
        actor: DashboardActor,
        request: types::dashboard::DashboardUserStatsLeaderboardRequest,
    ) -> DashboardResult<DashboardUserStatsLeaderboardResponse>;
    async fn user_usage_stats(
        &self,
        actor: DashboardActor,
        request: types::dashboard::DashboardUserUsageStatsRequest,
    ) -> DashboardResult<DashboardUserUsageStatsResponse>;
    async fn user_stats_time_series(
        &self,
        actor: DashboardActor,
        request: types::dashboard::DashboardUserStatsTimeSeriesRequest,
    ) -> DashboardResult<Vec<DashboardUserStatsTimeSeriesPoint>>;
}

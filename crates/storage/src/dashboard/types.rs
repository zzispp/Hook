use types::{dashboard::DashboardPreset, pagination::PageRequest};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DashboardScopeFilter {
    Me { user_id: String },
    Global,
    User { user_id: String },
    Token { token_id: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DashboardBucketFilter {
    Hour,
    Day,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardStoreOverviewQuery {
    pub preset: DashboardPreset,
    pub scope: DashboardScopeFilter,
    pub started_at: time::OffsetDateTime,
    pub ended_at: time::OffsetDateTime,
    pub today_started_at: time::OffsetDateTime,
    pub today_ended_at: time::OffsetDateTime,
    pub monthly_started_at: time::OffsetDateTime,
    pub monthly_ended_at: time::OffsetDateTime,
    pub bucket: DashboardBucketFilter,
    pub include_admin_breakdowns: bool,
    pub include_admin_costs: bool,
    pub tz_offset_minutes: i32,
    pub daily_page: PageRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardStoreActivityQuery {
    pub scope: DashboardScopeFilter,
    pub start_date: time::Date,
    pub end_date: time::Date,
    pub started_at: time::OffsetDateTime,
    pub ended_at: time::OffsetDateTime,
    pub include_admin_costs: bool,
    pub tz_offset_minutes: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardStoreFilterOptionsQuery {
    pub scope: DashboardScopeFilter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DashboardUserStatsBucket {
    Hour,
    Day,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardUserStatsStoreWindow {
    pub start_date: time::Date,
    pub end_date: time::Date,
    pub started_at: time::OffsetDateTime,
    pub ended_at: time::OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardUserStatsLeaderboardQuery {
    pub window: DashboardUserStatsStoreWindow,
    pub metric: types::dashboard::DashboardUserStatsMetric,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardUserUsageStatsQuery {
    pub window: DashboardUserStatsStoreWindow,
    pub user_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardUserStatsTimeSeriesQuery {
    pub window: DashboardUserStatsStoreWindow,
    pub bucket: DashboardUserStatsBucket,
    pub user_id: Option<String>,
}

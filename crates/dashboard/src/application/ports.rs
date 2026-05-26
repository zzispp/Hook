use async_trait::async_trait;
use types::dashboard::{DashboardActivityResponse, DashboardFilterOptionsResponse, DashboardOverviewResponse, DashboardPreset};

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
    pub bucket: DashboardBucket,
    pub admin: bool,
    pub tz_offset_minutes: i32,
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
}

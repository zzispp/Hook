use async_trait::async_trait;
use constants::auth::DEFAULT_USER_ROLE;
use types::{
    dashboard::{
        DashboardActivityRequest, DashboardActivityResponse, DashboardBreakdowns, DashboardDailyStats, DashboardFilterOptionsResponse,
        DashboardOverviewRequest, DashboardOverviewResponse, DashboardPreset, DashboardScopeParam, DashboardScopeResponse, DashboardSummary, DashboardWindow,
    },
    pagination::{Page, PageRequest},
};

use super::*;

#[tokio::test]
async fn user_activity_rejects_global_scope() {
    let service = DashboardService::new(RecordingRepository);
    let result = service
        .activity(
            user_actor(),
            DashboardActivityRequest {
                scope: Some(DashboardScopeParam::Global),
                user_id: None,
                token_id: None,
                tz_offset_minutes: 480,
            },
        )
        .await;

    assert!(matches!(result, Err(DashboardError::Forbidden(_))));
}

#[tokio::test]
async fn admin_activity_accepts_user_filter() {
    let service = DashboardService::new(RecordingRepository);
    let response = service
        .activity(
            admin_actor(),
            DashboardActivityRequest {
                scope: Some(DashboardScopeParam::User),
                user_id: Some("target-user".into()),
                token_id: None,
                tz_offset_minutes: 480,
            },
        )
        .await
        .unwrap();

    assert_eq!(response.scope.scope, "user");
    assert_eq!(response.scope.user_id.as_deref(), Some("target-user"));
}

#[tokio::test]
async fn user_overview_uses_me_scope() {
    let service = DashboardService::new(RecordingRepository);
    let response = service
        .overview(
            user_actor(),
            DashboardOverviewRequest {
                preset: DashboardPreset::Today,
                scope: None,
                user_id: None,
                token_id: None,
                tz_offset_minutes: 480,
                page: 1,
                page_size: 10,
            },
        )
        .await
        .unwrap();

    assert_eq!(response.scope.scope, "me");
    assert_eq!(response.scope.user_id.as_deref(), Some("user-1"));
}

#[tokio::test]
async fn admin_overview_accepts_user_filter() {
    let service = DashboardService::new(RecordingRepository);
    let response = service
        .overview(
            admin_actor(),
            DashboardOverviewRequest {
                preset: DashboardPreset::Today,
                scope: Some(DashboardScopeParam::User),
                user_id: Some("target-user".into()),
                token_id: None,
                tz_offset_minutes: 480,
                page: 2,
                page_size: 10,
            },
        )
        .await
        .unwrap();

    assert_eq!(response.scope.scope, "user");
    assert_eq!(response.scope.user_id.as_deref(), Some("target-user"));
    assert_eq!(response.daily.day_page.page, 2);
}

#[tokio::test]
async fn overview_rejects_invalid_page_size() {
    let service = DashboardService::new(RecordingRepository);
    let result = service
        .overview(
            admin_actor(),
            DashboardOverviewRequest {
                preset: DashboardPreset::Today,
                scope: Some(DashboardScopeParam::Global),
                user_id: None,
                token_id: None,
                tz_offset_minutes: 480,
                page: 1,
                page_size: 0,
            },
        )
        .await;

    assert!(matches!(result, Err(DashboardError::InvalidInput(_))));
}

#[test]
fn unknown_role_is_not_admin() {
    let actor = DashboardActor {
        user_id: "user-1".into(),
        role: DEFAULT_USER_ROLE.into(),
    };

    assert!(!is_admin(&actor));
}

struct RecordingRepository;

#[async_trait]
impl DashboardRepository for RecordingRepository {
    async fn overview(&self, query: DashboardOverviewQuery) -> DashboardResult<DashboardOverviewResponse> {
        Ok(DashboardOverviewResponse {
            scope: scope_response(&query.scope),
            preset: query.preset,
            window: test_window(),
            summary: empty_summary(),
            today: empty_summary(),
            monthly: empty_summary(),
            timeseries: Vec::new(),
            daily: daily_stats(query.daily_page),
            breakdowns: DashboardBreakdowns::default(),
        })
    }

    async fn activity(&self, query: DashboardActivityQuery) -> DashboardResult<DashboardActivityResponse> {
        Ok(DashboardActivityResponse {
            scope: scope_response(&query.scope),
            start_date: query.start_date.to_string(),
            end_date: query.end_date.to_string(),
            total_days: 0,
            max_request_count: 0,
            days: Vec::new(),
        })
    }

    async fn filter_options(&self, query: DashboardFilterOptionsQuery) -> DashboardResult<DashboardFilterOptionsResponse> {
        let _scope = query.scope;
        Ok(DashboardFilterOptionsResponse {
            users: Vec::new(),
            tokens: Vec::new(),
        })
    }

    async fn user_stats_leaderboard(
        &self,
        query: DashboardUserStatsLeaderboardQuery,
    ) -> DashboardResult<types::dashboard::DashboardUserStatsLeaderboardResponse> {
        Ok(types::dashboard::DashboardUserStatsLeaderboardResponse {
            items: Vec::new(),
            total: 0,
            metric: query.metric,
            start_date: query.window.start_date.to_string(),
            end_date: query.window.end_date.to_string(),
        })
    }

    async fn user_usage_stats(&self, _query: DashboardUserUsageStatsQuery) -> DashboardResult<types::dashboard::DashboardUserUsageStatsResponse> {
        Ok(types::dashboard::DashboardUserUsageStatsResponse::default())
    }

    async fn user_stats_time_series(
        &self,
        _query: DashboardUserStatsTimeSeriesQuery,
    ) -> DashboardResult<Vec<types::dashboard::DashboardUserStatsTimeSeriesPoint>> {
        Ok(Vec::new())
    }

    async fn cost_forecast(&self, query: DashboardCostForecastQuery) -> DashboardResult<types::dashboard::DashboardCostForecastResponse> {
        Ok(types::dashboard::DashboardCostForecastResponse {
            history: Vec::new(),
            forecast: Vec::new(),
            slope: 0.0,
            intercept: 0.0,
            start_date: query.window.start_date.to_string(),
            end_date: query.window.end_date.to_string(),
        })
    }

    async fn cost_savings(&self, _query: DashboardCostSavingsQuery) -> DashboardResult<types::dashboard::DashboardCostSavingsResponse> {
        Ok(types::dashboard::DashboardCostSavingsResponse::default())
    }

    async fn api_key_leaderboard(&self, query: DashboardApiKeyLeaderboardQuery) -> DashboardResult<types::dashboard::DashboardApiKeyLeaderboardResponse> {
        Ok(types::dashboard::DashboardApiKeyLeaderboardResponse {
            items: Vec::new(),
            total: 0,
            metric: query.metric,
            start_date: query.window.start_date.to_string(),
            end_date: query.window.end_date.to_string(),
        })
    }

    async fn provider_aggregation(
        &self,
        _query: DashboardProviderAggregationQuery,
    ) -> DashboardResult<Vec<types::dashboard::DashboardProviderAggregationItem>> {
        Ok(Vec::new())
    }
}

fn daily_stats(page: PageRequest) -> DashboardDailyStats {
    DashboardDailyStats {
        day_page: Page {
            items: Vec::new(),
            total: 0,
            page: page.page,
            page_size: page.page_size,
        },
        ..DashboardDailyStats::default()
    }
}

fn user_actor() -> DashboardActor {
    DashboardActor {
        user_id: "user-1".into(),
        role: DEFAULT_USER_ROLE.into(),
    }
}

fn admin_actor() -> DashboardActor {
    DashboardActor {
        user_id: "admin-1".into(),
        role: ADMIN_ROLE.into(),
    }
}

fn empty_summary() -> DashboardSummary {
    DashboardSummary {
        request_count: 0,
        success_count: 0,
        failed_count: 0,
        active_count: 0,
        success_rate: 0.0,
        error_rate: 0.0,
        cache_hit_rate: 0.0,
        prompt_tokens: 0,
        completion_tokens: 0,
        cache_creation_input_tokens: 0,
        cache_read_input_tokens: 0,
        total_tokens: 0,
        cache_creation_cost: rust_decimal::Decimal::ZERO,
        cache_read_cost: rust_decimal::Decimal::ZERO,
        total_cost: rust_decimal::Decimal::ZERO,
        upstream_total_cost: rust_decimal::Decimal::ZERO,
        profit: rust_decimal::Decimal::ZERO,
        profit_rate: 0.0,
        avg_latency_ms: None,
        avg_ttfb_ms: None,
        model_count: 0,
        provider_count: 0,
        user_count: 0,
        token_count: 0,
        failover_count: 0,
    }
}

fn test_window() -> DashboardWindow {
    DashboardWindow {
        started_at: "2026-05-18T00:00:00Z".into(),
        ended_at: "2026-05-19T00:00:00Z".into(),
        bucket: "hour".into(),
    }
}

fn scope_response(scope: &DashboardScope) -> DashboardScopeResponse {
    match scope {
        DashboardScope::Me { user_id } => scope_user("me", user_id),
        DashboardScope::Global => DashboardScopeResponse {
            scope: "global".into(),
            user_id: None,
            token_id: None,
        },
        DashboardScope::User { user_id } => scope_user("user", user_id),
        DashboardScope::Token { token_id } => DashboardScopeResponse {
            scope: "token".into(),
            user_id: None,
            token_id: Some(token_id.clone()),
        },
    }
}

fn scope_user(scope: &str, user_id: &str) -> DashboardScopeResponse {
    DashboardScopeResponse {
        scope: scope.into(),
        user_id: Some(user_id.into()),
        token_id: None,
    }
}

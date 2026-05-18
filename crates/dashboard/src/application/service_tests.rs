use async_trait::async_trait;
use constants::auth::DEFAULT_USER_ROLE;
use types::dashboard::{
    DashboardActivityRequest, DashboardActivityResponse, DashboardBreakdowns, DashboardFilterOptionsResponse, DashboardOverviewRequest,
    DashboardOverviewResponse, DashboardPreset, DashboardScopeParam, DashboardScopeResponse, DashboardSummary, DashboardWindow,
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
                tz_offset_minutes: 480,
            },
        )
        .await
        .unwrap();

    assert_eq!(response.scope.scope, "me");
    assert_eq!(response.scope.user_id.as_deref(), Some("user-1"));
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
            timeseries: Vec::new(),
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
        total_tokens: 0,
        total_cost: rust_decimal::Decimal::ZERO,
        avg_latency_ms: None,
        avg_ttfb_ms: None,
        model_count: 0,
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

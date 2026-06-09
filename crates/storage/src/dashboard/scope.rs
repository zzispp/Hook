use sea_orm::Value;
use types::dashboard::DashboardScopeResponse;

use super::DashboardScopeFilter;

pub(super) struct SqlParams {
    pub values: Vec<Value>,
}

impl SqlParams {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn push<T>(&mut self, value: T) -> String
    where
        T: Into<Value>,
    {
        self.values.push(value.into());
        format!("${}", self.values.len())
    }
}

pub(super) fn scope_response(scope: &DashboardScopeFilter) -> DashboardScopeResponse {
    match scope {
        DashboardScopeFilter::Me { user_id } => DashboardScopeResponse {
            scope: "me".into(),
            user_id: Some(user_id.clone()),
            token_id: None,
        },
        DashboardScopeFilter::Global => DashboardScopeResponse {
            scope: "global".into(),
            user_id: None,
            token_id: None,
        },
        DashboardScopeFilter::User { user_id } => DashboardScopeResponse {
            scope: "user".into(),
            user_id: Some(user_id.clone()),
            token_id: None,
        },
        DashboardScopeFilter::Token { token_id } => DashboardScopeResponse {
            scope: "token".into(),
            user_id: None,
            token_id: Some(token_id.clone()),
        },
    }
}

pub(super) fn scoped_metric_bucket_where(
    scope: &DashboardScopeFilter,
    started_at: time::OffsetDateTime,
    ended_at: time::OffsetDateTime,
    granularity: &str,
    params: &mut SqlParams,
) -> String {
    let mut filters = vec![
        "b.source_type = 'request'".into(),
        format!("b.bucket_granularity = {}", params.push(granularity.to_owned())),
        format!("b.bucket_started_at >= {}", params.push(started_at)),
        format!("b.bucket_started_at < {}", params.push(ended_at)),
    ];
    add_metric_scope_filter(scope, params, &mut filters);
    where_clause(filters)
}

pub(super) fn add_metric_scope_filter(scope: &DashboardScopeFilter, params: &mut SqlParams, filters: &mut Vec<String>) {
    match scope {
        DashboardScopeFilter::Me { user_id } | DashboardScopeFilter::User { user_id } => {
            filters.push(format!("b.user_id = {}", params.push(user_id.clone())));
        }
        DashboardScopeFilter::Token { token_id } => {
            filters.push(format!("b.token_id = {}", params.push(token_id.clone())));
        }
        DashboardScopeFilter::Global => {}
    }
}

fn where_clause(filters: Vec<String>) -> String {
    if filters.is_empty() {
        return String::new();
    }
    format!("WHERE {}", filters.join(" AND "))
}

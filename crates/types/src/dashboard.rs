use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum DashboardPreset {
    #[default]
    #[serde(rename = "today")]
    Today,
    #[serde(rename = "7d")]
    SevenDays,
    #[serde(rename = "30d")]
    ThirtyDays,
    #[serde(rename = "90d")]
    NinetyDays,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum DashboardScopeParam {
    #[serde(rename = "me")]
    Me,
    #[serde(rename = "global")]
    Global,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "token")]
    Token,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DashboardOverviewRequest {
    #[serde(default)]
    pub preset: DashboardPreset,
    #[serde(default)]
    pub tz_offset_minutes: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DashboardActivityRequest {
    #[serde(default)]
    pub scope: Option<DashboardScopeParam>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub token_id: Option<String>,
    #[serde(default)]
    pub tz_offset_minutes: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DashboardFilterOptionsRequest {
    #[serde(default)]
    pub tz_offset_minutes: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DashboardScopeResponse {
    pub scope: String,
    pub user_id: Option<String>,
    pub token_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DashboardWindow {
    pub started_at: String,
    pub ended_at: String,
    pub bucket: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardOverviewResponse {
    pub scope: DashboardScopeResponse,
    pub preset: DashboardPreset,
    pub window: DashboardWindow,
    pub summary: DashboardSummary,
    pub timeseries: Vec<DashboardTimeseriesPoint>,
    pub breakdowns: DashboardBreakdowns,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardSummary {
    pub request_count: i64,
    pub success_count: i64,
    pub failed_count: i64,
    pub active_count: i64,
    pub success_rate: f64,
    pub total_tokens: i64,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_cost: Decimal,
    pub avg_latency_ms: Option<f64>,
    pub avg_ttfb_ms: Option<f64>,
    pub model_count: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardTimeseriesPoint {
    pub bucket: String,
    pub request_count: i64,
    pub success_count: i64,
    pub failed_count: i64,
    pub total_tokens: i64,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_cost: Decimal,
    pub avg_latency_ms: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct DashboardBreakdowns {
    pub models: Vec<DashboardBreakdownItem>,
    pub api_formats: Vec<DashboardBreakdownItem>,
    pub tokens: Vec<DashboardBreakdownItem>,
    pub providers: Vec<DashboardBreakdownItem>,
    pub users: Vec<DashboardBreakdownItem>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardBreakdownItem {
    pub id: Option<String>,
    pub name: String,
    pub request_count: i64,
    pub total_tokens: i64,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_cost: Decimal,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardActivityResponse {
    pub scope: DashboardScopeResponse,
    pub start_date: String,
    pub end_date: String,
    pub total_days: usize,
    pub max_request_count: i64,
    pub days: Vec<DashboardActivityDay>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardActivityDay {
    pub date: String,
    pub request_count: i64,
    pub total_tokens: i64,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_cost: Decimal,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardFilterOptionsResponse {
    pub users: Vec<DashboardFilterOption>,
    pub tokens: Vec<DashboardFilterOption>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DashboardFilterOption {
    pub id: String,
    pub name: String,
}

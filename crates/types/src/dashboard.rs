use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::pagination::Page;

const DEFAULT_DAILY_PAGE: u64 = 1;
const DEFAULT_DAILY_PAGE_SIZE: u64 = 10;

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
    pub scope: Option<DashboardScopeParam>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub token_id: Option<String>,
    #[serde(default)]
    pub tz_offset_minutes: i32,
    #[serde(default = "default_daily_page")]
    pub page: u64,
    #[serde(default = "default_daily_page_size")]
    pub page_size: u64,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum DashboardUserStatsMetric {
    #[default]
    #[serde(rename = "requests")]
    Requests,
    #[serde(rename = "tokens")]
    Tokens,
    #[serde(rename = "cost")]
    Cost,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum DashboardUserStatsGranularity {
    #[default]
    #[serde(rename = "day")]
    Day,
    #[serde(rename = "hour")]
    Hour,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DashboardUserStatsLeaderboardRequest {
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub preset: Option<DashboardPreset>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub tz_offset_minutes: i32,
    #[serde(default)]
    pub metric: DashboardUserStatsMetric,
    #[serde(default = "default_user_stats_limit")]
    pub limit: u64,
    #[serde(default)]
    pub offset: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DashboardUserUsageStatsRequest {
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub preset: Option<DashboardPreset>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub tz_offset_minutes: i32,
    #[serde(default)]
    pub user_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DashboardUserStatsTimeSeriesRequest {
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub preset: Option<DashboardPreset>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub tz_offset_minutes: i32,
    #[serde(default)]
    pub granularity: DashboardUserStatsGranularity,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub metric: Option<DashboardUserStatsMetric>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub enum DashboardCostAnalysisPreset {
    #[default]
    #[serde(rename = "last30days")]
    Last30Days,
    #[serde(rename = "today")]
    Today,
    #[serde(rename = "yesterday")]
    Yesterday,
    #[serde(rename = "last7days")]
    Last7Days,
    #[serde(rename = "last90days")]
    Last90Days,
    #[serde(rename = "custom")]
    Custom,
}

impl<'de> Deserialize<'de> for DashboardCostAnalysisPreset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "last30days" | "last30d" => Ok(Self::Last30Days),
            "today" => Ok(Self::Today),
            "yesterday" => Ok(Self::Yesterday),
            "last7days" | "last7d" => Ok(Self::Last7Days),
            "last90days" | "last90d" => Ok(Self::Last90Days),
            "custom" => Ok(Self::Custom),
            _ => Err(serde::de::Error::unknown_variant(
                value.as_str(),
                &["today", "yesterday", "last7days", "last30days", "last90days", "custom"],
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DashboardCostAnalysisRequest {
    #[serde(default)]
    pub preset: DashboardCostAnalysisPreset,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default, deserialize_with = "deserialize_i32_query")]
    pub tz_offset_minutes: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DashboardCostForecastRequest {
    #[serde(flatten)]
    pub range: DashboardCostAnalysisRequest,
    #[serde(default = "default_forecast_days", deserialize_with = "deserialize_u32_query")]
    pub forecast_days: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DashboardCostSavingsRequest {
    #[serde(flatten)]
    pub range: DashboardCostAnalysisRequest,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DashboardApiKeyLeaderboardRequest {
    #[serde(flatten)]
    pub range: DashboardCostAnalysisRequest,
    #[serde(default)]
    pub metric: DashboardUserStatsMetric,
    #[serde(default)]
    pub order: DashboardSortOrder,
    #[serde(default = "default_user_stats_limit", deserialize_with = "deserialize_u64_query")]
    pub limit: u64,
    #[serde(default, deserialize_with = "deserialize_u64_query")]
    pub offset: u64,
    #[serde(default, deserialize_with = "deserialize_bool_query")]
    pub include_inactive: bool,
    #[serde(default, deserialize_with = "deserialize_bool_query")]
    pub exclude_admin: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DashboardProviderAggregationRequest {
    #[serde(flatten)]
    pub range: DashboardCostAnalysisRequest,
    #[serde(default = "default_provider_aggregation_group_by")]
    pub group_by: String,
    #[serde(default = "default_provider_aggregation_limit", deserialize_with = "deserialize_u64_query")]
    pub limit: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum DashboardSortOrder {
    #[default]
    #[serde(rename = "desc")]
    Desc,
    #[serde(rename = "asc")]
    Asc,
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
    pub today: DashboardSummary,
    pub monthly: DashboardSummary,
    pub timeseries: Vec<DashboardTimeseriesPoint>,
    pub daily: DashboardDailyStats,
    pub breakdowns: DashboardBreakdowns,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardSummary {
    pub request_count: i64,
    pub success_count: i64,
    pub failed_count: i64,
    pub active_count: i64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub cache_hit_rate: f64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub cache_creation_input_tokens: i64,
    pub cache_read_input_tokens: i64,
    pub total_tokens: i64,
    #[serde(with = "rust_decimal::serde::float")]
    pub cache_creation_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub cache_read_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub upstream_total_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub profit: Decimal,
    pub profit_rate: f64,
    pub avg_latency_ms: Option<f64>,
    pub avg_ttfb_ms: Option<f64>,
    pub avg_response_headers_ms: Option<f64>,
    pub avg_first_output_ms: Option<f64>,
    pub model_count: i64,
    pub provider_count: i64,
    pub user_count: i64,
    pub token_count: i64,
    pub failover_count: i64,
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
    #[serde(with = "rust_decimal::serde::float")]
    pub upstream_total_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub profit: Decimal,
    pub profit_rate: f64,
    pub avg_latency_ms: Option<f64>,
    pub avg_ttfb_ms: Option<f64>,
    pub avg_response_headers_ms: Option<f64>,
    pub avg_first_output_ms: Option<f64>,
    pub cache_hit_rate: f64,
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
    #[serde(with = "rust_decimal::serde::float")]
    pub upstream_total_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub profit: Decimal,
    pub profit_rate: f64,
    pub avg_latency_ms: Option<f64>,
    pub avg_ttfb_ms: Option<f64>,
    pub avg_response_headers_ms: Option<f64>,
    pub avg_first_output_ms: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardDailyStats {
    pub period: DashboardDailyPeriod,
    pub days: Vec<DashboardDailyStat>,
    pub day_page: Page<DashboardDailyStat>,
    pub model_summary: Vec<DashboardDailyModelSummary>,
    pub provider_summary: Vec<DashboardDailyProviderSummary>,
}

impl Default for DashboardDailyStats {
    fn default() -> Self {
        Self {
            period: DashboardDailyPeriod::default(),
            days: Vec::new(),
            day_page: Page {
                items: Vec::new(),
                total: 0,
                page: DEFAULT_DAILY_PAGE,
                page_size: DEFAULT_DAILY_PAGE_SIZE,
            },
            model_summary: Vec::new(),
            provider_summary: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct DashboardDailyPeriod {
    pub start_date: String,
    pub end_date: String,
    pub days: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardDailyStat {
    pub date: String,
    pub request_count: i64,
    pub total_tokens: i64,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub upstream_total_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub profit: Decimal,
    pub profit_rate: f64,
    pub avg_latency_ms: Option<f64>,
    pub unique_models: usize,
    pub unique_providers: usize,
    pub model_breakdown: Vec<DashboardDailyBreakdownItem>,
    pub provider_breakdown: Vec<DashboardDailyBreakdownItem>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardDailyBreakdownItem {
    pub name: String,
    pub request_count: i64,
    pub total_tokens: i64,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub upstream_total_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub profit: Decimal,
    pub profit_rate: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardDailyModelSummary {
    pub name: String,
    pub request_count: i64,
    pub total_tokens: i64,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub upstream_total_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub profit: Decimal,
    pub profit_rate: f64,
    pub avg_latency_ms: Option<f64>,
    #[serde(with = "rust_decimal::serde::float")]
    pub cost_per_request: Decimal,
    pub tokens_per_request: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardDailyProviderSummary {
    pub name: String,
    pub request_count: i64,
    pub total_tokens: i64,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub upstream_total_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub profit: Decimal,
    pub profit_rate: f64,
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
    #[serde(with = "rust_decimal::serde::float")]
    pub base_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub upstream_total_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub profit: Decimal,
    pub profit_rate: f64,
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

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardUserStatsLeaderboardResponse {
    pub items: Vec<DashboardUserStatsLeaderboardItem>,
    pub total: u64,
    pub metric: DashboardUserStatsMetric,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardUserStatsLeaderboardItem {
    pub rank: u64,
    pub id: String,
    pub name: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub value: Decimal,
    pub requests: i64,
    pub tokens: i64,
    #[serde(with = "rust_decimal::serde::float")]
    pub cost: Decimal,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct DashboardUserUsageStatsResponse {
    pub total_requests: i64,
    pub total_tokens: i64,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_cost: Decimal,
    pub error_rate: f64,
    pub avg_total_latency_ms: Option<f64>,
    pub avg_response_headers_ms: Option<f64>,
    pub avg_first_byte_ms: Option<f64>,
    pub avg_first_output_ms: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardUserStatsTimeSeriesPoint {
    pub date: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_cost: Decimal,
    pub total_requests: i64,
    pub total_tokens: i64,
    pub avg_total_latency_ms: Option<f64>,
    pub avg_response_headers_ms: Option<f64>,
    pub avg_first_byte_ms: Option<f64>,
    pub avg_first_output_ms: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardCostForecastResponse {
    pub history: Vec<DashboardCostForecastPoint>,
    pub forecast: Vec<DashboardCostForecastPoint>,
    pub slope: f64,
    pub intercept: f64,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardCostForecastPoint {
    pub date: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_cost: Decimal,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct DashboardCostSavingsResponse {
    pub cache_read_tokens: i64,
    #[serde(with = "rust_decimal::serde::float")]
    pub cache_read_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub cache_creation_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub estimated_full_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub cache_savings: Decimal,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardApiKeyLeaderboardResponse {
    pub items: Vec<DashboardUserStatsLeaderboardItem>,
    pub total: u64,
    pub metric: DashboardUserStatsMetric,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashboardProviderAggregationItem {
    pub provider_id: Option<String>,
    pub provider_key: String,
    pub provider_identity_source: String,
    pub provider: String,
    pub request_count: i64,
    pub total_tokens: i64,
    pub effective_input_tokens: i64,
    pub total_input_context: i64,
    pub output_tokens: i64,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub actual_cost: Decimal,
    pub avg_response_time_ms: f64,
    pub avg_response_headers_ms: Option<f64>,
    pub avg_first_byte_ms: Option<f64>,
    pub avg_first_output_ms: Option<f64>,
    pub success_rate: f64,
    pub error_count: i64,
    pub cache_creation_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_hit_rate: f64,
}

const fn default_daily_page() -> u64 {
    DEFAULT_DAILY_PAGE
}

const fn default_daily_page_size() -> u64 {
    DEFAULT_DAILY_PAGE_SIZE
}

const fn default_user_stats_limit() -> u64 {
    10
}

const fn default_forecast_days() -> u32 {
    7
}

const fn default_provider_aggregation_limit() -> u64 {
    8
}

fn default_provider_aggregation_group_by() -> String {
    "provider".into()
}

fn deserialize_i32_query<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_query_number(deserializer)
}

fn deserialize_u32_query<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_query_number(deserializer)
}

fn deserialize_u64_query<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_query_number(deserializer)
}

fn deserialize_query_number<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let value = String::deserialize(deserializer)?;
    value.parse::<T>().map_err(serde::de::Error::custom)
}

fn deserialize_bool_query<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    match value.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(serde::de::Error::unknown_variant(value.as_str(), &["true", "false"])),
    }
}

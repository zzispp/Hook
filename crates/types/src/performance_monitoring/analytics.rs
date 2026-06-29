use serde::{Deserialize, Serialize};

use super::{EffectiveTimeRange, PerformanceMonitoringRange, SnapshotGranularity};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PerformanceMonitoringAnalyticsRequest {
    #[serde(default)]
    pub range: PerformanceMonitoringRange,
    pub limit: Option<usize>,
    pub slow_threshold_ms: Option<i64>,
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub api_format: Option<String>,
    pub is_stream: Option<bool>,
    pub needs_conversion: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PerformanceMonitoringAnalyticsResponse {
    pub range: PerformanceMonitoringRange,
    pub effective_range: EffectiveTimeRange,
    pub bucket_granularity: SnapshotGranularity,
    pub percentiles: Vec<PerformancePercentilePoint>,
    pub error_distribution: Vec<ErrorDistributionItem>,
    pub error_trend: Vec<ErrorTrendPoint>,
    pub upstream_performance: UpstreamPerformance,
    pub recent_errors: Vec<RecentPerformanceError>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PerformancePercentilePoint {
    pub bucket_started_at: String,
    pub bucket_ended_at: String,
    pub p50_latency_ms: Option<i64>,
    pub p90_latency_ms: Option<i64>,
    pub p99_latency_ms: Option<i64>,
    pub p50_ttfb_ms: Option<i64>,
    pub p90_ttfb_ms: Option<i64>,
    pub p99_ttfb_ms: Option<i64>,
    pub p50_response_headers_ms: Option<i64>,
    pub p90_response_headers_ms: Option<i64>,
    pub p99_response_headers_ms: Option<i64>,
    pub p50_first_output_ms: Option<i64>,
    pub p90_first_output_ms: Option<i64>,
    pub p99_first_output_ms: Option<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ErrorDistributionItem {
    pub category: String,
    pub count: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ErrorTrendPoint {
    pub bucket_started_at: String,
    pub bucket_ended_at: String,
    pub total: i64,
    pub categories: Vec<ErrorDistributionItem>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UpstreamPerformance {
    pub summary: UpstreamPerformanceSummary,
    pub providers: Vec<UpstreamPerformanceProvider>,
    pub timeline: Vec<UpstreamPerformanceTimelinePoint>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UpstreamPerformanceSummary {
    pub request_count: i64,
    pub success_count: i64,
    pub error_count: i64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub output_tokens: i64,
    pub avg_output_tps: Option<f64>,
    pub avg_ttfb_ms: Option<f64>,
    pub avg_latency_ms: Option<f64>,
    pub avg_response_headers_ms: Option<f64>,
    pub avg_first_output_ms: Option<f64>,
    pub p90_latency_ms: Option<i64>,
    pub p99_latency_ms: Option<i64>,
    pub p90_ttfb_ms: Option<i64>,
    pub p99_ttfb_ms: Option<i64>,
    pub p90_response_headers_ms: Option<i64>,
    pub p99_response_headers_ms: Option<i64>,
    pub p90_first_output_ms: Option<i64>,
    pub p99_first_output_ms: Option<i64>,
    pub tps_sample_count: i64,
    pub latency_sample_count: i64,
    pub ttfb_sample_count: i64,
    pub response_headers_sample_count: i64,
    pub first_output_sample_count: i64,
    pub slow_request_count: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UpstreamPerformanceProvider {
    pub provider_id: String,
    pub provider_name: String,
    pub request_count: i64,
    pub success_count: i64,
    pub error_count: i64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub output_tokens: i64,
    pub avg_output_tps: Option<f64>,
    pub avg_ttfb_ms: Option<f64>,
    pub avg_latency_ms: Option<f64>,
    pub avg_response_headers_ms: Option<f64>,
    pub avg_first_output_ms: Option<f64>,
    pub p90_latency_ms: Option<i64>,
    pub p99_latency_ms: Option<i64>,
    pub p90_ttfb_ms: Option<i64>,
    pub p99_ttfb_ms: Option<i64>,
    pub p90_response_headers_ms: Option<i64>,
    pub p99_response_headers_ms: Option<i64>,
    pub p90_first_output_ms: Option<i64>,
    pub p99_first_output_ms: Option<i64>,
    pub tps_sample_count: i64,
    pub latency_sample_count: i64,
    pub ttfb_sample_count: i64,
    pub response_headers_sample_count: i64,
    pub first_output_sample_count: i64,
    pub slow_request_count: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UpstreamPerformanceTimelinePoint {
    pub bucket_started_at: String,
    pub bucket_ended_at: String,
    pub provider_id: String,
    pub provider_name: String,
    pub request_count: i64,
    pub success_count: i64,
    pub error_count: i64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub output_tokens: i64,
    pub avg_output_tps: Option<f64>,
    pub avg_ttfb_ms: Option<f64>,
    pub avg_latency_ms: Option<f64>,
    pub avg_response_headers_ms: Option<f64>,
    pub avg_first_output_ms: Option<f64>,
    pub slow_request_count: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RecentPerformanceError {
    pub created_at: String,
    pub request_id: String,
    pub provider_id: Option<String>,
    pub provider_name: Option<String>,
    pub model: Option<String>,
    pub status_code: Option<i32>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub response_headers_ms: Option<i64>,
    pub first_output_ms: Option<i64>,
    pub latency_ms: Option<i64>,
    pub ttfb_ms: Option<i64>,
}

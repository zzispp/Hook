use serde::{Deserialize, Serialize};

use super::{EffectiveTimeRange, PerformanceMonitoringRange, SnapshotGranularity};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PerformanceMonitoringOverviewRequest {
    #[serde(default)]
    pub range: PerformanceMonitoringRange,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PerformanceMonitoringOverviewResponse {
    pub range: PerformanceMonitoringRange,
    pub effective_range: EffectiveTimeRange,
    pub bucket_granularity: SnapshotGranularity,
    pub max_series_points: usize,
    pub status: SnapshotDataStatus,
    pub series: Vec<PerformanceSnapshotPoint>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PerformanceMonitoringRealtimeResponse {
    pub snapshot: Option<PerformanceSnapshotPoint>,
    pub host: HostRealtimeMetrics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotDataStatus {
    Ready,
    EmptySnapshot,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PerformanceSnapshotMetrics {
    pub core: CoreRequestMetrics,
    pub llm: LlmBusinessMetrics,
    pub network: NetworkConnectionMetrics,
    pub host: HostResourceMetrics,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CoreRequestMetrics {
    pub request_count: i64,
    pub qps: f64,
    pub concurrent_requests: i64,
    pub error_rate: f64,
    pub timeout_rate: f64,
    pub rate_limited_count: i64,
    pub server_error_count: i64,
    pub p50_latency_ms: Option<i64>,
    pub p90_latency_ms: Option<i64>,
    pub p95_latency_ms: Option<i64>,
    pub p99_latency_ms: Option<i64>,
    pub p50_ttfb_ms: Option<i64>,
    pub p90_ttfb_ms: Option<i64>,
    pub p95_ttfb_ms: Option<i64>,
    pub p99_ttfb_ms: Option<i64>,
    pub retry_count: i64,
    pub circuit_breaker_count: i64,
    pub stream_request_count: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LlmBusinessMetrics {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub tokens_per_request: f64,
    pub tokens_per_second: f64,
    pub failover_count: i64,
    pub cache_hit_rate: f64,
    pub quota_limited_count: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NetworkConnectionMetrics {
    pub inbound_bytes: i64,
    pub outbound_bytes: i64,
    pub inbound_bandwidth_bytes_per_second: f64,
    pub outbound_bandwidth_bytes_per_second: f64,
    pub current_connections: Option<i64>,
    pub new_connections_per_second: Option<f64>,
    pub tcp_total: Option<i64>,
    pub tcp_time_wait: Option<i64>,
    pub tcp_established: Option<i64>,
    pub tcp_close_wait: Option<i64>,
    pub retransmits: Option<i64>,
    pub packet_loss: Option<i64>,
    pub status: MetricSupportStatus,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HostResourceMetrics {
    pub cpu_usage_percent: Option<f64>,
    pub load_average_1m: Option<f64>,
    pub load_average_5m: Option<f64>,
    pub load_average_15m: Option<f64>,
    pub memory_rss_bytes: Option<i64>,
    pub memory_usage_bytes: Option<i64>,
    pub disk_total_bytes: Option<i64>,
    pub disk_available_bytes: Option<i64>,
    pub disk_read_bytes_per_second: Option<f64>,
    pub disk_write_bytes_per_second: Option<f64>,
    pub file_descriptors: Option<i64>,
    pub threads: Option<i64>,
    pub processes: Option<i64>,
    pub status: MetricSupportStatus,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricSupportStatus {
    #[default]
    Unsupported,
    Ready,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PerformanceSnapshotPoint {
    pub bucket_started_at: String,
    pub bucket_ended_at: String,
    pub metrics: PerformanceSnapshotMetrics,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HostRealtimeMetrics {
    pub collected_at: String,
    pub metrics: HostResourceMetrics,
}

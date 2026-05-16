use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub const MAX_SERIES_POINTS: usize = 720;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PerformanceMonitoringRange {
    Today,
    #[serde(rename = "7d")]
    SevenDays,
    #[serde(rename = "30d")]
    ThirtyDays,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotGranularity {
    Minute,
    Hour,
    Day,
}

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EffectiveTimeRange {
    pub started_at: String,
    pub ended_at: String,
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
    pub success_rate: f64,
    pub error_rate: f64,
    pub timeout_rate: f64,
    pub rate_limited_count: i64,
    pub server_error_count: i64,
    pub p50_latency_ms: Option<i64>,
    pub p95_latency_ms: Option<i64>,
    pub p99_latency_ms: Option<i64>,
    pub p50_ttft_ms: Option<i64>,
    pub p95_ttft_ms: Option<i64>,
    pub p99_ttft_ms: Option<i64>,
    pub retry_count: i64,
    pub circuit_breaker_count: i64,
    pub stream_request_count: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LlmBusinessMetrics {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub tokens_per_request: f64,
    pub tokens_per_second: f64,
    pub model_distribution: Vec<MetricDimension>,
    pub provider_distribution: Vec<MetricDimension>,
    pub failover_count: i64,
    pub cache_hit_rate: f64,
    #[serde(with = "rust_decimal::serde::float")]
    pub cost: Decimal,
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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MetricDimension {
    pub name: String,
    pub count: i64,
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

impl Default for PerformanceMonitoringRange {
    fn default() -> Self {
        Self::Today
    }
}

impl SnapshotGranularity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Minute => "minute",
            Self::Hour => "hour",
            Self::Day => "day",
        }
    }

    pub const fn bucket_seconds(self) -> i64 {
        match self {
            Self::Minute => 60,
            Self::Hour => 3_600,
            Self::Day => 86_400,
        }
    }
}

impl TryFrom<&str> for SnapshotGranularity {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "minute" => Ok(Self::Minute),
            "hour" => Ok(Self::Hour),
            "day" => Ok(Self::Day),
            _ => Err(format!("unsupported snapshot granularity: {value}")),
        }
    }
}

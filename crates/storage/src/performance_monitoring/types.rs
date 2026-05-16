use types::performance_monitoring::{HostResourceMetrics, NetworkConnectionMetrics, PerformanceSnapshotMetrics, SnapshotGranularity};

#[derive(Clone, Debug, PartialEq)]
pub struct PerformanceSnapshotInput {
    pub bucket_granularity: SnapshotGranularity,
    pub bucket_started_at: time::OffsetDateTime,
    pub bucket_ended_at: time::OffsetDateTime,
    pub metrics: PerformanceSnapshotMetrics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotQueryPlan {
    pub granularity: SnapshotGranularity,
    pub started_at: time::OffsetDateTime,
    pub ended_at: time::OffsetDateTime,
    pub effective_all: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotAggregationWindow {
    pub granularity: SnapshotGranularity,
    pub started_at: time::OffsetDateTime,
    pub ended_at: time::OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SystemMetricsSnapshot {
    pub network: NetworkConnectionMetrics,
    pub host: HostResourceMetrics,
}

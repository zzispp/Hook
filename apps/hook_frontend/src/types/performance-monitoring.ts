export type PerformanceMonitoringRange = 'realtime' | 'today' | '7d' | '30d' | 'all';
export type SnapshotGranularity = 'minute' | 'hour' | 'day';
export type SnapshotDataStatus = 'ready' | 'empty_snapshot';
export type MetricSupportStatus = 'ready' | 'unsupported';

export type EffectiveTimeRange = {
  started_at: string;
  ended_at: string;
};

export type MetricDimension = {
  name: string;
  count: number;
};

export type CoreRequestMetrics = {
  request_count: number;
  qps: number;
  concurrent_requests: number;
  success_rate: number;
  error_rate: number;
  timeout_rate: number;
  rate_limited_count: number;
  server_error_count: number;
  p50_latency_ms?: number | null;
  p95_latency_ms?: number | null;
  p99_latency_ms?: number | null;
  p50_ttft_ms?: number | null;
  p95_ttft_ms?: number | null;
  p99_ttft_ms?: number | null;
  retry_count: number;
  circuit_breaker_count: number;
  stream_request_count: number;
};

export type LlmBusinessMetrics = {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
  tokens_per_request: number;
  tokens_per_second: number;
  model_distribution: MetricDimension[];
  provider_distribution: MetricDimension[];
  failover_count: number;
  cache_hit_rate: number;
  cost: number;
  quota_limited_count: number;
};

export type NetworkConnectionMetrics = {
  inbound_bytes: number;
  outbound_bytes: number;
  inbound_bandwidth_bytes_per_second: number;
  outbound_bandwidth_bytes_per_second: number;
  current_connections?: number | null;
  new_connections_per_second?: number | null;
  tcp_total?: number | null;
  tcp_time_wait?: number | null;
  tcp_established?: number | null;
  tcp_close_wait?: number | null;
  retransmits?: number | null;
  packet_loss?: number | null;
  status: MetricSupportStatus;
};

export type HostResourceMetrics = {
  cpu_usage_percent?: number | null;
  load_average_1m?: number | null;
  load_average_5m?: number | null;
  load_average_15m?: number | null;
  memory_rss_bytes?: number | null;
  memory_usage_bytes?: number | null;
  disk_total_bytes?: number | null;
  disk_available_bytes?: number | null;
  disk_read_bytes_per_second?: number | null;
  disk_write_bytes_per_second?: number | null;
  file_descriptors?: number | null;
  threads?: number | null;
  processes?: number | null;
  status: MetricSupportStatus;
};

export type PerformanceSnapshotMetrics = {
  core: CoreRequestMetrics;
  llm: LlmBusinessMetrics;
  network: NetworkConnectionMetrics;
  host: HostResourceMetrics;
};

export type PerformanceSnapshotPoint = {
  bucket_started_at: string;
  bucket_ended_at: string;
  metrics: PerformanceSnapshotMetrics;
};

export type PerformanceMonitoringOverviewResponse = {
  range: PerformanceMonitoringRange;
  effective_range: EffectiveTimeRange;
  bucket_granularity: SnapshotGranularity;
  max_series_points: number;
  status: SnapshotDataStatus;
  series: PerformanceSnapshotPoint[];
};

export type HostRealtimeMetrics = {
  collected_at: string;
  metrics: HostResourceMetrics;
};

export type PerformanceMonitoringRealtimeResponse = {
  snapshot?: PerformanceSnapshotPoint | null;
  host: HostRealtimeMetrics;
};

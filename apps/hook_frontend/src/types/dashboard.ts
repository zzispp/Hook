import type { PageResponse } from './rbac';

export type DashboardPreset = 'today' | '7d' | '30d' | '90d';

export type DashboardScope = {
  scope: 'me' | 'global' | 'user' | 'token';
  user_id?: string | null;
  token_id?: string | null;
};

export type DashboardWindow = {
  started_at: string;
  ended_at: string;
  bucket: 'hour' | 'day';
};

export type DashboardSummary = {
  request_count: number;
  success_count: number;
  failed_count: number;
  active_count: number;
  success_rate: number;
  error_rate: number;
  cache_hit_rate: number;
  prompt_tokens: number;
  completion_tokens: number;
  cache_creation_input_tokens: number;
  cache_read_input_tokens: number;
  total_tokens: number;
  cache_creation_cost: number;
  cache_read_cost: number;
  total_cost: number;
  upstream_total_cost: number;
  profit: number;
  profit_rate: number;
  avg_latency_ms?: number | null;
  avg_ttfb_ms?: number | null;
  model_count: number;
  provider_count: number;
  user_count: number;
  token_count: number;
  failover_count: number;
};

export type DashboardTimeseriesPoint = {
  bucket: string;
  request_count: number;
  success_count: number;
  failed_count: number;
  total_tokens: number;
  total_cost: number;
  upstream_total_cost: number;
  profit: number;
  profit_rate: number;
  avg_latency_ms?: number | null;
  avg_ttfb_ms?: number | null;
  cache_hit_rate: number;
};

export type DashboardBreakdownItem = {
  id?: string | null;
  name: string;
  request_count: number;
  total_tokens: number;
  total_cost: number;
  upstream_total_cost: number;
  profit: number;
  profit_rate: number;
  avg_latency_ms?: number | null;
};

export type DashboardBreakdowns = {
  models: DashboardBreakdownItem[];
  api_formats: DashboardBreakdownItem[];
  tokens: DashboardBreakdownItem[];
  providers: DashboardBreakdownItem[];
  users: DashboardBreakdownItem[];
};

export type DashboardDailyPeriod = {
  start_date: string;
  end_date: string;
  days: number;
};

export type DashboardDailyBreakdownItem = {
  name: string;
  request_count: number;
  total_tokens: number;
  total_cost: number;
  upstream_total_cost: number;
  profit: number;
  profit_rate: number;
};

export type DashboardDailyStat = {
  date: string;
  request_count: number;
  total_tokens: number;
  total_cost: number;
  upstream_total_cost: number;
  profit: number;
  profit_rate: number;
  avg_latency_ms?: number | null;
  unique_models: number;
  unique_providers: number;
  model_breakdown: DashboardDailyBreakdownItem[];
  provider_breakdown: DashboardDailyBreakdownItem[];
};

export type DashboardDailyModelSummary = {
  name: string;
  request_count: number;
  total_tokens: number;
  total_cost: number;
  upstream_total_cost: number;
  profit: number;
  profit_rate: number;
  avg_latency_ms?: number | null;
  cost_per_request: number;
  tokens_per_request: number;
};

export type DashboardDailyProviderSummary = {
  name: string;
  request_count: number;
  total_tokens: number;
  total_cost: number;
  upstream_total_cost: number;
  profit: number;
  profit_rate: number;
};

export type DashboardDailyStats = {
  period: DashboardDailyPeriod;
  days: DashboardDailyStat[];
  day_page: PageResponse<DashboardDailyStat>;
  model_summary: DashboardDailyModelSummary[];
  provider_summary: DashboardDailyProviderSummary[];
};

export type DashboardOverviewResponse = {
  scope: DashboardScope;
  preset: DashboardPreset;
  window: DashboardWindow;
  summary: DashboardSummary;
  today: DashboardSummary;
  monthly: DashboardSummary;
  timeseries: DashboardTimeseriesPoint[];
  daily: DashboardDailyStats;
  breakdowns: DashboardBreakdowns;
};

export type DashboardActivityDay = {
  date: string;
  request_count: number;
  total_tokens: number;
  total_cost: number;
  base_cost: number;
  upstream_total_cost: number;
  profit: number;
  profit_rate: number;
};

export type DashboardActivityResponse = {
  scope: DashboardScope;
  start_date: string;
  end_date: string;
  total_days: number;
  max_request_count: number;
  days: DashboardActivityDay[];
};

export type DashboardFilterOption = {
  id: string;
  name: string;
};

export type DashboardFilterOptionsResponse = {
  users: DashboardFilterOption[];
  tokens: DashboardFilterOption[];
};

export type DashboardUserStatsMetric = 'requests' | 'tokens' | 'cost';
export type DashboardSortOrder = 'asc' | 'desc';

export type DashboardUserStatsGranularity = 'day' | 'hour';
export type DashboardCostAnalysisPreset =
  | 'today'
  | 'yesterday'
  | 'last7days'
  | 'last30days'
  | 'last90days'
  | 'custom';

export type DashboardUserStatsLeaderboardItem = {
  rank: number;
  id: string;
  name: string;
  value: number;
  requests: number;
  tokens: number;
  cost: number;
};

export type DashboardUserStatsLeaderboardResponse = {
  items: DashboardUserStatsLeaderboardItem[];
  total: number;
  metric: DashboardUserStatsMetric;
  start_date: string;
  end_date: string;
};

export type DashboardUserUsageStatsResponse = {
  total_requests: number;
  total_tokens: number;
  total_cost: number;
  error_rate: number;
};

export type DashboardUserStatsTimeSeriesPoint = {
  date: string;
  total_cost: number;
  total_requests: number;
  total_tokens: number;
};

export type DashboardCostForecastPoint = {
  date: string;
  total_cost: number;
};

export type DashboardCostForecastResponse = {
  history: DashboardCostForecastPoint[];
  forecast: DashboardCostForecastPoint[];
  slope: number;
  intercept: number;
  start_date: string;
  end_date: string;
};

export type DashboardCostSavingsResponse = {
  cache_read_tokens: number;
  cache_read_cost: number;
  cache_creation_cost: number;
  estimated_full_cost: number;
  cache_savings: number;
};

export type DashboardApiKeyLeaderboardResponse = DashboardUserStatsLeaderboardResponse;

export type DashboardProviderAggregationItem = {
  provider_id?: string | null;
  provider_key: string;
  provider_identity_source: string;
  provider: string;
  request_count: number;
  total_tokens: number;
  effective_input_tokens: number;
  total_input_context: number;
  output_tokens: number;
  total_cost: number;
  actual_cost: number;
  avg_response_time_ms: number;
  success_rate: number;
  error_count: number;
  cache_creation_tokens: number;
  cache_read_tokens: number;
  cache_hit_rate: number;
};

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
};

export type DashboardDailyStat = {
  date: string;
  request_count: number;
  total_tokens: number;
  total_cost: number;
  avg_latency_ms?: number | null;
  unique_models: number;
  unique_providers: number;
  model_breakdown: DashboardDailyBreakdownItem[];
};

export type DashboardDailyModelSummary = {
  name: string;
  request_count: number;
  total_tokens: number;
  total_cost: number;
  avg_latency_ms?: number | null;
  cost_per_request: number;
  tokens_per_request: number;
};

export type DashboardDailyProviderSummary = {
  name: string;
  request_count: number;
  total_tokens: number;
  total_cost: number;
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

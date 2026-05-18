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
  total_tokens: number;
  total_cost: number;
  avg_latency_ms?: number | null;
  avg_ttfb_ms?: number | null;
  model_count: number;
};

export type DashboardTimeseriesPoint = {
  bucket: string;
  request_count: number;
  success_count: number;
  failed_count: number;
  total_tokens: number;
  total_cost: number;
  avg_latency_ms?: number | null;
};

export type DashboardBreakdownItem = {
  id?: string | null;
  name: string;
  request_count: number;
  total_tokens: number;
  total_cost: number;
};

export type DashboardBreakdowns = {
  models: DashboardBreakdownItem[];
  api_formats: DashboardBreakdownItem[];
  tokens: DashboardBreakdownItem[];
  providers: DashboardBreakdownItem[];
  users: DashboardBreakdownItem[];
};

export type DashboardOverviewResponse = {
  scope: DashboardScope;
  preset: DashboardPreset;
  window: DashboardWindow;
  summary: DashboardSummary;
  timeseries: DashboardTimeseriesPoint[];
  breakdowns: DashboardBreakdowns;
};

export type DashboardActivityDay = {
  date: string;
  request_count: number;
  total_tokens: number;
  total_cost: number;
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

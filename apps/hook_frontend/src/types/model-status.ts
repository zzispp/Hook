import type { DashboardCostAnalysisPreset } from './dashboard';

export type ModelStatusValue = 'operational' | 'degraded' | 'failed' | 'error';
export type ModelStatusRangePreset = DashboardCostAnalysisPreset;

export type ModelStatusListFilters = {
  preset: ModelStatusRangePreset;
  start_date?: string;
  end_date?: string;
  enabled?: boolean;
  search?: string;
  api_format?: string;
};

export type ModelStatusRunListFilters = {
  search?: string;
  api_format?: string;
  status?: ModelStatusValue | '';
};

export type ModelStatusAvailability = {
  total_checks: number;
  available_checks: number;
  availability_pct?: string | null;
};

export type ModelStatusTimelinePoint = {
  status: ModelStatusValue;
  latency_ms?: number | null;
  status_code?: number | null;
  message?: string | null;
  checked_at: string;
};

export type ModelStatusCheck = {
  id: string;
  name: string;
  global_model_id: string;
  model_name: string;
  api_format: string;
  api_token_id: string;
  api_token_name: string;
  interval_seconds: number;
  enabled: boolean;
  next_due_at: string;
  last_status?: ModelStatusValue | null;
  last_checked_at?: string | null;
  last_latency_ms?: number | null;
  last_message?: string | null;
  availability: ModelStatusAvailability;
  timeline: ModelStatusTimelinePoint[];
  created_at: string;
  updated_at: string;
};

export type ModelStatusCheckListResponse = {
  checks: ModelStatusCheck[];
};

export type ModelStatusRun = {
  id: string;
  check_id: string;
  check_name: string;
  global_model_id: string;
  model_name: string;
  api_format: string;
  api_token_id: string;
  api_token_name: string;
  status: ModelStatusValue;
  latency_ms?: number | null;
  status_code?: number | null;
  message?: string | null;
  checked_at: string;
  created_at: string;
};

export type ModelStatusRunListResponse = {
  items: ModelStatusRun[];
  total: number;
  page: number;
  page_size: number;
};

export type ModelStatusCheckCreate = {
  name: string;
  global_model_id: string;
  api_format: string;
  api_token_id: string;
  interval_seconds: number;
  enabled?: boolean;
};

export type ModelStatusCheckBatchCreate = {
  name_prefix: string;
  global_model_ids: string[];
  api_formats: string[];
  api_token_id: string;
  interval_seconds: number;
  enabled?: boolean;
};

export type ModelStatusCheckUpdate = Partial<ModelStatusCheckCreate>;

export type ModelStatusBatchCreateResponse = {
  success_count: number;
  failed: { global_model_id: string; api_format: string; error: string }[];
};

export type ModelStatusBatchDeleteResponse = {
  success_count: number;
  failed: { id: string; error: string }[];
};

export type ModelStatusBatchUpdateRequest = {
  ids: string[];
  enabled?: boolean;
  interval_seconds?: number;
  name_prefix?: string;
  api_token_id?: string;
};

export type ModelStatusBatchUpdateResponse = {
  success_count: number;
  failed: { id: string; error: string }[];
};

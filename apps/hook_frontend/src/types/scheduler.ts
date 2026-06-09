import type { PageResponse } from './rbac';

export type ScheduledTaskConfigValueType = 'integer';

export type ScheduledTaskConfigField = {
  key: string;
  label_key: string;
  value_type: ScheduledTaskConfigValueType;
  required: boolean;
  min?: number | null;
  max?: number | null;
  unit_key?: string | null;
};

export type ScheduledTask = {
  code: string;
  name_key: string;
  description_key: string;
  enabled: boolean;
  interval_seconds: number;
  next_run_at?: string | null;
  config: Record<string, unknown>;
  config_schema: ScheduledTaskConfigField[];
  last_started_at?: string | null;
  last_finished_at?: string | null;
  last_status?: string | null;
  last_duration_ms?: number | null;
  last_error?: string | null;
  created_at: string;
  updated_at: string;
};

export type ScheduledTaskRun = {
  id: string;
  task_code: string;
  status: string;
  started_at: string;
  finished_at?: string | null;
  duration_ms?: number | null;
  message?: string | null;
  error?: string | null;
};

export type ScheduledTaskUpdate = Partial<{
  enabled: boolean;
  interval_seconds: number;
  config: Record<string, unknown>;
}>;

export type ScheduledTaskRunFilters = {
  status?: string;
  task_code?: string;
};

export type ScheduledTaskRunPage = PageResponse<ScheduledTaskRun>;

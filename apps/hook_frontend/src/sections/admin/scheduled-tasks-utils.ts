import type { AdminFilterState } from './admin-filters-toolbar';
import type { ScheduledTask, ScheduledTaskRunFilters } from 'src/types/scheduler';

import { formatWalletDateTime } from '../wallet/wallet-display';

export type ScheduledTaskTab = 'tasks' | 'runs';
export type RunStatusFilter = '' | 'running' | 'succeeded' | 'failed' | 'skipped_running';
export type TaskFormState = {
  enabled: boolean;
  interval_seconds: string;
  lease_seconds: string;
  config: Record<string, string>;
};

export type Translate = (key: string, options?: Record<string, unknown>) => string;

export const SCHEDULED_TASK_STATUS_VALUES = ['running', 'succeeded', 'failed', 'skipped_running'] as const;

export function taskLabel(t: Translate, task: ScheduledTask) {
  return t(task.name_key);
}

export function taskPayload(task: ScheduledTask, form: TaskFormState) {
  return {
    enabled: form.enabled,
    interval_seconds: Number(form.interval_seconds || 0),
    lease_seconds: Number(form.lease_seconds || 0),
    config: task.config_schema.reduce<Record<string, number>>((acc, field) => {
      acc[field.key] = Number(form.config[field.key] || 0);
      return acc;
    }, {}),
  };
}

export function formFromTask(task: ScheduledTask): TaskFormState {
  return {
    enabled: task.enabled,
    interval_seconds: String(task.interval_seconds),
    lease_seconds: String(task.lease_seconds),
    config: task.config_schema.reduce<Record<string, string>>((acc, field) => {
      acc[field.key] = String(task.config[field.key] ?? '');
      return acc;
    }, {}),
  };
}

export function filterTasks(tasks: ScheduledTask[], filters: AdminFilterState, t: Translate) {
  const search = filters.search.trim().toLowerCase();

  return tasks.filter((task) => {
    if (filters.status === 'enabled' && !task.enabled) {
      return false;
    }
    if (filters.status === 'disabled' && task.enabled) {
      return false;
    }
    if (!search) {
      return true;
    }

    return (
      task.code.toLowerCase().includes(search) ||
      taskLabel(t, task).toLowerCase().includes(search) ||
      t(task.description_key).toLowerCase().includes(search)
    );
  });
}

export function pagedTaskRows(tasks: ScheduledTask[], page: number, pageSize: number) {
  const start = page * pageSize;
  return tasks.slice(start, start + pageSize);
}

export function toRunFilters(taskCode: string, status: RunStatusFilter): ScheduledTaskRunFilters {
  return {
    status: status || undefined,
    task_code: taskCode.trim() || undefined,
  };
}

export function translateTaskStatus(t: Translate, status: string) {
  return SCHEDULED_TASK_STATUS_VALUES.includes(status as (typeof SCHEDULED_TASK_STATUS_VALUES)[number])
    ? t(`scheduledTasks.status.${status}`)
    : status;
}

export function formatOptionalDate(value: string | null | undefined, locale: string) {
  return value ? formatWalletDateTime(value, locale) : '-';
}

export function formatTaskDuration(value: number | null | undefined) {
  if (value === null || value === undefined) {
    return '-';
  }
  return `${value} ms`;
}

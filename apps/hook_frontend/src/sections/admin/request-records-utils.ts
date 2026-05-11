import type { RequestRecord, RequestRecordStatus } from 'src/types/provider';

export const REQUEST_RECORD_STATUS_OPTIONS: RequestRecordStatus[] = [
  'pending',
  'streaming',
  'success',
  'failed',
];

export function requestStatusLabel(status: string, t: (key: string) => string) {
  const key = `requestRecords.status.${status}`;
  return t(key);
}

export function requestStatusColor(status: string) {
  if (status === 'success') return 'success';
  if (status === 'failed') return 'error';
  if (status === 'streaming') return 'info';
  return 'warning';
}

export function billingStatusLabel(status: string, t: (key: string) => string) {
  return t(`requestRecords.billingStatus.${status}`);
}

export function formatDuration(value?: number | null) {
  if (value === null || value === undefined) return 'N/A';
  if (value < 1000) return `${value}ms`;
  return `${(value / 1000).toFixed(2)}s`;
}

export function formatCost(value?: number | null) {
  return `$${Number(value ?? 0).toFixed(6)}`;
}

export function formatTokens(record: RequestRecord) {
  if (record.total_tokens === null || record.total_tokens === undefined) return '-';
  return `${record.total_tokens}`;
}

export function formatRequestDate(value: string, locale: string) {
  return new Intl.DateTimeFormat(locale, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }).format(new Date(value));
}

export function compactId(value: string) {
  return value.length <= 8 ? value : value.slice(0, 8);
}

export function tokenDisplay(record: RequestRecord) {
  if (record.token_prefix) return `${record.token_prefix}...`;
  return record.token_name || '-';
}

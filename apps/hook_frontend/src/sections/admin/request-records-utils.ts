import type { CurrencyDisplay } from 'src/utils/currency-format';
import type { RequestRecord, RequestRecordStatus } from 'src/types/provider';

import { formatMoney } from 'src/utils/currency-format';

import { formatApiFormat } from './provider-management-utils';

const THOUSAND_TOKENS = 1000;
const MILLION_TOKENS = 1000000;
const COMPACT_INTEGER_THRESHOLD = 100;
const COMPACT_ONE_DECIMAL_THRESHOLD = 10;

export const DEFAULT_REQUEST_RECORD_ROWS_PER_PAGE = 20;
export const REQUEST_RECORD_ROWS_PER_PAGE_OPTIONS = [10, DEFAULT_REQUEST_RECORD_ROWS_PER_PAGE, 50];

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

export function formatCost(value: number | null | undefined, display: CurrencyDisplay) {
  return formatMoney(value, display);
}

export function formatTokenCount(value?: number | null) {
  const normalized = Number(value ?? 0);
  if (!Number.isFinite(normalized)) return '0';
  if (normalized < THOUSAND_TOKENS) return String(normalized);
  if (normalized < MILLION_TOKENS) return compactTokenNumber(normalized / THOUSAND_TOKENS, 'K');
  return compactTokenNumber(normalized / MILLION_TOKENS, 'M');
}

export function hasTokenValue(value?: number | null) {
  return value !== null && value !== undefined && value > 0;
}

function compactTokenNumber(value: number, unit: string) {
  if (value >= COMPACT_INTEGER_THRESHOLD) return `${Math.round(value)}${unit}`;
  if (value >= COMPACT_ONE_DECIMAL_THRESHOLD) return `${value.toFixed(1)}${unit}`;
  return `${value.toFixed(2)}${unit}`;
}

export function formatRequestApiFormat(record: RequestRecord) {
  const client = formatApiFormat(record.client_api_format);
  const provider = record.provider_api_format;
  if (!provider || provider === record.client_api_format) return client;
  return `${client} -> ${formatApiFormat(provider)}`;
}

export function userDisplay(record: RequestRecord) {
  return record.username || record.user_id || '-';
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

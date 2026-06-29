import { fData, fNumber, fPercent, fTokenCount } from 'src/utils/format-number';

export function formatMs(value?: number | null) {
  if (value === null || value === undefined) return '-';
  if (value < 1000) return `${fNumber(Math.round(value))}ms`;
  return `${(value / 1000).toFixed(2)}s`;
}

export function formatRate(value?: number | null) {
  return fNumber(value ?? 0, { maximumFractionDigits: 2 });
}

export function formatOptionalRate(value?: number | null) {
  return value === null || value === undefined ? '-' : fNumber(value, { maximumFractionDigits: 2 });
}

export function formatTokens(value?: number | null) {
  return fTokenCount(value ?? 0);
}

export function formatTokenRate(value?: number | null) {
  return `${formatTokens(value)}/s`;
}

export function formatRatio(value?: number | null) {
  return fPercent((value ?? 0) * 100);
}

export function formatBytes(value?: number | null) {
  return value === null || value === undefined ? '-' : fData(value);
}

export function formatPercentNumber(value?: number | null) {
  return value === null || value === undefined ? '-' : `${fNumber(value)}%`;
}

export function valueOrDash(value?: number | null) {
  return value === null || value === undefined ? '-' : fNumber(value);
}

export function formatDateTime(value: string) {
  return new Date(value).toLocaleString();
}

export function safeChartValue(value?: number | null) {
  return value ?? 0;
}

export function round(value: number) {
  return Number(value.toFixed(4));
}

import { formatMoneyCompact } from 'src/utils/currency-format';

export function formatInteger(value: number | undefined, locale: string) {
  return new Intl.NumberFormat(locale, { maximumFractionDigits: 0 }).format(value ?? 0);
}

export function formatDashboardCost(value: number | undefined) {
  return formatMoneyCompact(value);
}

export function formatMs(value?: number | null) {
  if (value === null || value === undefined) return '0ms';
  if (value < 1000) return `${Math.round(value)}ms`;
  return `${(value / 1000).toFixed(2)}s`;
}

export function errorMessage(error: unknown) {
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  return 'Request failed';
}

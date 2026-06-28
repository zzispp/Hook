import { fTokenCount } from 'src/utils/format-number';
import { formatMoneyCompact } from 'src/utils/currency-format';

const DETAILED_COST_MAX_FRACTION_DIGITS = 6;
const MONEY_SYMBOL = '$';
const PERCENT_MULTIPLIER = 100;

export function formatInteger(value: number | undefined, locale: string) {
  return new Intl.NumberFormat(locale, { maximumFractionDigits: 0 }).format(value ?? 0);
}

export function formatPlainInteger(value: number | undefined) {
  return new Intl.NumberFormat('en-US', {
    useGrouping: false,
    maximumFractionDigits: 0,
  }).format(value ?? 0);
}

export function formatDashboardCost(value: number | undefined) {
  return formatMoneyCompact(value);
}

export function formatDashboardCostDetail(value: number | undefined) {
  return `${MONEY_SYMBOL}${new Intl.NumberFormat('en-US', {
    useGrouping: false,
    minimumFractionDigits: 2,
    maximumFractionDigits: DETAILED_COST_MAX_FRACTION_DIGITS,
  }).format(Number(value ?? 0))}`;
}

export function formatDashboardTokens(value: number | undefined) {
  return fTokenCount(value ?? 0);
}

export function formatDashboardPercent(value: number | undefined) {
  return `${((value ?? 0) * PERCENT_MULTIPLIER).toFixed(1)}%`;
}

export function formatMs(value?: number | null) {
  if (value === null || value === undefined) return '-';
  if (value < 1000) return `${Math.round(value)}ms`;
  return `${(value / 1000).toFixed(2)}s`;
}

export function errorMessage(error: unknown) {
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  return 'Request failed';
}

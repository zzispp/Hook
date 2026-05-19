import type { PaletteColorKey } from 'src/theme/core';
import type { IconifyName } from 'src/components/iconify';
import type { DashboardOverviewResponse } from 'src/types/dashboard';

import {
  formatMs,
  formatInteger,
  formatDashboardCost,
  formatDashboardTokens,
} from './dashboard-format';

const PERCENT_MULTIPLIER = 100;
const RATIO_PRECISION = 1;

export type KpiCardData = {
  label: string;
  value: string;
  color: PaletteColorKey;
  icon: IconifyName;
  series: number[];
};

export type KpiCardConfig = {
  labelKey: string;
  color: PaletteColorKey;
  icon: IconifyName;
  value: (summary: DashboardOverviewResponse['summary'] | undefined, locale: string) => string;
  series: (points: DashboardOverviewResponse['timeseries']) => number[];
};

export const KPI_CARD_CONFIGS: KpiCardConfig[] = [
  {
    labelKey: 'dashboard.stats.kpi.requests',
    color: 'primary',
    icon: 'solar:transfer-horizontal-bold-duotone',
    value: (summary, locale) => formatInteger(summary?.request_count, locale),
    series: (points) => points.map((point) => point.request_count),
  },
  {
    labelKey: 'dashboard.stats.kpi.successRate',
    color: 'success',
    icon: 'solar:verified-check-bold',
    value: (summary) => formatPercent(summary?.success_rate ?? 0),
    series: successRateSeries,
  },
  {
    labelKey: 'dashboard.stats.kpi.tokens',
    color: 'warning',
    icon: 'solar:file-text-bold',
    value: (summary) => formatDashboardTokens(summary?.total_tokens),
    series: (points) => points.map((point) => point.total_tokens),
  },
  {
    labelKey: 'dashboard.stats.kpi.cost',
    color: 'info',
    icon: 'solar:bill-list-bold',
    value: (summary) => formatDashboardCost(summary?.total_cost),
    series: (points) => points.map((point) => point.total_cost),
  },
  {
    labelKey: 'dashboard.stats.kpi.active',
    color: 'secondary',
    icon: 'solar:play-circle-bold',
    value: (summary, locale) => formatInteger(summary?.active_count, locale),
    series: (points) => points.map((point) => point.request_count),
  },
  {
    labelKey: 'dashboard.stats.kpi.failed',
    color: 'error',
    icon: 'solar:danger-triangle-bold',
    value: (summary, locale) => formatInteger(summary?.failed_count, locale),
    series: (points) => points.map((point) => point.failed_count),
  },
  {
    labelKey: 'dashboard.stats.kpi.latency',
    color: 'secondary',
    icon: 'solar:clock-circle-bold',
    value: (summary) => formatMs(summary?.avg_latency_ms),
    series: (points) => points.map((point) => point.avg_latency_ms ?? 0),
  },
  {
    labelKey: 'dashboard.stats.kpi.models',
    color: 'primary',
    icon: 'solar:atom-bold-duotone',
    value: (summary, locale) => formatInteger(summary?.model_count, locale),
    series: (points) => points.map((point) => point.request_count),
  },
];

function successRateSeries(points: DashboardOverviewResponse['timeseries']) {
  return points.map((point) =>
    ratioPercent(point.success_count, point.success_count + point.failed_count)
  );
}

function ratioPercent(value: number, total: number) {
  if (total <= 0) return 0;
  return Number(((value / total) * PERCENT_MULTIPLIER).toFixed(RATIO_PRECISION));
}

function formatPercent(value: number) {
  return `${(value * PERCENT_MULTIPLIER).toFixed(RATIO_PRECISION)}%`;
}

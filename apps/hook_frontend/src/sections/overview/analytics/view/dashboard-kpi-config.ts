import type { PaletteColorKey } from 'src/theme/core';
import type { IconifyName } from 'src/components/iconify';
import type { DashboardOverviewResponse } from 'src/types/dashboard';

import {
  formatMs,
  formatInteger,
  formatDashboardCost,
  formatDashboardTokens,
  formatDashboardPercent,
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
  adminOnly?: boolean;
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
    labelKey: 'dashboard.stats.kpi.upstreamCost',
    color: 'warning',
    icon: 'solar:cart-3-bold',
    value: (summary) => formatDashboardCost(summary?.upstream_total_cost),
    series: (points) => points.map((point) => point.upstream_total_cost),
    adminOnly: true,
  },
  {
    labelKey: 'dashboard.stats.kpi.profitRate',
    color: 'success',
    icon: 'solar:double-alt-arrow-up-bold-duotone',
    value: (summary) => formatDashboardPercent(summary?.profit_rate),
    series: (points) => points.map((point) => ratioValuePercent(point.profit_rate)),
    adminOnly: true,
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
    labelKey: 'dashboard.stats.kpi.cacheHitRate',
    color: 'success',
    icon: 'solar:chart-square-outline',
    value: (summary) => formatPercent(summary?.cache_hit_rate ?? 0),
    series: (points) => points.map((point) => ratioValuePercent(point.cache_hit_rate)),
  },
  {
    labelKey: 'dashboard.stats.kpi.ttfb',
    color: 'primary',
    icon: 'solar:clock-circle-outline',
    value: (summary) => formatMs(summary?.avg_ttfb_ms),
    series: (points) => points.map((point) => point.avg_ttfb_ms ?? 0),
  },
];

function successRateSeries(points: DashboardOverviewResponse['timeseries']) {
  return points.map((point) =>
    ratioPercent(point.success_count, point.success_count + point.failed_count)
  );
}

function ratioPercent(value: number, total: number) {
  if (total <= 0) return 0;
  return ratioValuePercent(value / total);
}

function ratioValuePercent(value: number) {
  return Number((value * PERCENT_MULTIPLIER).toFixed(RATIO_PRECISION));
}

function formatPercent(value: number) {
  return formatDashboardPercent(value);
}

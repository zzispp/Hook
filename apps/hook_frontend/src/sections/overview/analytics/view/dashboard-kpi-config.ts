import type { TFunction } from 'i18next';
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

export type KpiCardData = {
  label: string;
  value: string;
  detail?: string;
  color: PaletteColorKey;
  icon: IconifyName;
  series: number[];
};

export type KpiCardConfig = {
  label: (t: TFunction<'admin'>, period: string) => string;
  color: PaletteColorKey;
  icon: IconifyName;
  value: (summary: DashboardOverviewResponse['summary'] | undefined, locale: string) => string;
  detail?: (
    summary: DashboardOverviewResponse['summary'] | undefined,
    t: TFunction<'admin'>
  ) => string;
  series: (points: DashboardOverviewResponse['timeseries']) => number[];
  adminOnly?: boolean;
  userOnly?: boolean;
};

export const KPI_CARD_CONFIGS: KpiCardConfig[] = [
  {
    label: (t, period) => t('dashboard.stats.period.requests', { period }),
    color: 'primary',
    icon: 'solar:transfer-horizontal-bold-duotone',
    value: (summary, locale) => formatInteger(summary?.request_count, locale),
    series: emptySeries,
  },
  {
    label: (t, period) => t('dashboard.stats.period.tokens', { period }),
    color: 'warning',
    icon: 'solar:file-text-bold',
    value: (summary) => formatDashboardTokens(summary?.total_tokens),
    series: emptySeries,
  },
  {
    label: (t, period) => t('dashboard.stats.period.cacheHitRate', { period }),
    color: 'info',
    icon: 'solar:chart-square-outline',
    value: (summary) => formatDashboardPercent(summary?.cache_hit_rate),
    series: emptySeries,
    adminOnly: true,
  },
  {
    label: (t, period) => t('dashboard.stats.period.firstToken', { period }),
    color: 'secondary',
    icon: 'solar:clock-circle-bold',
    value: (summary) => formatMs(summary?.avg_first_output_ms),
    series: (points) => points.map((point) => point.avg_first_output_ms ?? 0),
  },
  {
    label: (t, period) => t('dashboard.stats.period.apiKeys', { period }),
    color: 'success',
    icon: 'solar:shield-keyhole-bold-duotone',
    value: (summary, locale) => formatInteger(summary?.token_count, locale),
    series: emptySeries,
    userOnly: true,
  },
  {
    label: (t, period) => t('dashboard.stats.period.cost', { period }),
    color: 'info',
    icon: 'solar:bill-list-bold',
    value: (summary) => formatDashboardCost(summary?.total_cost),
    series: emptySeries,
    adminOnly: true,
  },
  {
    label: (t, period) => t('dashboard.stats.period.cost', { period }),
    color: 'info',
    icon: 'solar:bill-list-bold',
    value: (summary) => formatDashboardCost(summary?.total_cost),
    series: emptySeries,
    userOnly: true,
  },
];

function emptySeries() {
  return [];
}

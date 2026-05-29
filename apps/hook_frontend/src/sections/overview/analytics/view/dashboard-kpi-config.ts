import type { TFunction } from 'i18next';
import type { PaletteColorKey } from 'src/theme/core';
import type { IconifyName } from 'src/components/iconify';
import type { DashboardOverviewResponse } from 'src/types/dashboard';

import {
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
  labelKey: string;
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
    labelKey: 'dashboard.stats.today.requests',
    color: 'primary',
    icon: 'solar:transfer-horizontal-bold-duotone',
    value: (summary, locale) => formatInteger(summary?.request_count, locale),
    series: emptySeries,
  },
  {
    labelKey: 'dashboard.stats.today.tokens',
    color: 'warning',
    icon: 'solar:file-text-bold',
    value: (summary) => formatDashboardTokens(summary?.total_tokens),
    series: emptySeries,
  },
  {
    labelKey: 'dashboard.stats.today.activeUsers',
    color: 'success',
    icon: 'solar:users-group-rounded-bold',
    value: (summary, locale) => formatInteger(summary?.user_count, locale),
    series: emptySeries,
    adminOnly: true,
  },
  {
    labelKey: 'dashboard.stats.today.apiKeys',
    color: 'success',
    icon: 'solar:shield-keyhole-bold-duotone',
    value: (summary, locale) => formatInteger(summary?.token_count, locale),
    series: emptySeries,
    userOnly: true,
  },
  {
    labelKey: 'dashboard.stats.today.cost',
    color: 'info',
    icon: 'solar:bill-list-bold',
    value: (summary) => formatDashboardCost(summary?.total_cost),
    detail: (summary, t) =>
      [
        t('dashboard.stats.period.upstreamCost', {
          value: formatDashboardCost(summary?.upstream_total_cost),
        }),
        `${t('dashboard.stats.kpi.cacheHitRate')} ${formatDashboardPercent(summary?.cache_hit_rate)}`,
      ].join('\n'),
    series: emptySeries,
    adminOnly: true,
  },
  {
    labelKey: 'dashboard.stats.today.cost',
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

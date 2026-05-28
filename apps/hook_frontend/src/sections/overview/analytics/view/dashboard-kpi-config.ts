import type { PaletteColorKey } from 'src/theme/core';
import type { IconifyName } from 'src/components/iconify';
import type { DashboardOverviewResponse } from 'src/types/dashboard';

import { formatInteger, formatDashboardCost, formatDashboardTokens } from './dashboard-format';

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
    series: emptySeries,
  },
];

function emptySeries() {
  return [];
}

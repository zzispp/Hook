import type { TFunction } from 'i18next';
import type { DashboardPreset } from 'src/types/dashboard';
import type { DashboardUserStatsFilters } from 'src/actions/dashboard';

import { useMemo } from 'react';

import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';

import {
  useDashboardUserUsageStats,
  useDashboardUserStatsTimeSeries,
  useDashboardUserStatsLeaderboard,
  useDashboardCompareUserStatsTimeSeries,
} from 'src/actions/dashboard';

import {
  SummaryCard,
  UserTrendCard,
  LeaderboardCard,
  UserCompareTrendCard,
} from './dashboard-user-stats-cards';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
  preset: DashboardPreset;
  filters: DashboardUserStatsFilters;
  onChange: (filters: DashboardUserStatsFilters) => void;
};

export function DashboardUserStats({ t, locale, preset, filters, onChange }: Props) {
  const requestFilters = useMemo(() => ({ ...filters, preset }), [filters, preset]);
  const leaderboard = useDashboardUserStatsLeaderboard(true, requestFilters);
  const summary = useDashboardUserUsageStats(true, requestFilters);
  const mainTrend = useDashboardUserStatsTimeSeries(true, requestFilters);
  const compareTrend = useDashboardCompareUserStatsTimeSeries(true, requestFilters);

  function patch(next: Partial<DashboardUserStatsFilters>) {
    onChange({ ...filters, ...next });
  }

  return (
    <Stack spacing={3}>
      <Grid container spacing={3}>
        <Grid size={{ xs: 12, lg: 6 }}>
          <LeaderboardCard
            t={t}
            locale={locale}
            metric={filters.metric}
            loading={leaderboard.isLoading}
            page={filters.leaderboardPage}
            total={leaderboard.data?.total ?? 0}
            items={leaderboard.data?.items}
            rowsPerPage={filters.leaderboardPageSize}
            onMetricChange={(metric) => patch({ metric, leaderboardPage: 0 })}
            onPageChange={(page) => patch({ leaderboardPage: page })}
            onRowsPerPageChange={(pageSize) =>
              patch({ leaderboardPage: 0, leaderboardPageSize: pageSize })
            }
          />
        </Grid>
        <Grid size={{ xs: 12, lg: 6 }}>
          <SummaryCard
            t={t}
            locale={locale}
            loading={summary.isLoading}
            data={summary.data}
          />
        </Grid>
      </Grid>
      <UserTrendCard
        t={t}
        title={t('dashboard.stats.userStats.trend')}
        loading={mainTrend.isLoading}
        data={mainTrend.data}
      />
      <UserCompareTrendCard
        t={t}
        loading={compareTrend.isLoading}
        enabled={Boolean(filters.compareUserId)}
        data={compareTrend.data}
      />
    </Stack>
  );
}

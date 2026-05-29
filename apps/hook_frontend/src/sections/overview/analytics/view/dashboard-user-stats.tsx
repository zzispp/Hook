import type { TFunction } from 'i18next';
import type { DashboardUserStatsFilters } from 'src/actions/dashboard';

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
  LeaderboardCard,
} from './dashboard-user-stats-cards';
import { UserTrendCard, UserCompareTrendCard } from './dashboard-user-stats-trend';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
  filters: DashboardUserStatsFilters;
  onChange: (filters: DashboardUserStatsFilters) => void;
};

export function DashboardUserStats({ t, locale, filters, onChange }: Props) {
  const leaderboard = useDashboardUserStatsLeaderboard(true, filters);
  const summary = useDashboardUserUsageStats(true, filters);
  const mainTrend = useDashboardUserStatsTimeSeries(true, filters);
  const compareTrend = useDashboardCompareUserStatsTimeSeries(true, filters);
  const leaderboardData =
    leaderboard.data?.metric === filters.metric ? leaderboard.data : undefined;

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
            total={leaderboardData?.total ?? 0}
            items={leaderboardData?.items}
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

import type { TFunction } from 'i18next';
import type { ChartOptions } from 'src/components/chart';
import type { DashboardUserStatsTimeSeriesPoint } from 'src/types/dashboard';

import Card from '@mui/material/Card';
import Skeleton from '@mui/material/Skeleton';
import CardHeader from '@mui/material/CardHeader';
import { useTheme, alpha as hexAlpha } from '@mui/material/styles';

import { Chart, useChart } from 'src/components/chart';

import { formatMs, formatDashboardCostDetail } from './dashboard-format';

export function UserTrendCard({
  t,
  title,
  loading,
  data,
}: {
  t: TFunction<'admin'>;
  title: string;
  loading: boolean;
  data?: DashboardUserStatsTimeSeriesPoint[];
}) {
  const options = useUserTrendOptions(data);
  return (
    <Card variant="outlined">
      <CardHeader title={title} />
      {loading ? <Skeleton variant="rectangular" height={280} sx={{ m: 2 }} /> : null}
      {!loading ? (
        <Chart
          type="line"
          series={[
            { name: t('dashboard.stats.userStats.totalCost'), data: costSeries(data) },
            { name: t('dashboard.stats.userStats.summary.avgFirstOutput'), data: firstOutputSeries(data) },
          ]}
          options={options}
          sx={{ height: 280, p: 2 }}
        />
      ) : null}
    </Card>
  );
}

export function UserCompareTrendCard({
  t,
  loading,
  enabled,
  data,
}: {
  t: TFunction<'admin'>;
  loading: boolean;
  enabled: boolean;
  data?: DashboardUserStatsTimeSeriesPoint[];
}) {
  if (!enabled) return null;
  return (
    <UserTrendCard
      t={t}
      title={t('dashboard.stats.userStats.compareTrend')}
      loading={loading}
      data={data}
    />
  );
}

function useUserTrendOptions(data?: DashboardUserStatsTimeSeriesPoint[]) {
  const theme = useTheme();
  return useChart({
    colors: [hexAlpha(theme.palette.info.main, 0.88)],
    xaxis: { categories: data?.map((point) => point.date) ?? [], labels: { rotate: -35 } },
    legend: { show: true },
    tooltip: {
      y: {
        formatter: (value: number, context) => formatUserTrendTooltip(value, context),
      },
    },
  } satisfies ChartOptions);
}

function costSeries(data?: DashboardUserStatsTimeSeriesPoint[]) {
  return data?.map((point) => point.total_cost) ?? [];
}

function firstOutputSeries(data?: DashboardUserStatsTimeSeriesPoint[]) {
  return data?.map((point) => point.avg_first_output_ms ?? null) ?? [];
}

function formatUserTrendTooltip(value: number, context?: { seriesIndex?: number }) {
  if (context?.seriesIndex === 0) {
    return formatDashboardCostDetail(value);
  }
  return formatMs(value);
}

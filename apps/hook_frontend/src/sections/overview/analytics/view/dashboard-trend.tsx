import type { TFunction } from 'i18next';
import type { ChartOptions } from 'src/components/chart';
import type { DashboardOverviewResponse } from 'src/types/dashboard';

import Card from '@mui/material/Card';
import Skeleton from '@mui/material/Skeleton';
import CardHeader from '@mui/material/CardHeader';
import { useTheme, alpha as hexAlpha } from '@mui/material/styles';

import { Chart, useChart } from 'src/components/chart';

import { formatDashboardCostDetail } from './dashboard-format';

export function TrendCard({
  t,
  isAdmin,
  loading,
  data,
}: {
  t: TFunction<'admin'>;
  isAdmin: boolean;
  loading: boolean;
  data?: DashboardOverviewResponse;
}) {
  const theme = useTheme();
  const categories = data?.timeseries.map((point) => point.bucket) ?? [];
  const options = useChart({
    colors: [
      hexAlpha(theme.palette.primary.dark, 0.88),
      hexAlpha(theme.palette.success.main, 0.88),
      hexAlpha(theme.palette.error.main, 0.72),
      hexAlpha(theme.palette.warning.main, 0.72),
      hexAlpha(theme.palette.info.main, 0.72),
    ],
    xaxis: { categories, labels: { rotate: -35 } },
    legend: { show: true },
    stroke: { width: 2, colors: ['transparent'] },
    tooltip: { y: { formatter: (value: number, context) => formatTrendTooltip(value, context) } },
    plotOptions: { bar: { columnWidth: '44%' } },
  } satisfies ChartOptions);

  return (
    <Card>
      <CardHeader title={t('dashboard.stats.trend.title')} />
      {loading ? <Skeleton variant="rectangular" height={364} sx={{ m: 3 }} /> : null}
      {!loading ? (
        <Chart
          type="bar"
          series={series(t, isAdmin, data)}
          options={options}
          slotProps={{ loading: { p: 2.5 } }}
          sx={{ height: 364, pl: 1, py: 2.5, pr: 2.5 }}
        />
      ) : null}
    </Card>
  );
}

function series(t: TFunction<'admin'>, isAdmin: boolean, data?: DashboardOverviewResponse) {
  const requests = [
    {
      name: t('dashboard.stats.trend.requests'),
      data: data?.timeseries.map((point) => point.request_count) ?? [],
    },
    {
      name: t('dashboard.stats.trend.success'),
      data: data?.timeseries.map((point) => point.success_count) ?? [],
    },
    {
      name: t('dashboard.stats.trend.failed'),
      data: data?.timeseries.map((point) => point.failed_count) ?? [],
    },
  ];
  if (!isAdmin) {
    return requests;
  }
  return [
    ...requests,
    {
      name: t('dashboard.stats.trend.upstreamCost'),
      data: data?.timeseries.map((point) => point.upstream_total_cost) ?? [],
    },
    {
      name: t('dashboard.stats.trend.profit'),
      data: data?.timeseries.map((point) => point.profit) ?? [],
    },
  ];
}

function formatTrendTooltip(value: number, options?: { seriesIndex?: number }) {
  if ((options?.seriesIndex ?? 0) >= 3) {
    return formatDashboardCostDetail(value);
  }
  return `${value}`;
}

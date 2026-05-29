'use client';

import type { ChartOptions } from 'src/components/chart';
import type { ErrorTrendPoint, ErrorDistributionItem } from 'src/types/performance-monitoring';

import Card from '@mui/material/Card';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';
import CardHeader from '@mui/material/CardHeader';

import { fNumber } from 'src/utils/format-number';

import { useTranslate } from 'src/locales/use-locales';

import { Chart, useChart, ChartLegends } from 'src/components/chart';

const ERROR_COLORS = ['#ef4444', '#f59e0b', '#06b6d4', '#22c55e', '#8b5cf6', '#64748b'];

export function ErrorDistributionChart({ items }: { items: ErrorDistributionItem[] }) {
  const { t } = useTranslate('admin');
  const options = useDistributionOptions(items);

  return (
    <Card sx={{ height: 1 }}>
      <CardHeader title={t('performanceMonitoring.charts.errorDistribution')} />
      {!items.length ? <EmptyState message={t('performanceMonitoring.empty.noErrorData')} /> : null}
      {items.length ? (
        <>
          <Chart
            type="donut"
            series={items.map((item) => item.count)}
            options={options}
            sx={{ my: 3, mx: 'auto', width: 260, height: 260 }}
          />
          <Divider sx={{ borderStyle: 'dashed' }} />
          <ChartLegends
            labels={items.map((item) => item.category)}
            colors={ERROR_COLORS}
            values={items.map((item) => fNumber(item.count))}
            sx={{ p: 3, justifyContent: 'center' }}
          />
        </>
      ) : null}
    </Card>
  );
}

export function ErrorTrendChart({ points }: { points: ErrorTrendPoint[] }) {
  const { t } = useTranslate('admin');
  const options = useTrendOptions(points);

  return (
    <Card>
      <CardHeader title={t('performanceMonitoring.charts.errorTrend')} />
      {!points.length ? (
        <EmptyState message={t('performanceMonitoring.empty.noErrorData')} />
      ) : (
        <Chart type="bar" series={errorTrendSeries(points)} options={options} sx={{ height: 320, p: 2 }} />
      )}
    </Card>
  );
}

function EmptyState({ message }: { message: string }) {
  return (
    <Typography variant="body2" color="text.secondary" sx={{ px: 3, py: 4 }}>
      {message}
    </Typography>
  );
}

function errorTrendSeries(points: ErrorTrendPoint[]) {
  const categories = Array.from(
    new Set(points.flatMap((point) => point.categories.map((item) => item.category)))
  );

  return categories.map((category) => ({
    name: category,
    data: points.map((point) => point.categories.find((item) => item.category === category)?.count ?? 0),
  }));
}

function useDistributionOptions(items: ErrorDistributionItem[]): ChartOptions {
  const { t } = useTranslate('admin');

  return useChart({
    colors: ERROR_COLORS,
    labels: items.map((item) => item.category),
    stroke: { width: 0 },
    tooltip: { y: { formatter: (value: number) => fNumber(value) } },
    noData: { text: t('performanceMonitoring.empty.noErrorData') },
  } satisfies ChartOptions);
}

function useTrendOptions(points: ErrorTrendPoint[]): ChartOptions {
  const { t } = useTranslate('admin');

  return useChart({
    chart: { stacked: true },
    xaxis: { categories: points.map((point) => new Date(point.bucket_started_at).toLocaleString()) },
    legend: { show: true },
    tooltip: { y: { formatter: (value: number) => fNumber(value) } },
    noData: { text: t('performanceMonitoring.empty.noErrorData') },
  } satisfies ChartOptions);
}

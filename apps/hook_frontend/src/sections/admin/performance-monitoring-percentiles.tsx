'use client';

import type { ChartOptions } from 'src/components/chart';
import type { PerformancePercentilePoint } from 'src/types/performance-monitoring';

import Card from '@mui/material/Card';
import Typography from '@mui/material/Typography';
import CardHeader from '@mui/material/CardHeader';

import { useTranslate } from 'src/locales/use-locales';

import { Chart, useChart } from 'src/components/chart';

export function PercentileChart({
  title,
  mode,
  points,
}: {
  title: string;
  mode: PercentileMode;
  points: PerformancePercentilePoint[];
}) {
  const { t } = useTranslate('admin');
  const options = usePercentileOptions(points);

  return (
    <Card>
      <CardHeader title={title} />
      {!points.length ? (
        <Typography variant="body2" color="text.secondary" sx={{ px: 3, py: 4 }}>
          {t('performanceMonitoring.empty.noPercentileData')}
        </Typography>
      ) : (
        <Chart
          type="line"
          series={percentileSeries(points, mode)}
          options={options}
          sx={{ height: 320, p: 2 }}
        />
      )}
    </Card>
  );
}

type PercentileMode = 'latency' | 'ttfb' | 'response_headers' | 'first_output';

function percentileSeries(points: PerformancePercentilePoint[], mode: PercentileMode) {
  return [
    { name: 'P50', data: points.map((point) => percentileValue(point, mode, 'p50')) },
    { name: 'P90', data: points.map((point) => percentileValue(point, mode, 'p90')) },
    { name: 'P99', data: points.map((point) => percentileValue(point, mode, 'p99')) },
  ];
}

function percentileValue(
  point: PerformancePercentilePoint,
  mode: PercentileMode,
  percentile: 'p50' | 'p90' | 'p99'
) {
  const key = `${percentile}_${mode}_ms` as keyof PerformancePercentilePoint;
  const value = point[key];
  return typeof value === 'number' ? value : null;
}

function usePercentileOptions(points: PerformancePercentilePoint[]): ChartOptions {
  const { t } = useTranslate('admin');

  return useChart({
    xaxis: { categories: points.map((point) => new Date(point.bucket_started_at).toLocaleString()) },
    legend: { show: true },
    markers: { size: points.length === 1 ? 4 : 0 },
    tooltip: { y: { formatter: (value: number) => `${value}ms` } },
    noData: { text: t('performanceMonitoring.empty.noPercentileData') },
  } satisfies ChartOptions);
}

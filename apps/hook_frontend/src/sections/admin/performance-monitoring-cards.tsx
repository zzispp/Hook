'use client';

import type { ChartOptions } from 'src/components/chart';
import type { PerformanceSnapshotPoint } from 'src/types/performance-monitoring';

import Card from '@mui/material/Card';
import Typography from '@mui/material/Typography';
import CardHeader from '@mui/material/CardHeader';

import { useTranslate } from 'src/locales/use-locales';

import { Chart, useChart } from 'src/components/chart';

import { round, safeChartValue } from './performance-monitoring-format';

export function RequestTrendChart({ series }: { series: PerformanceSnapshotPoint[] }) {
  const { t } = useTranslate('admin');
  const options = useSnapshotLineOptions(series);

  return (
    <ChartCard title={t('performanceMonitoring.charts.requests')} empty={!series.length}>
      <Chart
        type="area"
        series={[
          { name: 'QPS', data: series.map((point) => round(point.metrics.core.qps)) },
          {
            name: t('performanceMonitoring.series.errorRate'),
            data: series.map((point) => round(point.metrics.core.error_rate * 100)),
          },
        ]}
        options={options}
        sx={{ height: 320, p: 2 }}
      />
    </ChartCard>
  );
}

export function SnapshotLatencyChart({ series }: { series: PerformanceSnapshotPoint[] }) {
  const { t } = useTranslate('admin');
  const options = useSnapshotLineOptions(series);

  return (
    <ChartCard title={t('performanceMonitoring.charts.latency')} empty={!series.length}>
      <Chart
        type="line"
        series={[
          {
            name: t('requestRecords.responseHeaders'),
            data: series.map((point) => safeChartValue(point.metrics.core.p90_response_headers_ms)),
          },
          {
            name: t('requestRecords.firstByte'),
            data: series.map((point) => safeChartValue(point.metrics.core.p90_first_byte_ms)),
          },
          {
            name: t('requestRecords.firstToken'),
            data: series.map((point) => safeChartValue(point.metrics.core.p90_first_token_ms)),
          },
          {
            name: t('requestRecords.totalLatency'),
            data: series.map((point) => safeChartValue(point.metrics.core.p90_latency_ms)),
          },
        ]}
        options={options}
        sx={{ height: 320, p: 2 }}
      />
    </ChartCard>
  );
}

function ChartCard({
  title,
  empty,
  children,
}: {
  title: string;
  empty: boolean;
  children: React.ReactNode;
}) {
  const { t } = useTranslate('admin');

  return (
    <Card>
      <CardHeader title={title} />
      {empty ? (
        <Typography variant="body2" color="text.secondary" sx={{ px: 3, py: 4 }}>
          {t('performanceMonitoring.empty.noSeriesData')}
        </Typography>
      ) : (
        children
      )}
    </Card>
  );
}

function useSnapshotLineOptions(series: PerformanceSnapshotPoint[]): ChartOptions {
  const { t } = useTranslate('admin');

  return useChart({
    xaxis: { categories: series.map((point) => formatBucket(point.bucket_started_at)) },
    legend: { show: true },
    markers: { size: series.length === 1 ? 4 : 0 },
    tooltip: { x: { show: true } },
    noData: { text: t('performanceMonitoring.empty.noSeriesData') },
  } satisfies ChartOptions);
}

function formatBucket(value: string) {
  return new Date(value).toLocaleString();
}

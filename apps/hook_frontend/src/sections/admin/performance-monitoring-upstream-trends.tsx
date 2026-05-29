'use client';

import type { ChartOptions } from 'src/components/chart';
import type {
  UpstreamPerformanceProvider,
  UpstreamPerformanceTimelinePoint,
} from 'src/types/performance-monitoring';

import Card from '@mui/material/Card';
import Grid from '@mui/material/Grid';
import Typography from '@mui/material/Typography';
import CardHeader from '@mui/material/CardHeader';

import { useTranslate } from 'src/locales/use-locales';

import { Chart, useChart } from 'src/components/chart';

const PROVIDER_COLORS = [
  '#2563eb',
  '#059669',
  '#d97706',
  '#dc2626',
  '#7c3aed',
  '#0891b2',
  '#ea580c',
  '#0f766e',
];

type TrendMetric = 'avg_output_tps' | 'avg_ttfb_ms';

export function UpstreamTrendCharts({
  providers,
  timeline,
}: {
  providers: UpstreamPerformanceProvider[];
  timeline: UpstreamPerformanceTimelinePoint[];
}) {
  const { t } = useTranslate('admin');

  return (
    <Grid container spacing={3}>
      <Grid size={{ xs: 12, md: 6 }}>
        <TrendCard
          title={t('performanceMonitoring.charts.outputTpsTrend')}
          suffix=" tps"
          metric="avg_output_tps"
          providers={providers}
          timeline={timeline}
        />
      </Grid>
      <Grid size={{ xs: 12, md: 6 }}>
        <TrendCard
          title={t('performanceMonitoring.charts.avgTtfbTrend')}
          suffix="ms"
          metric="avg_ttfb_ms"
          providers={providers}
          timeline={timeline}
        />
      </Grid>
    </Grid>
  );
}

function TrendCard({
  title,
  suffix,
  metric,
  providers,
  timeline,
}: {
  title: string;
  suffix: string;
  metric: TrendMetric;
  providers: UpstreamPerformanceProvider[];
  timeline: UpstreamPerformanceTimelinePoint[];
}) {
  const { t } = useTranslate('admin');
  const labels = timelineLabels(timeline);
  const options = useTrendOptions(labels, suffix);

  return (
    <Card>
      <CardHeader title={title} />
      {!timeline.length ? (
        <Typography variant="body2" color="text.secondary" sx={{ px: 3, py: 4 }}>
          {t('performanceMonitoring.empty.noUpstreamData')}
        </Typography>
      ) : (
        <Chart
          type="line"
          series={providerSeries(labels, providers, timeline, metric)}
          options={options}
          sx={{ height: 320, p: 2 }}
        />
      )}
    </Card>
  );
}

function providerSeries(
  labels: string[],
  providers: UpstreamPerformanceProvider[],
  timeline: UpstreamPerformanceTimelinePoint[],
  metric: TrendMetric
) {
  return providers.map((provider) => {
    const byBucket = new Map(
      timeline
        .filter((point) => point.provider_id === provider.provider_id)
        .map((point) => [bucketLabel(point.bucket_started_at), point[metric]] as const)
    );
    return {
      name: provider.provider_name,
      data: labels.map((label) => byBucket.get(label) ?? null),
    };
  });
}

function useTrendOptions(labels: string[], suffix: string): ChartOptions {
  const { t } = useTranslate('admin');

  return useChart({
    colors: PROVIDER_COLORS,
    xaxis: { categories: labels },
    legend: { show: true },
    tooltip: { y: { formatter: (value: number) => `${value}${suffix}` } },
    noData: { text: t('performanceMonitoring.empty.noUpstreamData') },
  } satisfies ChartOptions);
}

function timelineLabels(timeline: UpstreamPerformanceTimelinePoint[]) {
  return Array.from(new Set(timeline.map((point) => bucketLabel(point.bucket_started_at))));
}

function bucketLabel(value: string) {
  return new Date(value).toLocaleString();
}

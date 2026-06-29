'use client';

import type { PerformanceMonitoringRange } from 'src/types/performance-monitoring';

import { useMemo, useState } from 'react';

import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import ButtonGroup from '@mui/material/ButtonGroup';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';
import {
  usePerformanceMonitoringOverview,
  usePerformanceMonitoringRealtime,
  usePerformanceMonitoringAnalytics,
} from 'src/actions/performance-monitoring';

import { RefreshButton, AdminBreadcrumbs } from './shared';
import { PercentileChart } from './performance-monitoring-percentiles';
import { RecentErrorsTable } from './performance-monitoring-recent-errors';
import { UpstreamTrendCharts } from './performance-monitoring-upstream-trends';
import { UpstreamSummaryCards } from './performance-monitoring-upstream-summary';
import { UpstreamPerformanceTable } from './performance-monitoring-upstream-table';
import { SummaryGrid, PerformanceDetailPanels } from './performance-monitoring-summary';
import { RequestTrendChart, SnapshotLatencyChart } from './performance-monitoring-cards';
import { ErrorTrendChart, ErrorDistributionChart } from './performance-monitoring-errors';
import {
  AnalyticsFilters,
  toAnalyticsQueryFilters,
  DEFAULT_ANALYTICS_FILTERS,
} from './performance-monitoring-analytics-filters';

const RANGE_OPTIONS: PerformanceMonitoringRange[] = ['realtime', 'today', '7d', '30d', 'all'];

export function PerformanceMonitoringView() {
  const [range, setRange] = useState<PerformanceMonitoringRange>('realtime');
  const [filters, setFilters] = useState(DEFAULT_ANALYTICS_FILTERS);
  const isRealtime = range === 'realtime';
  const overview = usePerformanceMonitoringOverview(range);
  const realtime = usePerformanceMonitoringRealtime(isRealtime);
  const analyticsQuery = useMemo(() => ({ range, ...toAnalyticsQueryFilters(filters) }), [filters, range]);
  const analytics = usePerformanceMonitoringAnalytics(analyticsQuery);
  const snapshot =
    (isRealtime ? realtime.data?.snapshot : overview.data?.series.at(-1)) ?? undefined;
  const series =
    overview.data?.series ?? (isRealtime && realtime.data?.snapshot ? [realtime.data.snapshot] : []);
  const analyticsData = analytics.data;

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.performanceMonitoring}
        action={
          <HeaderActions
            range={range}
            loading={overview.isLoading || realtime.isLoading || analytics.isLoading}
            onRangeChange={setRange}
            onRefresh={() => {
              void overview.refresh();
              void analytics.refresh();
              if (isRealtime) void realtime.refresh();
            }}
          />
        }
      />

      <Stack spacing={3}>
        <StatusAlerts
          overview={overview.data}
          error={overview.error ?? realtime.error ?? analytics.error}
        />
        <SummaryGrid snapshot={snapshot} />
        <PerformanceDetailPanels snapshot={snapshot} hostStatus={realtime.data?.host.metrics.status} />
        <SnapshotCharts series={series} />
        <AnalyticsFilters filters={filters} onChange={setFilters} />
        <UpstreamSummaryCards summary={analyticsData?.upstream_performance.summary} />
        <UpstreamTrendCharts
          providers={analyticsData?.upstream_performance.providers ?? []}
          timeline={analyticsData?.upstream_performance.timeline ?? []}
        />
        <AnalyticsCharts
          percentiles={analyticsData?.percentiles ?? []}
          errorDistribution={analyticsData?.error_distribution ?? []}
          errorTrend={analyticsData?.error_trend ?? []}
        />
        <UpstreamPerformanceTable providers={analyticsData?.upstream_performance.providers ?? []} />
        <RecentErrorsTable errors={analyticsData?.recent_errors ?? []} />
      </Stack>
    </DashboardContent>
  );
}

function SnapshotCharts({ series }: { series: Parameters<typeof RequestTrendChart>[0]['series'] }) {
  return (
    <Grid container spacing={3}>
      <Grid size={{ xs: 12, lg: 6 }}>
        <RequestTrendChart series={series} />
      </Grid>
      <Grid size={{ xs: 12, lg: 6 }}>
        <SnapshotLatencyChart series={series} />
      </Grid>
    </Grid>
  );
}

function AnalyticsCharts({
  percentiles,
  errorTrend,
  errorDistribution,
}: {
  percentiles: Parameters<typeof PercentileChart>[0]['points'];
  errorTrend: Parameters<typeof ErrorTrendChart>[0]['points'];
  errorDistribution: Parameters<typeof ErrorDistributionChart>[0]['items'];
}) {
  const { t } = useTranslate('admin');

  return (
    <Grid container spacing={3}>
      <Grid size={{ xs: 12, lg: 6 }}>
        <PercentileChart
          mode="latency"
          points={percentiles}
          title={t('performanceMonitoring.charts.responsePercentiles')}
        />
      </Grid>
      <Grid size={{ xs: 12, lg: 6 }}>
        <PercentileChart
          mode="response_headers"
          points={percentiles}
          title={t('performanceMonitoring.columns.responseHeaders')}
        />
      </Grid>
      <Grid size={{ xs: 12, lg: 6 }}>
        <PercentileChart
          mode="ttfb"
          points={percentiles}
          title={t('performanceMonitoring.charts.ttfbPercentiles')}
        />
      </Grid>
      <Grid size={{ xs: 12, lg: 6 }}>
        <PercentileChart
          mode="first_output"
          points={percentiles}
          title={t('performanceMonitoring.charts.firstOutputPercentiles')}
        />
      </Grid>
      <Grid size={{ xs: 12, lg: 6 }}>
        <ErrorDistributionChart items={errorDistribution} />
      </Grid>
      <Grid size={{ xs: 12, lg: 6 }}>
        <ErrorTrendChart points={errorTrend} />
      </Grid>
    </Grid>
  );
}

function HeaderActions({
  range,
  loading,
  onRefresh,
  onRangeChange,
}: {
  range: PerformanceMonitoringRange;
  loading: boolean;
  onRefresh: VoidFunction;
  onRangeChange: (value: PerformanceMonitoringRange) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction="row" spacing={1} alignItems="center">
      <ButtonGroup variant="outlined" size="small">
        {RANGE_OPTIONS.map((item) => (
          <Button
            key={item}
            variant={range === item ? 'contained' : 'outlined'}
            onClick={() => onRangeChange(item)}
          >
            {t(`performanceMonitoring.ranges.${item}`)}
          </Button>
        ))}
      </ButtonGroup>
      <RefreshButton loading={loading} onClick={onRefresh} />
    </Stack>
  );
}

function StatusAlerts({
  overview,
  error,
}: {
  overview?: ReturnType<typeof usePerformanceMonitoringOverview>['data'];
  error?: Error;
}) {
  const { t } = useTranslate('admin');

  if (error) return <Alert severity="error">{error.message}</Alert>;
  if (overview?.status === 'empty_snapshot') {
    return <Alert severity="info">{t('performanceMonitoring.emptySnapshot')}</Alert>;
  }
  if (overview?.range === 'all') {
    return (
      <Alert severity="info">
        {t('performanceMonitoring.allRangeNotice', {
          granularity: t(`performanceMonitoring.granularity.${overview.bucket_granularity}`),
        })}
      </Alert>
    );
  }
  return null;
}

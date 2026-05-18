'use client';

import type { PerformanceMonitoringRange } from 'src/types/performance-monitoring';

import { useState } from 'react';

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
} from 'src/actions/performance-monitoring';

import { RefreshButton, AdminBreadcrumbs } from './shared';
import {
  SummaryGrid,
  DetailCards,
  SeriesChart,
  LatencyChart,
  DistributionCard,
} from './performance-monitoring-cards';

const RANGE_OPTIONS: PerformanceMonitoringRange[] = ['realtime', 'today', '7d', '30d', 'all'];

export function PerformanceMonitoringView() {
  const { t } = useTranslate('admin');
  const [range, setRange] = useState<PerformanceMonitoringRange>('realtime');
  const isRealtime = range === 'realtime';
  const overview = usePerformanceMonitoringOverview(range);
  const realtime = usePerformanceMonitoringRealtime(isRealtime);
  const activeSnapshot =
    (range === 'realtime' ? realtime.data?.snapshot : overview.data?.series.at(-1)) ?? undefined;
  const chartSeries =
    overview.data?.series ?? (isRealtime && realtime.data?.snapshot ? [realtime.data.snapshot] : []);

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.performanceMonitoring}
        action={
          <HeaderActions
            range={range}
            loading={overview.isLoading || realtime.isLoading}
            onRangeChange={setRange}
            onRefresh={() => {
              void overview.refresh();
              if (isRealtime) {
                void realtime.refresh();
              }
            }}
          />
        }
      />

      <Stack spacing={3}>
        <StatusAlerts
          overview={overview.data}
          error={overview.error ?? realtime.error}
        />
        <SummaryGrid snapshot={activeSnapshot} />
        <Grid container spacing={3}>
          <Grid size={{ xs: 12, md: 8 }}>
            <SeriesChart title={t('performanceMonitoring.charts.requests')} series={chartSeries} />
          </Grid>
          <Grid size={{ xs: 12, md: 4 }}>
            <DistributionCard
              title={t('performanceMonitoring.charts.models')}
              items={activeSnapshot?.metrics.llm.model_distribution ?? []}
            />
          </Grid>
          <Grid size={{ xs: 12, md: 8 }}>
            <LatencyChart title={t('performanceMonitoring.charts.latency')} series={chartSeries} />
          </Grid>
          <Grid size={{ xs: 12, md: 4 }}>
            <DistributionCard
              title={t('performanceMonitoring.charts.providers')}
              items={activeSnapshot?.metrics.llm.provider_distribution ?? []}
            />
          </Grid>
        </Grid>
        <DetailCards snapshot={activeSnapshot} hostStatus={realtime.data?.host.metrics.status} />
      </Stack>
    </DashboardContent>
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

  if (error) {
    return <Alert severity="error">{error.message}</Alert>;
  }
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

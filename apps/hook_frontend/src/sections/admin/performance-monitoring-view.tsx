'use client';

import type { PerformanceMonitoringRange } from 'src/types/performance-monitoring';

import { useMemo, useState } from 'react';

import Tab from '@mui/material/Tab';
import Grid from '@mui/material/Grid';
import Card from '@mui/material/Card';
import Tabs from '@mui/material/Tabs';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';

import { currencyDisplayFromResponse } from 'src/utils/currency-format';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useCurrencyDisplay } from 'src/actions/system-settings';
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
  const currency = useCurrencyDisplay();
  const overview = usePerformanceMonitoringOverview(range);
  const realtime = usePerformanceMonitoringRealtime(isRealtime);
  const currencyDisplay = useMemo(
    () => currencyDisplayFromResponse(currency.data, t('requestRecords.exchangeRateUnavailable')),
    [currency.data, t]
  );
  const activeSnapshot =
    (range === 'realtime' ? realtime.data?.snapshot : overview.data?.series.at(-1)) ?? undefined;
  const chartSeries =
    overview.data?.series ?? (isRealtime && realtime.data?.snapshot ? [realtime.data.snapshot] : []);

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.performanceMonitoring}
        action={
          <RefreshButton
            loading={overview.isLoading || realtime.isLoading}
            onClick={() => {
              void overview.refresh();
              if (isRealtime) {
                void realtime.refresh();
              }
            }}
          />
        }
      />

      <Stack spacing={3}>
        <RangeTabs value={range} onChange={setRange} />
        <StatusAlerts
          overview={overview.data}
          error={overview.error ?? realtime.error ?? currency.error}
        />
        <SummaryGrid snapshot={activeSnapshot} currencyDisplay={currencyDisplay} />
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

function RangeTabs({
  value,
  onChange,
}: {
  value: PerformanceMonitoringRange;
  onChange: (value: PerformanceMonitoringRange) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Card sx={{ px: 2 }}>
      <Tabs value={value} onChange={(_, next) => onChange(next)}>
        {RANGE_OPTIONS.map((item) => (
          <Tab key={item} value={item} label={t(`performanceMonitoring.ranges.${item}`)} />
        ))}
      </Tabs>
    </Card>
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

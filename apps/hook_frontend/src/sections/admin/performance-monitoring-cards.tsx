'use client';

import type { Theme } from '@mui/material/styles';
import type { ChartOptions } from 'src/components/chart';
import type { MetricDimension, PerformanceSnapshotPoint } from 'src/types/performance-monitoring';

import Box from '@mui/material/Box';
import Grid from '@mui/material/Grid';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Divider from '@mui/material/Divider';
import { useTheme } from '@mui/material/styles';
import Typography from '@mui/material/Typography';
import CardHeader from '@mui/material/CardHeader';

import { formatMoneyCompact } from 'src/utils/currency-format';
import { fData, fNumber, fPercent } from 'src/utils/format-number';

import { useTranslate } from 'src/locales/use-locales';

import { Chart, useChart, ChartLegends } from 'src/components/chart';

export function SummaryGrid({
  snapshot,
}: {
  snapshot?: PerformanceSnapshotPoint;
}) {
  const { t } = useTranslate('admin');
  const core = snapshot?.metrics.core;
  const llm = snapshot?.metrics.llm;

  return (
    <Grid container spacing={3}>
      <MetricCard label={t('performanceMonitoring.metrics.qps')} value={formatRate(core?.qps)} />
      <MetricCard
        label={t('performanceMonitoring.metrics.successRate')}
        value={formatRatio(core?.success_rate)}
      />
      <MetricCard label={t('performanceMonitoring.metrics.p95')} value={formatMs(core?.p95_latency_ms)} />
      <MetricCard
        label={t('performanceMonitoring.metrics.tokensPerSecond')}
        value={formatRate(llm?.tokens_per_second)}
      />
      <MetricCard label={t('performanceMonitoring.metrics.totalTokens')} value={fNumber(llm?.total_tokens ?? 0)} />
      <MetricCard
        label={t('performanceMonitoring.metrics.cost')}
        value={formatCost(llm?.cost)}
      />
    </Grid>
  );
}

export function SeriesChart({ title, series }: { title: string; series: PerformanceSnapshotPoint[] }) {
  const chartOptions = useLineOptions(series);

  return (
    <Card>
      <CardHeader title={title} />
      <Chart
        type="area"
        series={[
          { name: 'QPS', data: series.map((point) => round(point.metrics.core.qps)) },
          { name: 'Errors', data: series.map((point) => point.metrics.core.error_rate * 100) },
        ]}
        options={chartOptions}
        sx={{ height: 360, p: 2 }}
      />
    </Card>
  );
}

export function LatencyChart({ title, series }: { title: string; series: PerformanceSnapshotPoint[] }) {
  const chartOptions = useLineOptions(series);

  return (
    <Card>
      <CardHeader title={title} />
      <Chart
        type="line"
        series={[
          { name: 'p50', data: series.map((point) => point.metrics.core.p50_latency_ms ?? 0) },
          { name: 'p95', data: series.map((point) => point.metrics.core.p95_latency_ms ?? 0) },
          { name: 'p99', data: series.map((point) => point.metrics.core.p99_latency_ms ?? 0) },
          { name: 'TTFT p95', data: series.map((point) => point.metrics.core.p95_ttft_ms ?? 0) },
        ]}
        options={chartOptions}
        sx={{ height: 360, p: 2 }}
      />
    </Card>
  );
}

export function DistributionCard({ title, items }: { title: string; items: MetricDimension[] }) {
  const { t } = useTranslate('admin');
  const theme = useTheme();
  const colors = dimensionColors(theme).slice(0, items.length);
  const options = useChart({
    chart: { sparkline: { enabled: true } },
    colors,
    labels: items.map((item) => item.name),
    stroke: { width: 0 },
    dataLabels: { enabled: true, dropShadow: { enabled: false } },
    tooltip: { y: { formatter: (value: number) => fNumber(value) } },
    plotOptions: { pie: { donut: { labels: { show: false } } } },
  } satisfies ChartOptions);

  return (
    <Card sx={{ height: '100%' }}>
      <CardHeader title={title} />
      {!items.length ? (
        <Typography variant="body2" color="text.secondary" sx={{ p: 3 }}>
          {t('performanceMonitoring.noDimensionData')}
        </Typography>
      ) : null}
      {items.length ? (
        <>
          <Chart
            type="pie"
            series={items.map((item) => item.count)}
            options={options}
            sx={{ my: 6, mx: 'auto', width: { xs: 240, xl: 260 }, height: { xs: 240, xl: 260 } }}
          />
          <Divider sx={{ borderStyle: 'dashed' }} />
          <ChartLegends
            labels={items.map((item) => item.name)}
            colors={colors}
            values={items.map((item) => fNumber(item.count))}
            sx={{ p: 3, justifyContent: 'center' }}
          />
        </>
      ) : null}
    </Card>
  );
}

function dimensionColors(theme: Theme) {
  return [
    theme.palette.primary.main,
    theme.palette.warning.light,
    theme.palette.info.dark,
    theme.palette.error.main,
    theme.palette.success.main,
    theme.palette.secondary.main,
    theme.palette.primary.dark,
    theme.palette.warning.dark,
    theme.palette.info.main,
    theme.palette.error.dark,
  ];
}

export function DetailCards({
  snapshot,
  hostStatus,
}: {
  snapshot?: PerformanceSnapshotPoint;
  hostStatus?: string;
}) {
  const { t } = useTranslate('admin');
  const core = snapshot?.metrics.core;
  const llm = snapshot?.metrics.llm;
  const network = snapshot?.metrics.network;
  const host = snapshot?.metrics.host;

  return (
    <Grid container spacing={3}>
      <DetailCard
        title={t('performanceMonitoring.groups.core')}
        rows={[
          [t('performanceMonitoring.rows.qpsConcurrency'), `${formatRate(core?.qps)} / ${fNumber(core?.concurrent_requests ?? 0)}`],
          [t('performanceMonitoring.rows.rateLimitedServerErrors'), `${fNumber(core?.rate_limited_count ?? 0)} / ${fNumber(core?.server_error_count ?? 0)}`],
          [t('performanceMonitoring.rows.timeoutRetryCircuitBreaker'), `${formatRatio(core?.timeout_rate)} / ${fNumber(core?.retry_count ?? 0)} / ${fNumber(core?.circuit_breaker_count ?? 0)}`],
          [t('performanceMonitoring.rows.streamRequests'), fNumber(core?.stream_request_count ?? 0)],
        ]}
      />
      <DetailCard
        title={t('performanceMonitoring.groups.llm')}
        rows={[
          [t('performanceMonitoring.rows.inputOutputTotalTokens'), `${fNumber(llm?.prompt_tokens ?? 0)} / ${fNumber(llm?.completion_tokens ?? 0)} / ${fNumber(llm?.total_tokens ?? 0)}`],
          [t('performanceMonitoring.rows.tokensPerRequestPerSecond'), `${formatRate(llm?.tokens_per_request)} / ${formatRate(llm?.tokens_per_second)}`],
          [t('performanceMonitoring.rows.failoverCacheHit'), `${fNumber(llm?.failover_count ?? 0)} / ${formatRatio(llm?.cache_hit_rate)}`],
          [t('performanceMonitoring.rows.quotaLimited'), fNumber(llm?.quota_limited_count ?? 0)],
        ]}
      />
      <DetailCard
        title={t('performanceMonitoring.groups.network')}
        rows={[
          [t('performanceMonitoring.rows.totalInboundOutbound'), `${fData(network?.inbound_bytes ?? 0)} / ${fData(network?.outbound_bytes ?? 0)}`],
          [t('performanceMonitoring.rows.realtimeBandwidth'), `${fData(network?.inbound_bandwidth_bytes_per_second ?? 0)}/s / ${fData(network?.outbound_bandwidth_bytes_per_second ?? 0)}/s`],
          [t('performanceMonitoring.rows.connectionStatus'), supportText(t, network?.status)],
          [t('performanceMonitoring.rows.tcpEstablishedCloseWait'), `${valueOrDash(network?.tcp_established)} / ${valueOrDash(network?.tcp_close_wait)}`],
        ]}
      />
      <DetailCard
        title={t('performanceMonitoring.groups.host')}
        rows={[
          [t('performanceMonitoring.rows.cpuLoad'), `${formatPercentNumber(host?.cpu_usage_percent)} / ${valueOrDash(host?.load_average_1m)}`],
          [t('performanceMonitoring.rows.memory'), `${formatBytes(host?.memory_rss_bytes)} / ${formatBytes(host?.memory_usage_bytes)}`],
          [t('performanceMonitoring.rows.diskSpace'), `${formatBytes(host?.disk_available_bytes)} / ${formatBytes(host?.disk_total_bytes)}`],
          [t('performanceMonitoring.rows.collectionStatus'), supportText(t, hostStatus ?? host?.status)],
        ]}
      />
    </Grid>
  );
}

function MetricCard({ label, value }: { label: string; value: string }) {
  return (
    <Grid size={{ xs: 12, sm: 6, md: 2 }}>
      <Card sx={{ p: 2.5 }}>
        <Typography variant="overline" color="text.secondary">
          {label}
        </Typography>
        <Typography variant="h4" sx={{ mt: 1 }}>
          {value}
        </Typography>
      </Card>
    </Grid>
  );
}

function DetailCard({ title, rows }: { title: string; rows: [string, string][] }) {
  return (
    <Grid size={{ xs: 12, md: 6 }}>
      <Card sx={{ p: 3 }}>
        <Typography variant="h6">{title}</Typography>
        <Divider sx={{ my: 2 }} />
        <Stack spacing={1.5}>
          {rows.map(([label, value]) => (
            <Stack key={label} direction="row" justifyContent="space-between" spacing={2}>
              <Typography variant="body2" color="text.secondary">
                {label}
              </Typography>
              <Box component="span" sx={{ typography: 'body2', fontWeight: 600, textAlign: 'right' }}>
                {value}
              </Box>
            </Stack>
          ))}
        </Stack>
      </Card>
    </Grid>
  );
}

function useLineOptions(series: PerformanceSnapshotPoint[]): ChartOptions {
  return useChart({
    xaxis: { categories: series.map((point) => new Date(point.bucket_started_at).toLocaleString()) },
    legend: { show: true },
    markers: { size: series.length === 1 ? 4 : 0 },
    tooltip: { x: { show: true } },
  });
}

function formatMs(value?: number | null) {
  return value === null || value === undefined ? '-' : `${fNumber(value)}ms`;
}

function formatRate(value?: number | null) {
  return fNumber(value ?? 0, { maximumFractionDigits: 2 });
}

function formatRatio(value?: number | null) {
  return fPercent((value ?? 0) * 100);
}

function formatCost(value: number | null | undefined) {
  return formatMoneyCompact(value);
}

function formatBytes(value?: number | null) {
  return value === null || value === undefined ? '-' : fData(value);
}

function formatPercentNumber(value?: number | null) {
  return value === null || value === undefined ? '-' : `${fNumber(value)}%`;
}

function valueOrDash(value?: number | null) {
  return value === null || value === undefined ? '-' : fNumber(value);
}

function supportText(t: ReturnType<typeof useTranslate>['t'], status?: string) {
  return status === 'ready'
    ? t('performanceMonitoring.status.ready')
    : t('performanceMonitoring.status.unsupported');
}

function round(value: number) {
  return Number(value.toFixed(4));
}

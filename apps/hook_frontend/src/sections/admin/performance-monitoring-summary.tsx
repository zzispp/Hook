'use client';

import type { PerformanceSnapshotPoint } from 'src/types/performance-monitoring';

import Box from '@mui/material/Box';
import Grid from '@mui/material/Grid';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';

import { fNumber } from 'src/utils/format-number';

import { useTranslate } from 'src/locales/use-locales';

import {
  formatMs,
  formatRate,
  formatRatio,
  formatBytes,
  valueOrDash,
  formatTokens,
  formatTokenRate,
  formatPercentNumber,
} from './performance-monitoring-format';

export function SummaryGrid({ snapshot }: { snapshot?: PerformanceSnapshotPoint }) {
  const { t } = useTranslate('admin');
  const core = snapshot?.metrics.core;
  const llm = snapshot?.metrics.llm;

  return (
    <Grid container spacing={2}>
      <MetricCard label={t('performanceMonitoring.metrics.qps')} value={formatRate(core?.qps)} />
      <MetricCard
        label={t('performanceMonitoring.metrics.p95Latency')}
        value={formatMs(core?.p95_latency_ms)}
      />
      <MetricCard
        label={t('performanceMonitoring.metrics.ttfb')}
        value={formatMs(core?.p90_ttfb_ms)}
      />
      <MetricCard
        label={t('performanceMonitoring.metrics.tokensPerSecond')}
        value={formatTokenRate(llm?.tokens_per_second)}
      />
    </Grid>
  );
}

export function PerformanceDetailPanels({
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
      <DetailPanel
        title={t('performanceMonitoring.groups.core')}
        rows={[
          [t('performanceMonitoring.rows.qpsConcurrency'), `${formatRate(core?.qps)} / ${fNumber(core?.concurrent_requests ?? 0)}`],
          [t('performanceMonitoring.rows.errorTimeout'), `${formatRatio(core?.error_rate)} / ${formatRatio(core?.timeout_rate)}`],
          [t('performanceMonitoring.rows.rateLimitedServerErrors'), `${fNumber(core?.rate_limited_count ?? 0)} / ${fNumber(core?.server_error_count ?? 0)}`],
          [t('performanceMonitoring.rows.retryCircuitBreaker'), `${fNumber(core?.retry_count ?? 0)} / ${fNumber(core?.circuit_breaker_count ?? 0)}`],
          [t('performanceMonitoring.rows.streamRequests'), fNumber(core?.stream_request_count ?? 0)],
        ]}
      />
      <DetailPanel
        title={t('performanceMonitoring.groups.llm')}
        rows={[
          [t('performanceMonitoring.rows.inputOutputTokens'), `${formatTokens(llm?.prompt_tokens)} / ${formatTokens(llm?.completion_tokens)}`],
          [t('performanceMonitoring.rows.tokensPerRequestPerSecond'), `${formatTokens(llm?.tokens_per_request)} / ${formatTokenRate(llm?.tokens_per_second)}`],
          [t('performanceMonitoring.rows.failoverCacheHit'), `${fNumber(llm?.failover_count ?? 0)} / ${formatRatio(llm?.cache_hit_rate)}`],
          [t('performanceMonitoring.rows.quotaLimited'), fNumber(llm?.quota_limited_count ?? 0)],
        ]}
      />
      <DetailPanel
        title={t('performanceMonitoring.groups.host')}
        rows={[
          [t('performanceMonitoring.rows.cpuLoad'), `${formatPercentNumber(host?.cpu_usage_percent)} / ${valueOrDash(host?.load_average_1m)}`],
          [t('performanceMonitoring.rows.memory'), `${formatBytes(host?.memory_rss_bytes)} / ${formatBytes(host?.memory_usage_bytes)}`],
          [t('performanceMonitoring.rows.diskSpace'), `${formatBytes(host?.disk_available_bytes)} / ${formatBytes(host?.disk_total_bytes)}`],
          [t('performanceMonitoring.rows.collectionStatus'), supportText(t, hostStatus ?? host?.status)],
        ]}
      />
      <DetailPanel
        title={t('performanceMonitoring.groups.network')}
        rows={[
          [t('performanceMonitoring.rows.totalInboundOutbound'), `${formatBytes(network?.inbound_bytes)} / ${formatBytes(network?.outbound_bytes)}`],
          [t('performanceMonitoring.rows.realtimeBandwidth'), `${formatBytes(network?.inbound_bandwidth_bytes_per_second)}/s / ${formatBytes(network?.outbound_bandwidth_bytes_per_second)}/s`],
          [t('performanceMonitoring.rows.connectionStatus'), supportText(t, network?.status)],
          [t('performanceMonitoring.rows.tcpEstablishedCloseWait'), `${valueOrDash(network?.tcp_established)} / ${valueOrDash(network?.tcp_close_wait)}`],
        ]}
      />
    </Grid>
  );
}

function MetricCard({ label, value }: { label: string; value: string }) {
  return (
    <Grid size={{ xs: 12, sm: 6, md: 3 }}>
      <Card sx={{ p: 2.5, minHeight: 120 }}>
        <Typography variant="overline" color="text.secondary">
          {label}
        </Typography>
        <Typography variant="h3" sx={{ mt: 1 }}>
          {value}
        </Typography>
      </Card>
    </Grid>
  );
}

function DetailPanel({ title, rows }: { title: string; rows: [string, string][] }) {
  return (
    <Grid size={{ xs: 12, md: 6, xl: 3 }}>
      <Card sx={{ p: 3, height: '100%', minHeight: 260 }}>
        <Typography variant="h5">{title}</Typography>
        <Divider sx={{ my: 2 }} />
        <Stack spacing={1.75}>
          {rows.map(([label, value]) => (
            <Stack key={label} direction="row" justifyContent="space-between" spacing={2}>
              <Typography variant="body2" color="text.secondary">
                {label}
              </Typography>
              <Box component="span" sx={{ typography: 'subtitle2', textAlign: 'right' }}>
                {value}
              </Box>
            </Stack>
          ))}
        </Stack>
      </Card>
    </Grid>
  );
}

function supportText(t: ReturnType<typeof useTranslate>['t'], status?: string) {
  return status === 'ready'
    ? t('performanceMonitoring.status.ready')
    : t('performanceMonitoring.status.unsupported');
}

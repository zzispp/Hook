'use client';

import type { UpstreamPerformanceSummary } from 'src/types/performance-monitoring';

import Grid from '@mui/material/Grid';
import Card from '@mui/material/Card';
import Typography from '@mui/material/Typography';

import { fNumber } from 'src/utils/format-number';

import { useTranslate } from 'src/locales/use-locales';

import { formatMs, formatRatio, formatOptionalRate } from './performance-monitoring-format';

export function UpstreamSummaryCards({ summary }: { summary?: UpstreamPerformanceSummary }) {
  const { t } = useTranslate('admin');
  const cards = [
    {
      label: t('performanceMonitoring.upstream.requestSamples'),
      value: fNumber(summary?.request_count ?? 0),
      detail: t('performanceMonitoring.upstream.errorCount', { value: fNumber(summary?.error_count ?? 0) }),
    },
    {
      label: t('performanceMonitoring.upstream.p99Latency'),
      value: formatMs(summary?.p99_latency_ms),
      detail: t('performanceMonitoring.upstream.p90Value', { value: formatMs(summary?.p90_latency_ms) }),
    },
    {
      label: t('performanceMonitoring.upstream.p99Ttfb'),
      value: formatMs(summary?.p99_ttfb_ms),
      detail: t('performanceMonitoring.upstream.p90Value', { value: formatMs(summary?.p90_ttfb_ms) }),
    },
    {
      label: t('performanceMonitoring.upstream.outputTps'),
      value: outputTps(summary?.avg_output_tps),
      detail: t('performanceMonitoring.upstream.tpsSamples', {
        value: fNumber(summary?.tps_sample_count ?? 0),
      }),
    },
    {
      label: t('performanceMonitoring.upstream.avgTtfb'),
      value: formatMs(summary?.avg_ttfb_ms),
      detail: t('performanceMonitoring.upstream.ttfbSamples', {
        value: fNumber(summary?.ttfb_sample_count ?? 0),
      }),
    },
    {
      label: t('performanceMonitoring.upstream.avgFirstOutput'),
      value: formatMs(summary?.avg_first_output_ms),
      detail: t('performanceMonitoring.upstream.firstOutputSamples', {
        value: fNumber(summary?.first_output_sample_count ?? 0),
      }),
    },
    {
      label: t('performanceMonitoring.upstream.sseToOutputWait'),
      value: formatMs(summary?.avg_sse_to_output_ms),
      detail: t('performanceMonitoring.upstream.sseToOutputSamples', {
        value: fNumber(summary?.sse_to_output_sample_count ?? 0),
      }),
    },
    {
      label: t('performanceMonitoring.upstream.responseHeaders'),
      value: formatMs(summary?.avg_response_headers_ms),
      detail: t('performanceMonitoring.upstream.firstSse', {
        value: formatMs(summary?.avg_first_sse_event_ms),
      }),
    },
    {
      label: t('performanceMonitoring.upstream.avgLatency'),
      value: formatMs(summary?.avg_latency_ms),
      detail: t('performanceMonitoring.upstream.latencySamples', {
        value: fNumber(summary?.latency_sample_count ?? 0),
      }),
    },
    {
      label: t('performanceMonitoring.upstream.errorRate'),
      value: formatRatio(summary?.error_rate),
      detail: t('performanceMonitoring.upstream.successRate', {
        value: formatRatio(summary?.success_rate),
      }),
    },
    {
      label: t('performanceMonitoring.upstream.slowRequests'),
      value: fNumber(summary?.slow_request_count ?? 0),
      detail: t('performanceMonitoring.upstream.currentWindow'),
    },
  ];

  return (
    <Grid container spacing={2}>
      {cards.map((card) => (
        <Grid key={card.label} size={{ xs: 12, sm: 6, md: 3 }}>
          <Card sx={{ p: 2.5, height: 1 }}>
            <Typography variant="caption" color="text.secondary">
              {card.label}
            </Typography>
            <Typography variant="h5" sx={{ mt: 0.75 }}>
              {card.value}
            </Typography>
            <Typography variant="caption" color="text.secondary" sx={{ mt: 0.75, display: 'block' }}>
              {card.detail}
            </Typography>
          </Card>
        </Grid>
      ))}
    </Grid>
  );
}

function outputTps(value?: number | null) {
  return value === null || value === undefined ? '-' : `${formatOptionalRate(value)} tps`;
}

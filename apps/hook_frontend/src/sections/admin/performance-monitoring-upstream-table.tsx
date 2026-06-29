'use client';

import type { UpstreamPerformanceProvider } from 'src/types/performance-monitoring';

import Card from '@mui/material/Card';
import Table from '@mui/material/Table';
import Stack from '@mui/material/Stack';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableHead from '@mui/material/TableHead';
import Typography from '@mui/material/Typography';
import CardHeader from '@mui/material/CardHeader';
import TableContainer from '@mui/material/TableContainer';

import { fNumber } from 'src/utils/format-number';

import { useTranslate } from 'src/locales/use-locales';

import { Scrollbar } from 'src/components/scrollbar';

import { formatMs, formatRatio, formatOptionalRate } from './performance-monitoring-format';

export function UpstreamPerformanceTable({
  providers,
}: {
  providers: UpstreamPerformanceProvider[];
}) {
  const { t } = useTranslate('admin');

  return (
    <Card>
      <CardHeader title={t('performanceMonitoring.tables.upstreamPerformance')} />
      {!providers.length ? (
        <Typography variant="body2" color="text.secondary" sx={{ px: 3, py: 4 }}>
          {t('performanceMonitoring.empty.noUpstreamData')}
        </Typography>
      ) : (
        <TableContainer component={Scrollbar} sx={{ maxHeight: 460 }}>
          <Table size="small" stickyHeader sx={{ minWidth: 1560 }}>
            <TableHead>
              <TableRow>
                <TableCell>{t('performanceMonitoring.columns.upstreamService')}</TableCell>
                <TableCell align="right">{t('performanceMonitoring.columns.requests')}</TableCell>
                <TableCell align="right">{t('performanceMonitoring.columns.successRate')}</TableCell>
                <TableCell align="right">{t('performanceMonitoring.columns.errorRate')}</TableCell>
                <TableCell align="right">{t('performanceMonitoring.columns.outputTps')}</TableCell>
                <TableCell align="right">{t('performanceMonitoring.columns.responseHeaders')}</TableCell>
                <TableCell align="right">{t('requestRecords.firstChar')}</TableCell>
                <TableCell align="right">{t('performanceMonitoring.columns.firstOutput')}</TableCell>
                <TableCell align="right">{t('performanceMonitoring.columns.avgLatency')}</TableCell>
                <TableCell align="right">{t('performanceMonitoring.columns.p90P99Latency')}</TableCell>
                <TableCell align="right">{t('performanceMonitoring.columns.p90P99Ttfb')}</TableCell>
                <TableCell align="right">{t('performanceMonitoring.columns.slowRequests')}</TableCell>
                <TableCell align="right">{t('performanceMonitoring.columns.sampleCoverage')}</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {providers.map((provider) => (
                <ProviderRow key={provider.provider_id} provider={provider} />
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}
    </Card>
  );
}

function ProviderRow({ provider }: { provider: UpstreamPerformanceProvider }) {
  return (
    <TableRow hover>
      <TableCell>
        <Stack spacing={0.25}>
          <Typography variant="subtitle2">{provider.provider_name}</Typography>
          <Typography variant="caption" color="text.secondary">
            {provider.provider_id}
          </Typography>
        </Stack>
      </TableCell>
      <TableCell align="right">{fNumber(provider.request_count)}</TableCell>
      <TableCell align="right">{formatRatio(provider.success_rate)}</TableCell>
      <TableCell align="right">{formatRatio(provider.error_rate)}</TableCell>
      <TableCell align="right">{outputTps(provider.avg_output_tps)}</TableCell>
      <TableCell align="right">{formatMs(provider.avg_response_headers_ms)}</TableCell>
      <TableCell align="right">{formatMs(provider.avg_ttfb_ms)}</TableCell>
      <TableCell align="right">{formatMs(provider.avg_first_output_ms)}</TableCell>
      <TableCell align="right">{formatMs(provider.avg_latency_ms)}</TableCell>
      <TableCell align="right">
        {formatMs(provider.p90_latency_ms)} / {formatMs(provider.p99_latency_ms)}
      </TableCell>
      <TableCell align="right">
        {formatMs(provider.p90_ttfb_ms)} / {formatMs(provider.p99_ttfb_ms)}
      </TableCell>
      <TableCell align="right">{fNumber(provider.slow_request_count)}</TableCell>
      <TableCell align="right">{sampleCoverage(provider)}</TableCell>
    </TableRow>
  );
}

function outputTps(value?: number | null) {
  return value === null || value === undefined ? '-' : `${formatOptionalRate(value)} tps`;
}

function sampleCoverage(provider: UpstreamPerformanceProvider) {
  return [
    `${provider.tps_sample_count} TPS`,
    `${provider.latency_sample_count} latency`,
    `${provider.response_headers_sample_count} headers`,
    `${provider.ttfb_sample_count} first token`,
    `${provider.first_output_sample_count} first output`,
  ].join(' / ');
}

'use client';

import type { RecentPerformanceError } from 'src/types/performance-monitoring';

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

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';
import { Scrollbar } from 'src/components/scrollbar';

import { formatMs, formatDateTime } from './performance-monitoring-format';

export function RecentErrorsTable({ errors }: { errors: RecentPerformanceError[] }) {
  const { t } = useTranslate('admin');

  return (
    <Card>
      <CardHeader title={t('performanceMonitoring.tables.recentErrors')} />
      {!errors.length ? (
        <Typography variant="body2" color="text.secondary" sx={{ px: 3, py: 4 }}>
          {t('performanceMonitoring.empty.noRecentErrors')}
        </Typography>
      ) : (
        <TableContainer component={Scrollbar} sx={{ maxHeight: 420 }}>
          <Table size="small" stickyHeader sx={{ minWidth: 1260 }}>
            <TableHead>
              <TableRow>
                <TableCell>{t('performanceMonitoring.columns.time')}</TableCell>
                <TableCell>{t('performanceMonitoring.columns.requestId')}</TableCell>
                <TableCell>{t('performanceMonitoring.columns.providerModel')}</TableCell>
                <TableCell align="right">{t('performanceMonitoring.columns.statusCode')}</TableCell>
                <TableCell>{t('performanceMonitoring.columns.errorType')}</TableCell>
                <TableCell>{t('performanceMonitoring.columns.errorMessage')}</TableCell>
                <TableCell align="right">{t('performanceMonitoring.columns.responseHeaders')}</TableCell>
                <TableCell align="right">{t('requestRecords.firstByte')}</TableCell>
                <TableCell align="right">{t('requestRecords.firstToken')}</TableCell>
                <TableCell align="right">{t('requestRecords.totalLatency')}</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {errors.map((error) => (
                <ErrorRow key={`${error.created_at}-${error.request_id}`} error={error} />
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}
    </Card>
  );
}

function ErrorRow({ error }: { error: RecentPerformanceError }) {
  return (
    <TableRow hover>
      <TableCell>{formatDateTime(error.created_at)}</TableCell>
      <TableCell>
        <Typography variant="caption" sx={{ fontFamily: 'monospace' }}>
          {error.request_id}
        </Typography>
      </TableCell>
      <TableCell>
        <Stack spacing={0.25}>
          <Typography variant="body2">{error.provider_name ?? error.provider_id ?? '-'}</Typography>
          <Typography variant="caption" color="text.secondary">
            {error.model ?? '-'}
          </Typography>
        </Stack>
      </TableCell>
      <TableCell align="right">{error.status_code ?? '-'}</TableCell>
      <TableCell>
        {error.error_type ? (
          <Label color="error" variant="soft">
            {error.error_type}
          </Label>
        ) : (
          '-'
        )}
      </TableCell>
      <TableCell sx={{ maxWidth: 360 }}>
        <Typography variant="body2" noWrap title={error.error_message ?? undefined}>
          {error.error_message ?? '-'}
        </Typography>
      </TableCell>
      <TableCell align="right">{formatMs(error.response_headers_ms)}</TableCell>
      <TableCell align="right">{formatMs(error.first_byte_ms)}</TableCell>
      <TableCell align="right">{formatMs(error.first_token_ms)}</TableCell>
      <TableCell align="right">{formatMs(error.latency_ms)}</TableCell>
    </TableRow>
  );
}

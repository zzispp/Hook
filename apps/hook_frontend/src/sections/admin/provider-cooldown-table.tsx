'use client';

import type { ProviderCooldown } from 'src/types/provider';
import type { UseTableReturn, TableHeadCellProps } from 'src/components/table';

import Box from '@mui/material/Box';
import Table from '@mui/material/Table';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TextField from '@mui/material/TextField';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';
import InputAdornment from '@mui/material/InputAdornment';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import {
  TableNoData,
  TablePaginationCustom,
  tableStickyActionCellSx,
  withStickyActionHeadCell,
} from 'src/components/table';

import { TableLoadingRows, ManagementTableHead } from './shared';
import { compactId, formatRequestDate } from './request-records-utils';

export type ProviderCooldownFilterState = {
  search: string;
  statusCode: string;
};

type Props = {
  rows: ProviderCooldown[];
  total: number;
  loading: boolean;
  table: UseTableReturn;
  filters: ProviderCooldownFilterState;
  locale: string;
  releasingId: string | null;
  onFiltersChange: (filters: ProviderCooldownFilterState) => void;
  onRelease: (providerId: string) => void;
};

export const DEFAULT_PROVIDER_COOLDOWN_FILTERS: ProviderCooldownFilterState = {
  search: '',
  statusCode: '',
};

export function ProviderCooldownTable(props: Props) {
  const { t } = useTranslate('admin');
  const head = tableHead(t);

  return (
    <>
      <ProviderCooldownToolbar filters={props.filters} onChange={props.onFiltersChange} />
      <Scrollbar>
        <Table sx={{ minWidth: 1600 }}>
          <ManagementTableHead head={head} />
          <TableBody>
            {props.loading ? <TableLoadingRows head={head} rows={props.table.rowsPerPage} /> : null}
            {!props.loading
              ? props.rows.map((row) => (
                  <ProviderCooldownRow
                    key={row.provider_id}
                    row={row}
                    locale={props.locale}
                    releasing={props.releasingId === row.provider_id}
                    onRelease={props.onRelease}
                  />
                ))
              : null}
            <TableNoData title={t('providers.noCooldowns')} notFound={!props.loading && props.rows.length === 0} />
          </TableBody>
        </Table>
      </Scrollbar>
      <TablePaginationCustom
        page={props.table.page}
        count={props.total}
        rowsPerPage={props.table.rowsPerPage}
        onPageChange={props.table.onChangePage}
        onRowsPerPageChange={props.table.onChangeRowsPerPage}
      />
    </>
  );
}

function ProviderCooldownToolbar({
  filters,
  onChange,
}: {
  filters: ProviderCooldownFilterState;
  onChange: (filters: ProviderCooldownFilterState) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Box sx={{ px: 2.5, py: 2, borderBottom: (theme) => `1px solid ${theme.palette.divider}` }}>
      <Box
        sx={{
          gap: 1.5,
          display: 'grid',
          gridTemplateColumns: { xs: '1fr', md: 'minmax(220px, 1fr) 180px auto' },
        }}
      >
        <TextField
          size="small"
          value={filters.search}
          placeholder={t('providers.cooldownSearchPlaceholder')}
          onChange={(event) => onChange({ ...filters, search: event.target.value })}
          slotProps={{
            input: {
              startAdornment: (
                <InputAdornment position="start">
                  <Iconify icon="eva:search-fill" sx={{ color: 'text.disabled' }} />
                </InputAdornment>
              ),
            },
          }}
        />
        <TextField
          size="small"
          type="number"
          label={t('providers.cooldownStatusCode')}
          value={filters.statusCode}
          onChange={(event) => onChange({ ...filters, statusCode: event.target.value })}
        />
        <Tooltip title={t('wallet.actions.reset')}>
          <Button color="inherit" variant="outlined" startIcon={<Iconify icon="solar:close-circle-bold" />} onClick={() => onChange(DEFAULT_PROVIDER_COOLDOWN_FILTERS)}>
            {t('common.clear')}
          </Button>
        </Tooltip>
      </Box>
    </Box>
  );
}

function ProviderCooldownRow({
  row,
  locale,
  releasing,
  onRelease,
}: {
  row: ProviderCooldown;
  locale: string;
  releasing: boolean;
  onRelease: (providerId: string) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover>
      <TableCell sx={{ minWidth: 220 }}>
        <Typography variant="subtitle2">{row.provider_name}</Typography>
        <Typography variant="caption" color="text.secondary">{compactId(row.provider_id)}</Typography>
      </TableCell>
      <TableCell>
        <Label color={statusColor(row.status_code)} variant="soft">{row.status_code}</Label>
      </TableCell>
      <TableCell>{row.observed_count} / {row.threshold_count}</TableCell>
      <TableCell>{formatSeconds(row.window_seconds)} / {formatSeconds(row.cooldown_seconds)}</TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>
        <Stack spacing={0.25}>
          <Typography variant="body2">{formatRequestDate(row.cooldown_until, locale)}</Typography>
          <Typography variant="caption" color="text.secondary">{remainingText(row.cooldown_until, t)}</Typography>
        </Stack>
      </TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{formatRequestDate(row.triggered_at, locale)}</TableCell>
      <TableCell>{compactId(row.request_id)} / {row.candidate_index}:{row.retry_index}</TableCell>
      <TableCell>{row.endpoint_name || row.endpoint_id || '-'}</TableCell>
      <TableCell>{row.key_name || row.key_id || '-'}</TableCell>
      <TableCell sx={{ maxWidth: 320 }}>
        <Stack spacing={0.25}>
          <Typography variant="body2" noWrap title={row.error_message ?? undefined}>{row.error_message || '-'}</Typography>
          <Typography variant="caption" color="text.secondary" noWrap>
            {[row.error_type, row.error_code, row.error_param].filter(Boolean).join(' / ') || '-'}
          </Typography>
        </Stack>
      </TableCell>
      <TableCell align="left" sx={tableStickyActionCellSx}>
        <Button size="small" variant="outlined" loading={releasing} onClick={() => onRelease(row.provider_id)}>
          {t('providers.releaseCooldown')}
        </Button>
      </TableCell>
    </TableRow>
  );
}

function tableHead(t: (key: string, options?: Record<string, unknown>) => string): TableHeadCellProps[] {
  return [
    { id: 'provider', label: t('providers.name'), width: 220 },
    { id: 'status_code', label: t('providers.cooldownStatusCode'), width: 120 },
    { id: 'observed', label: t('providers.cooldownObservedThreshold'), width: 130 },
    { id: 'duration', label: t('providers.cooldownWindowAndDuration'), width: 180 },
    { id: 'until', label: t('providers.cooldownUntil'), width: 220 },
    { id: 'triggered_at', label: t('providers.cooldownTriggeredAt'), width: 180 },
    { id: 'request', label: t('providers.cooldownRequest'), width: 150 },
    { id: 'endpoint', label: t('providers.cooldownEndpoint'), width: 150 },
    { id: 'key', label: t('providers.cooldownKey'), width: 150 },
    { id: 'error', label: t('providers.cooldownError'), width: 320 },
    withStickyActionHeadCell({ id: 'actions', label: t('common.actions'), width: 130, align: 'left' }),
  ];
}

export function toProviderCooldownFilters(filters: ProviderCooldownFilterState) {
  const statusCode = Number(filters.statusCode);
  return {
    search: filters.search.trim() || undefined,
    status_code: Number.isFinite(statusCode) && statusCode > 0 ? statusCode : undefined,
  };
}

function formatSeconds(value: number) {
  return `${value}s`;
}

function remainingText(value: string, t: (key: string, options?: Record<string, unknown>) => string) {
  const seconds = Math.max(0, Math.ceil((new Date(value).getTime() - Date.now()) / 1000));
  return t('providers.cooldownRemainingSeconds', { seconds });
}

function statusColor(statusCode: number) {
  if (statusCode === 429) return 'warning';
  if (statusCode >= 500) return 'error';
  return 'info';
}

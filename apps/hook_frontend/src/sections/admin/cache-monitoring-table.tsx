'use client';

import type { CacheAffinityItem } from 'src/types/cache-monitoring';
import type { UseTableReturn, TableHeadCellProps } from 'src/components/table';

import { useMemo } from 'react';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Table from '@mui/material/Table';
import Stack from '@mui/material/Stack';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { TableNoData, TablePaginationCustom } from 'src/components/table';

import { TableLoadingRows, ManagementTableHead } from './shared';

type Props = {
  loading: boolean;
  rows: CacheAffinityItem[];
  table: UseTableReturn;
  total: number;
  onDelete: (row: CacheAffinityItem) => void;
};

const EMPTY_VALUE = '-';

export function CacheMonitoringTable({ loading, rows, table, total, onDelete }: Props) {
  const { t } = useTranslate('admin');
  const tableHead = useCacheMonitoringTableHead();

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 1180 }}>
          <ManagementTableHead head={tableHead} />
          <TableBody>
            {loading ? (
              <TableLoadingRows head={tableHead} rows={table.rowsPerPage} />
            ) : (
              rows.map((row) => (
                <CacheMonitoringTableRow key={cacheRowKey(row)} row={row} onDelete={onDelete} />
              ))
            )}
            <TableNoData title={t('common.noData')} notFound={!loading && rows.length === 0} />
          </TableBody>
        </Table>
      </Scrollbar>
      <TablePaginationCustom
        page={table.page}
        count={total}
        rowsPerPage={table.rowsPerPage}
        onPageChange={table.onChangePage}
        onRowsPerPageChange={table.onChangeRowsPerPage}
      />
    </>
  );
}

function CacheMonitoringTableRow({
  row,
  onDelete,
}: {
  row: CacheAffinityItem;
  onDelete: (row: CacheAffinityItem) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover>
      <TableCell>
        <PrimarySecondaryText primary={row.username || EMPTY_VALUE} secondary={row.user_email || row.user_id || EMPTY_VALUE} />
      </TableCell>
      <TableCell>
        <PrimarySecondaryText primary={row.token_name || row.token_prefix || EMPTY_VALUE} secondary={row.affinity_key} monoSecondary />
      </TableCell>
      <TableCell>
        <PrimarySecondaryText primary={row.provider_name || row.provider_id} secondary={row.provider_key_name || row.endpoint_base_url || row.endpoint_id} />
      </TableCell>
      <TableCell>
        <PrimarySecondaryText primary={row.model_name || row.model_id} secondary={row.model_id} monoSecondary />
      </TableCell>
      <TableCell>
        <Chip label={row.api_format} size="small" variant="outlined" />
      </TableCell>
      <TableCell>{formatRemainingSeconds(row.ttl_seconds)}</TableCell>
      <TableCell>{row.request_count}</TableCell>
      <TableCell align="right">
        <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
          <Tooltip title={t('common.delete')}>
            <IconButton color="error" onClick={() => onDelete(row)}>
              <Iconify icon="solar:trash-bin-trash-bold" />
            </IconButton>
          </Tooltip>
        </Box>
      </TableCell>
    </TableRow>
  );
}

function PrimarySecondaryText({
  primary,
  secondary,
  monoSecondary,
}: {
  primary: string;
  secondary: string;
  monoSecondary?: boolean;
}) {
  return (
    <Stack spacing={0.5}>
      <Typography variant="body2">{primary}</Typography>
      <Typography
        variant="caption"
        color="text.secondary"
        sx={monoSecondary ? { fontFamily: 'monospace' } : undefined}
      >
        {secondary}
      </Typography>
    </Stack>
  );
}

function useCacheMonitoringTableHead() {
  const { t } = useTranslate('admin');

  return useMemo<TableHeadCellProps[]>(
    () => [
      { id: 'user', label: t('cacheMonitoring.columns.user'), width: 180 },
      { id: 'key', label: t('cacheMonitoring.columns.key'), width: 220 },
      { id: 'provider', label: t('cacheMonitoring.columns.provider'), width: 220 },
      { id: 'model', label: t('cacheMonitoring.columns.model'), width: 220 },
      { id: 'api_format', label: t('cacheMonitoring.columns.apiFormat'), width: 140 },
      { id: 'ttl', label: t('cacheMonitoring.columns.ttl'), width: 140 },
      { id: 'request_count', label: t('cacheMonitoring.columns.requestCount'), width: 120 },
      { id: '', width: 88 },
    ],
    [t]
  );
}

function cacheRowKey(row: CacheAffinityItem) {
  return `${row.affinity_key}:${row.endpoint_id}:${row.model_id}:${row.api_format}`;
}

function formatRemainingSeconds(totalSeconds: number) {
  if (totalSeconds <= 0) {
    return '0s';
  }
  const days = Math.floor(totalSeconds / 86400);
  const hours = Math.floor((totalSeconds % 86400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  const parts = [
    days ? `${days}d` : null,
    hours ? `${hours}h` : null,
    minutes ? `${minutes}m` : null,
    !days && !hours ? `${seconds}s` : null,
  ].filter(Boolean);
  return parts.join(' ');
}

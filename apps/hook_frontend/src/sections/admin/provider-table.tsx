'use client';

import type { Provider } from 'src/types/provider';
import type { UseTableReturn, TableHeadCellProps } from 'src/components/table';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import {
  TableNoData,
  TablePaginationCustom,
  tableStickyActionCellSx,
  withStickyActionHeadCell,
} from 'src/components/table';

import { EnabledLabel, TableLoadingRows, ManagementTableHead } from './shared';
import { providerTypeLabel, providerOriginLabel } from './provider-management-utils';

export function ProviderTable({
  rows,
  total,
  loading,
  table,
  selectedId,
  onSelect,
  onEdit,
  onDelete,
  onAppendImport,
  onSyncSettings,
}: {
  rows: Provider[];
  total: number;
  loading: boolean;
  table: UseTableReturn;
  selectedId?: string;
  onSelect: (provider: Provider) => void;
  onEdit: (provider: Provider) => void;
  onDelete: (provider: Provider) => void;
  onAppendImport: (provider: Provider) => void;
  onSyncSettings: (provider: Provider) => void;
}) {
  const { t } = useTranslate('admin');
  const tableHead = providerTableHead(t);

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 1280 }}>
          <ManagementTableHead head={tableHead} />
          <TableBody>
            {loading ? (
              <TableLoadingRows head={tableHead} rows={table.rowsPerPage} />
            ) : (
              rows.map((row) => (
                <ProviderTableRow
                  key={row.id}
                  row={row}
                  selected={row.id === selectedId}
                  onSelect={onSelect}
                  onEdit={onEdit}
                  onDelete={onDelete}
                  onAppendImport={onAppendImport}
                  onSyncSettings={onSyncSettings}
                />
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

function ProviderTableRow({
  row,
  selected,
  onSelect,
  onEdit,
  onDelete,
  onAppendImport,
  onSyncSettings,
}: {
  row: Provider;
  selected: boolean;
  onSelect: (provider: Provider) => void;
  onEdit: (provider: Provider) => void;
  onDelete: (provider: Provider) => void;
  onAppendImport: (provider: Provider) => void;
  onSyncSettings: (provider: Provider) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover selected={selected} sx={{ cursor: 'pointer' }} onClick={() => onSelect(row)}>
      <TableCell>
        <Typography variant="subtitle2">{row.name}</Typography>
        <Typography variant="caption" color="text.secondary">
          {providerTypeLabel(row.provider_type, t)}
        </Typography>
      </TableCell>
      <TableCell>
        <Chip
          size="small"
          color={row.provider_origin === 'quick_import' ? 'info' : 'default'}
          label={providerOriginLabel(row.provider_origin, t)}
        />
      </TableCell>
      <TableCell>
        <Stack direction="row" flexWrap="wrap" sx={{ gap: 0.75 }}>
          <Chip
            size="small"
            label={`${t('providers.maxRetries')}: ${optionalValue(row.max_retries)}`}
          />
        </Stack>
      </TableCell>
      <TableCell>{row.priority}</TableCell>
      <TableCell>
        <EnabledLabel enabled={row.is_active} />
      </TableCell>
      <TableCell align="left" sx={tableStickyActionCellSx}>
        <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
          {row.provider_origin === 'quick_import' ? (
            <>
              <Tooltip title={t('actions.quickImportAppendTokens')}>
                <IconButton
                  onClick={(event) => {
                    event.stopPropagation();
                    onAppendImport(row);
                  }}
                >
                  <Iconify icon="solar:import-bold" />
                </IconButton>
              </Tooltip>
              <Tooltip title={t('providers.quickImportSyncSection')}>
                <IconButton
                  onClick={(event) => {
                    event.stopPropagation();
                    onSyncSettings(row);
                  }}
                >
                  <Iconify icon="solar:settings-bold" />
                </IconButton>
              </Tooltip>
            </>
          ) : null}
          <Tooltip title={t('common.edit')}>
            <IconButton
              onClick={(event) => {
                event.stopPropagation();
                onEdit(row);
              }}
            >
              <Iconify icon="solar:pen-bold" />
            </IconButton>
          </Tooltip>
          <Tooltip title={t('common.delete')}>
            <IconButton
              color="error"
              onClick={(event) => {
                event.stopPropagation();
                onDelete(row);
              }}
            >
              <Iconify icon="solar:trash-bin-trash-bold" />
            </IconButton>
          </Tooltip>
        </Box>
      </TableCell>
    </TableRow>
  );
}

function providerTableHead(
  t: (key: string, options?: Record<string, unknown>) => string
): TableHeadCellProps[] {
  return [
    { id: 'name', label: t('providers.name'), width: 220 },
    { id: 'provider_origin', label: t('providers.providerOrigin'), width: 130 },
    { id: 'request_config', label: t('providers.requestConfig') },
    { id: 'priority', label: t('providers.priority'), width: 100 },
    { id: 'status', label: t('common.status'), width: 120 },
    withStickyActionHeadCell({
      id: 'actions',
      label: t('common.actions'),
      width: 180,
      align: 'left',
    }),
  ];
}

function optionalValue(value?: number | null) {
  return value === null || value === undefined ? '-' : value;
}

'use client';

import type { BillingGroup } from 'src/types/group';
import type { UseTableReturn } from 'src/components/table';

import Box from '@mui/material/Box';
import Table from '@mui/material/Table';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import TableContainer from '@mui/material/TableContainer';
import TablePagination from '@mui/material/TablePagination';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';

import {
  EnabledLabel,
  TableLoadingRows,
  ManagementTableHead,
} from './shared';

export function BillingGroupTable({
  rows,
  total,
  loading,
  table,
  onEdit,
  onDelete,
}: {
  rows: BillingGroup[];
  total: number;
  loading: boolean;
  table: UseTableReturn;
  onEdit: (group: BillingGroup) => void;
  onDelete: (group: BillingGroup) => void;
}) {
  const { t } = useTranslate('admin');
  const tableHead = groupTableHead(t);

  return (
    <>
      <TableContainer>
        <Scrollbar>
          <Table size={table.dense ? 'small' : 'medium'} sx={{ minWidth: 1040 }}>
            <ManagementTableHead head={tableHead} />
            <TableBody>
              {loading ? <TableLoadingRows head={tableHead} /> : rows.map((row) => (
                <BillingGroupTableRow key={row.id} row={row} onEdit={onEdit} onDelete={onDelete} />
              ))}
              <EmptyRow loading={loading} rows={rows} colSpan={tableHead.length} />
            </TableBody>
          </Table>
        </Scrollbar>
      </TableContainer>
      <TablePagination
        page={table.page}
        component="div"
        count={total}
        rowsPerPage={table.rowsPerPage}
        rowsPerPageOptions={[5, 10, 25]}
        onPageChange={table.onChangePage}
        onRowsPerPageChange={table.onChangeRowsPerPage}
      />
    </>
  );
}

function BillingGroupTableRow({
  row,
  onEdit,
  onDelete,
}: {
  row: BillingGroup;
  onEdit: (group: BillingGroup) => void;
  onDelete: (group: BillingGroup) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover>
      <TableCell>
        <Typography variant="subtitle2">{row.code}</Typography>
        <Typography variant="caption" color="text.secondary">
          {row.description || t('common.none')}
        </Typography>
      </TableCell>
      <TableCell>{row.name}</TableCell>
      <TableCell>{row.billing_multiplier}</TableCell>
      <TableCell>{modelAccessText(row, t)}</TableCell>
      <TableCell>{providerAccessText(row, t)}</TableCell>
      <TableCell><EnabledLabel enabled={row.is_active} /></TableCell>
      <TableCell>{row.is_system ? t('common.system') : t('common.custom')}</TableCell>
      <TableCell>{row.sort_order}</TableCell>
      <TableCell align="right">
        <GroupActions row={row} onEdit={onEdit} onDelete={onDelete} />
      </TableCell>
    </TableRow>
  );
}

function GroupActions({
  row,
  onEdit,
  onDelete,
}: {
  row: BillingGroup;
  onEdit: (group: BillingGroup) => void;
  onDelete: (group: BillingGroup) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <>
      <Tooltip title={t('common.edit')}>
        <IconButton onClick={() => onEdit(row)}>
          <Iconify icon="solar:pen-bold" />
        </IconButton>
      </Tooltip>
      <Tooltip title={t('common.delete')}>
        <span>
          <IconButton disabled={row.is_system} color="error" onClick={() => onDelete(row)}>
            <Iconify icon="solar:trash-bin-trash-bold" />
          </IconButton>
        </span>
      </Tooltip>
    </>
  );
}

function EmptyRow({
  loading,
  rows,
  colSpan,
}: {
  loading: boolean;
  rows: BillingGroup[];
  colSpan: number;
}) {
  const { t } = useTranslate('admin');
  if (loading || rows.length > 0) return null;
  return (
    <TableRow>
      <TableCell colSpan={colSpan}>
        <Box sx={{ py: 4, textAlign: 'center', color: 'text.secondary' }}>{t('common.noData')}</Box>
      </TableCell>
    </TableRow>
  );
}

function groupTableHead(t: (key: string, options?: Record<string, unknown>) => string) {
  return [
    { id: 'code', label: t('common.code') },
    { id: 'name', label: t('common.name') },
    { id: 'billing_multiplier', label: t('fields.billingMultiplier') },
    { id: 'allowed_model_ids', label: t('fields.allowedModels') },
    { id: 'allowed_provider_ids', label: t('fields.allowedProviders') },
    { id: 'status', label: t('common.status') },
    { id: 'system', label: t('common.system') },
    { id: 'sort_order', label: t('common.sortOrder') },
    { id: '', width: 96 },
  ];
}

function modelAccessText(group: BillingGroup, t: (key: string, options?: Record<string, unknown>) => string) {
  return group.allowed_model_ids.length === 0
    ? t('billingGroups.allModels')
    : t('billingGroups.selectedModelCount', { count: group.allowed_model_ids.length });
}

function providerAccessText(group: BillingGroup, t: (key: string, options?: Record<string, unknown>) => string) {
  return group.allowed_provider_ids.length === 0
    ? t('billingGroups.allProviders')
    : t('billingGroups.selectedProviderCount', { count: group.allowed_provider_ids.length });
}

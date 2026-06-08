'use client';

import type { BillingGroup } from 'src/types/group';
import type { UserGroup } from 'src/types/user-group';
import type { UseTableReturn } from 'src/components/table';
import type { ProviderGroup, ProviderKeyGroup } from 'src/types/provider-group';

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

import { userGroupSelectionLabel } from './user-group-utils';
import {
  EnabledLabel,
  TableLoadingRows,
  ManagementTableHead,
} from './shared';

export function BillingGroupTable({
  rows,
  userGroups,
  providerGroups,
  providerKeyGroups,
  total,
  loading,
  table,
  onView,
  onEdit,
  onDelete,
}: {
  rows: BillingGroup[];
  userGroups: UserGroup[];
  providerGroups: ProviderGroup[];
  providerKeyGroups: ProviderKeyGroup[];
  total: number;
  loading: boolean;
  table: UseTableReturn;
  onView: (group: BillingGroup) => void;
  onEdit: (group: BillingGroup) => void;
  onDelete: (group: BillingGroup) => void;
}) {
  const { t } = useTranslate('admin');
  const tableHead = groupTableHead(t);

  return (
    <>
      <TableContainer>
        <Scrollbar>
          <Table size={table.dense ? 'small' : 'medium'} sx={{ minWidth: 1180 }}>
            <ManagementTableHead head={tableHead} />
            <TableBody>
              {loading ? (
                <TableLoadingRows head={tableHead} />
              ) : (
                rows.map((row) => (
                  <BillingGroupTableRow
                    key={row.id}
                    row={row}
                    userGroups={userGroups}
                    providerGroups={providerGroups}
                    providerKeyGroups={providerKeyGroups}
                    onView={onView}
                    onEdit={onEdit}
                    onDelete={onDelete}
                  />
                ))
              )}
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
  userGroups,
  providerGroups,
  providerKeyGroups,
  onView,
  onEdit,
  onDelete,
}: {
  row: BillingGroup;
  userGroups: UserGroup[];
  providerGroups: ProviderGroup[];
  providerKeyGroups: ProviderKeyGroup[];
  onView: (group: BillingGroup) => void;
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
      <TableCell>{accessScopeText(row, providerGroups, providerKeyGroups, t)}</TableCell>
      <TableCell>{userGroupSelectionLabel(row.visible_user_group_codes, userGroups, t)}</TableCell>
      <TableCell><EnabledLabel enabled={row.is_active} /></TableCell>
      <TableCell>{row.is_system ? t('common.system') : t('common.custom')}</TableCell>
      <TableCell>{row.sort_order}</TableCell>
      <TableCell align="right">
        <GroupActions row={row} onView={onView} onEdit={onEdit} onDelete={onDelete} />
      </TableCell>
    </TableRow>
  );
}

function GroupActions({
  row,
  onView,
  onEdit,
  onDelete,
}: {
  row: BillingGroup;
  onView: (group: BillingGroup) => void;
  onEdit: (group: BillingGroup) => void;
  onDelete: (group: BillingGroup) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <>
      <Tooltip title={t('common.details')}>
        <IconButton onClick={() => onView(row)}>
          <Iconify icon="solar:eye-bold" />
        </IconButton>
      </Tooltip>
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
    { id: 'access_scope', label: t('billingGroups.accessScope') },
    { id: 'visible_user_group_codes', label: t('fields.visibleUserGroups') },
    { id: 'status', label: t('common.status') },
    { id: 'system', label: t('common.system') },
    { id: 'sort_order', label: t('common.sortOrder') },
    { id: '', width: 144 },
  ];
}

function modelAccessText(group: BillingGroup, t: (key: string, options?: Record<string, unknown>) => string) {
  return group.allowed_model_ids.length === 0
    ? t('billingGroups.allModels')
    : t('billingGroups.selectedModelCount', { count: group.allowed_model_ids.length });
}

function accessScopeText(
  group: BillingGroup,
  providerGroups: ProviderGroup[],
  providerKeyGroups: ProviderKeyGroup[],
  t: (key: string, options?: Record<string, unknown>) => string
) {
  if (group.allowed_provider_key_group_ids.length > 0) {
    return groupNamesText(group.allowed_provider_key_group_ids, providerKeyGroups, t);
  }
  if (group.allowed_provider_group_ids.length > 0) {
    return groupNamesText(group.allowed_provider_group_ids, providerGroups, t);
  }
  return t('billingGroups.accessModeUnrestricted');
}

function groupNamesText(
  ids: string[],
  groups: Array<ProviderGroup | ProviderKeyGroup>,
  t: (key: string, options?: Record<string, unknown>) => string
) {
  if (ids.length > 2) return t('billingGroups.selectedGroupCount', { count: ids.length });
  const labels = new Map(groups.map((group) => [group.id, group.name]));
  return ids.map((id) => labels.get(id) ?? id).join(', ');
}

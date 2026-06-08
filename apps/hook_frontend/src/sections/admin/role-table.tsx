'use client';

import type { Role } from 'src/types/rbac';
import type { UseTableReturn, TableHeadCellProps } from 'src/components/table';

import { useMemo } from 'react';

import Box from '@mui/material/Box';
import Table from '@mui/material/Table';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import {
  TableNoData,
  TablePaginationCustom,
  tableStickyActionCellSx,
  withStickyActionHeadCell,
} from 'src/components/table';

import {
  EnabledLabel,
  BooleanLabel,
  TableLoadingRows,
  ManagementTableHead,
} from './shared';

type Props = {
  loading: boolean;
  rows: Role[];
  table: UseTableReturn;
  total: number;
  onDelete: (role: Role) => void;
  onEdit: (role: Role) => void;
  onPermissions: (role: Role) => void;
};

export function RoleTable({ loading, rows, table, total, onDelete, onEdit, onPermissions }: Props) {
  const { t } = useTranslate('admin');
  const tableHead = useRoleTableHead();

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 1050 }}>
          <ManagementTableHead head={tableHead} />
          <TableBody>
            {loading ? (
              <TableLoadingRows head={tableHead} rows={table.rowsPerPage} />
            ) : (
              rows.map((row) => (
                <RoleTableRow
                  key={row.code}
                  row={row}
                  onDelete={onDelete}
                  onEdit={onEdit}
                  onPermissions={onPermissions}
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

function RoleTableRow({
  row,
  onDelete,
  onEdit,
  onPermissions,
}: {
  row: Role;
  onDelete: (role: Role) => void;
  onEdit: (role: Role) => void;
  onPermissions: (role: Role) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover>
      <TableCell>{row.name}</TableCell>
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.code}</TableCell>
      <TableCell>{row.description || '-'}</TableCell>
      <TableCell>{row.sort_order}</TableCell>
      <TableCell>
        <EnabledLabel enabled={row.enabled} />
      </TableCell>
      <TableCell>
        <BooleanLabel enabled={row.system} trueText={t('common.system')} falseText={t('common.custom')} />
      </TableCell>
      <TableCell align="left" sx={tableStickyActionCellSx}>
        <RoleTableActions row={row} onDelete={onDelete} onEdit={onEdit} onPermissions={onPermissions} />
      </TableCell>
    </TableRow>
  );
}

function RoleTableActions({
  row,
  onDelete,
  onEdit,
  onPermissions,
}: {
  row: Role;
  onDelete: (role: Role) => void;
  onEdit: (role: Role) => void;
  onPermissions: (role: Role) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
      <Tooltip title={t('common.permissions')}>
        <IconButton onClick={() => onPermissions(row)}>
          <Iconify icon="solar:shield-keyhole-bold-duotone" />
        </IconButton>
      </Tooltip>
      <Tooltip title={t('common.edit')}>
        <span>
          <IconButton disabled={row.system} onClick={() => onEdit(row)}>
            <Iconify icon="solar:pen-bold" />
          </IconButton>
        </span>
      </Tooltip>
      <Tooltip title={t('common.delete')}>
        <span>
          <IconButton color="error" disabled={row.system} onClick={() => onDelete(row)}>
            <Iconify icon="solar:trash-bin-trash-bold" />
          </IconButton>
        </span>
      </Tooltip>
    </Box>
  );
}

function useRoleTableHead() {
  const { t } = useTranslate('admin');

  return useMemo<TableHeadCellProps[]>(
    () => [
      { id: 'name', label: t('common.role'), width: 220 },
      { id: 'code', label: t('common.code'), width: 200 },
      { id: 'description', label: t('common.description') },
      { id: 'sort_order', label: t('common.sort'), width: 100 },
      { id: 'enabled', label: t('common.status'), width: 120 },
      { id: 'system', label: t('common.type'), width: 120 },
      withStickyActionHeadCell({ id: 'actions', label: t('common.actions'), width: 144, align: 'left' }),
    ],
    [t]
  );
}

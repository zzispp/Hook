'use client';

import type { MenuItem as RbacMenuItem } from 'src/types/rbac';
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
import { TableNoData, TablePaginationCustom } from 'src/components/table';

import {
  EnabledLabel,
  TableLoadingRows,
  translatedMenuItem,
  ManagementTableHead,
} from './shared';

type Props = {
  loading: boolean;
  rows: RbacMenuItem[];
  sectionNameById: Map<string, string>;
  table: UseTableReturn;
  total: number;
  onBindApis: (item: RbacMenuItem) => void;
  onDelete: (item: RbacMenuItem) => void;
  onEdit: (item: RbacMenuItem) => void;
};

export function MenuItemsTable({
  loading,
  rows,
  sectionNameById,
  table,
  total,
  onBindApis,
  onDelete,
  onEdit,
}: Props) {
  const { t } = useTranslate('admin');
  const tableHead = useMenuItemsTableHead();

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 1100 }}>
          <ManagementTableHead head={tableHead} />
          <TableBody>
            {loading ? (
              <TableLoadingRows head={tableHead} rows={table.rowsPerPage} />
            ) : (
              rows.map((row) => (
                <MenuItemsTableRow
                  key={row.id}
                  row={row}
                  sectionName={sectionNameById.get(row.section_id) ?? row.section_id}
                  onBindApis={onBindApis}
                  onDelete={onDelete}
                  onEdit={onEdit}
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

function MenuItemsTableRow({
  row,
  sectionName,
  onBindApis,
  onDelete,
  onEdit,
}: {
  row: RbacMenuItem;
  sectionName: string;
  onBindApis: (item: RbacMenuItem) => void;
  onDelete: (item: RbacMenuItem) => void;
  onEdit: (item: RbacMenuItem) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover>
      <TableCell>{translatedMenuItem(row, t)}</TableCell>
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.code}</TableCell>
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.path}</TableCell>
      <TableCell>{sectionName}</TableCell>
      <TableCell>{row.sort_order}</TableCell>
      <TableCell>
        <EnabledLabel enabled={row.enabled} />
      </TableCell>
      <TableCell align="right">
        <MenuItemsTableActions row={row} onBindApis={onBindApis} onDelete={onDelete} onEdit={onEdit} />
      </TableCell>
    </TableRow>
  );
}

function MenuItemsTableActions({
  row,
  onBindApis,
  onDelete,
  onEdit,
}: {
  row: RbacMenuItem;
  onBindApis: (item: RbacMenuItem) => void;
  onDelete: (item: RbacMenuItem) => void;
  onEdit: (item: RbacMenuItem) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
      <Tooltip title={t('actions.bindApis')}>
        <IconButton onClick={() => onBindApis(row)}>
          <Iconify icon="eva:link-2-fill" />
        </IconButton>
      </Tooltip>
      <Tooltip title={t('common.edit')}>
        <IconButton onClick={() => onEdit(row)}>
          <Iconify icon="solar:pen-bold" />
        </IconButton>
      </Tooltip>
      <Tooltip title={t('common.delete')}>
        <IconButton color="error" onClick={() => onDelete(row)}>
          <Iconify icon="solar:trash-bin-trash-bold" />
        </IconButton>
      </Tooltip>
    </Box>
  );
}

function useMenuItemsTableHead() {
  const { t } = useTranslate('admin');

  return useMemo<TableHeadCellProps[]>(
    () => [
      { id: 'title', label: t('common.title'), width: 220 },
      { id: 'code', label: t('common.code'), width: 220 },
      { id: 'path', label: t('common.path') },
      { id: 'section', label: t('common.section'), width: 180 },
      { id: 'sort_order', label: t('common.sort'), width: 100 },
      { id: 'enabled', label: t('common.status'), width: 120 },
      { id: '', width: 144 },
    ],
    [t]
  );
}

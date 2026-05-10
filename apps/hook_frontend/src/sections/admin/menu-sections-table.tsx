'use client';

import type { MenuSection } from 'src/types/rbac';
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
  ManagementTableHead,
} from './shared';

type Props = {
  loading: boolean;
  rows: MenuSection[];
  table: UseTableReturn;
  total: number;
  onDelete: (section: MenuSection) => void;
  onEdit: (section: MenuSection) => void;
};

export function MenuSectionsTable({ loading, rows, table, total, onDelete, onEdit }: Props) {
  const { t } = useTranslate('admin');
  const tableHead = useMenuSectionsTableHead();

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 820 }}>
          <ManagementTableHead head={tableHead} />
          <TableBody>
            {loading ? (
              <TableLoadingRows head={tableHead} rows={table.rowsPerPage} />
            ) : (
              rows.map((row) => (
                <MenuSectionsTableRow key={row.id} row={row} onDelete={onDelete} onEdit={onEdit} />
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

function MenuSectionsTableRow({
  row,
  onDelete,
  onEdit,
}: {
  row: MenuSection;
  onDelete: (section: MenuSection) => void;
  onEdit: (section: MenuSection) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover>
      <TableCell>{row.subheader}</TableCell>
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.code}</TableCell>
      <TableCell>{row.sort_order}</TableCell>
      <TableCell>
        <EnabledLabel enabled={row.enabled} />
      </TableCell>
      <TableCell align="right">
        <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
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
      </TableCell>
    </TableRow>
  );
}

function useMenuSectionsTableHead() {
  const { t } = useTranslate('admin');

  return useMemo<TableHeadCellProps[]>(
    () => [
      { id: 'subheader', label: t('fields.subheader'), width: 240 },
      { id: 'code', label: t('common.code'), width: 240 },
      { id: 'sort_order', label: t('common.sort'), width: 100 },
      { id: 'enabled', label: t('common.status'), width: 120 },
      { id: '', width: 144 },
    ],
    [t]
  );
}

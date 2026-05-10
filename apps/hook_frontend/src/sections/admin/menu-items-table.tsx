'use client';

import type { MenuSection, MenuItem as RbacMenuItem } from 'src/types/rbac';
import type { UseTableReturn, TableHeadCellProps } from 'src/components/table';

import { useMemo } from 'react';

import Box from '@mui/material/Box';
import Table from '@mui/material/Table';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';
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
  sections: MenuSection[];
  loading: boolean;
  rows: RbacMenuItem[];
  table: UseTableReturn;
  total: number;
  onBindApis: (item: RbacMenuItem) => void;
  onDelete: (item: RbacMenuItem) => void;
  onEdit: (item: RbacMenuItem) => void;
};

export function MenuItemsTable({
  sections,
  loading,
  rows,
  table,
  total,
  onBindApis,
  onDelete,
  onEdit,
}: Props) {
  const { t } = useTranslate('admin');
  const tableHead = useMenuItemsTableHead();
  const groupedRows = useMemo(() => menuRowsBySection(rows, sections), [rows, sections]);

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 1100 }}>
          <ManagementTableHead head={tableHead} />
          <TableBody>
            {loading ? (
              <TableLoadingRows head={tableHead} rows={table.rowsPerPage} />
            ) : (
              groupedRows.map((group) => [
                <MenuSectionHeaderRow key={`section-${group.sectionId}`} sectionName={group.sectionName} itemCount={group.items.length} />,
                ...group.items.map((row) => (
                  <MenuItemsTableRow
                    key={row.id}
                    row={row}
                    onBindApis={onBindApis}
                    onDelete={onDelete}
                    onEdit={onEdit}
                  />
                )),
              ])
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

function MenuSectionHeaderRow({
  sectionName,
  itemCount,
}: {
  sectionName: string;
  itemCount: number;
}) {
  const { t } = useTranslate('admin');

  return (
    <TableRow>
      <TableCell colSpan={6} sx={{ bgcolor: 'background.neutral', py: 1.25 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <Iconify width={18} icon="solar:file-bold-duotone" />
          <Typography variant="subtitle2">{sectionName}</Typography>
          <Typography variant="caption" sx={{ color: 'text.secondary' }}>
            {t('menu.itemsCount', { count: itemCount })}
          </Typography>
        </Box>
      </TableCell>
    </TableRow>
  );
}

function MenuItemsTableRow({
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
  return (
    <TableRow hover>
      <TableCell>
        <Box sx={{ display: 'flex', alignItems: 'center', pl: 3 }}>
          <Box component="span" sx={{ mr: 1, width: 10, height: 1, bgcolor: 'divider' }} />
          {row.title}
        </Box>
      </TableCell>
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.code}</TableCell>
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.path}</TableCell>
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

function menuRowsBySection(rows: RbacMenuItem[], sections: MenuSection[]) {
  const sectionOrder = new Map(sections.map((section, index) => [section.id, index]));
  const sectionNameById = new Map(sections.map((section) => [section.id, section.subheader]));
  const groupMap = new Map<string, RbacMenuItem[]>();

  for (const row of rows) {
    groupMap.set(row.section_id, [...(groupMap.get(row.section_id) ?? []), row]);
  }

  return Array.from(groupMap.entries())
    .sort(
      ([left], [right]) =>
        (sectionOrder.get(left) ?? Number.MAX_SAFE_INTEGER) -
        (sectionOrder.get(right) ?? Number.MAX_SAFE_INTEGER)
    )
    .map(([sectionId, items]) => ({
      sectionId,
      sectionName: sectionNameById.get(sectionId) ?? sectionId,
      items: [...items].sort((left, right) => left.sort_order - right.sort_order),
    }));
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
      { id: 'sort_order', label: t('common.sort'), width: 100 },
      { id: 'enabled', label: t('common.status'), width: 120 },
      { id: '', width: 144 },
    ],
    [t]
  );
}

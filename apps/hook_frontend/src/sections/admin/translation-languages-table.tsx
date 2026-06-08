'use client';

import type { TranslationLanguage } from 'src/types/i18n';
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

import { EnabledLabel, BooleanLabel, TableLoadingRows, ManagementTableHead } from './shared';

type Props = {
  loading: boolean;
  rows: TranslationLanguage[];
  table: UseTableReturn;
  total: number;
  onDelete: (language: TranslationLanguage) => void;
  onEdit: (language: TranslationLanguage) => void;
};

export function TranslationLanguagesTable({ loading, rows, table, total, onDelete, onEdit }: Props) {
  const { t } = useTranslate('admin');
  const tableHead = useTranslationLanguagesTableHead();

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 900 }}>
          <ManagementTableHead head={tableHead} />
          <TableBody>
            {loading ? (
              <TableLoadingRows head={tableHead} rows={table.rowsPerPage} />
            ) : (
              rows.map((row) => (
                <TranslationLanguageTableRow key={row.code} row={row} onDelete={onDelete} onEdit={onEdit} />
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

function TranslationLanguageTableRow({
  row,
  onDelete,
  onEdit,
}: {
  row: TranslationLanguage;
  onDelete: (language: TranslationLanguage) => void;
  onEdit: (language: TranslationLanguage) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover>
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.code}</TableCell>
      <TableCell>{row.name}</TableCell>
      <TableCell>{row.native_name}</TableCell>
      <TableCell>{row.sort_order}</TableCell>
      <TableCell>
        <EnabledLabel enabled={row.enabled} />
      </TableCell>
      <TableCell>
        <BooleanLabel enabled={row.system} trueText={t('common.system')} falseText={t('common.custom')} />
      </TableCell>
      <TableCell align="left" sx={tableStickyActionCellSx}>
        <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
          <Tooltip title={t('common.edit')}>
            <IconButton onClick={() => onEdit(row)}>
              <Iconify icon="solar:pen-bold" />
            </IconButton>
          </Tooltip>
          <Tooltip title={t('common.delete')}>
            <span>
              <IconButton color="error" disabled={row.system} onClick={() => onDelete(row)}>
                <Iconify icon="solar:trash-bin-trash-bold" />
              </IconButton>
            </span>
          </Tooltip>
        </Box>
      </TableCell>
    </TableRow>
  );
}

function useTranslationLanguagesTableHead() {
  const { t } = useTranslate('admin');

  return useMemo<TableHeadCellProps[]>(
    () => [
      { id: 'code', label: t('translations.fields.langCode'), width: 140 },
      { id: 'name', label: t('common.name'), width: 180 },
      { id: 'native_name', label: t('translations.fields.nativeName'), width: 180 },
      { id: 'sort_order', label: t('common.sort'), width: 100 },
      { id: 'enabled', label: t('common.status'), width: 120 },
      { id: 'system', label: t('common.type'), width: 120 },
      withStickyActionHeadCell({ id: 'actions', label: t('common.actions'), width: 120, align: 'left' }),
    ],
    [t]
  );
}

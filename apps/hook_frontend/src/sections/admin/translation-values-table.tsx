'use client';

import type { TranslationLanguage } from 'src/types/i18n';
import type { TranslationValueRow } from './translation-management-utils';
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

import { EnabledLabel, TableLoadingRows, ManagementTableHead } from './shared';

type Props = {
  languages: TranslationLanguage[];
  loading: boolean;
  rows: TranslationValueRow[];
  table: UseTableReturn;
  total: number;
  onDelete: (row: TranslationValueRow) => void;
  onEdit: (row: TranslationValueRow) => void;
};

export function TranslationValuesTable({ languages, loading, rows, table, total, onDelete, onEdit }: Props) {
  const { t } = useTranslate('admin');
  const tableHead = useTranslationValueTableHead(languages);
  const groups = useMemo(() => rowsByGroup(rows), [rows]);

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 1120 }}>
          <ManagementTableHead head={tableHead} />
          <TableBody>
            {loading ? (
              <TableLoadingRows head={tableHead} rows={table.rowsPerPage} />
            ) : (
              groups.map((group) => [
                <TranslationGroupHeaderRow key={group.groupKey} colSpan={tableHead.length} group={group} />,
                ...group.rows.map((row) => (
                  <TranslationValueTableRow
                    key={row.id}
                    languages={languages}
                    row={row}
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

function TranslationGroupHeaderRow({
  colSpan,
  group,
}: {
  colSpan: number;
  group: TranslationGroup;
}) {
  return (
    <TableRow>
      <TableCell colSpan={colSpan} sx={{ bgcolor: 'background.neutral', py: 1.25 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <Iconify width={18} icon="solar:file-bold-duotone" />
          <Typography variant="subtitle2">{group.groupKey}</Typography>
          <Typography variant="caption" sx={{ color: 'text.secondary' }}>
            {group.rows.length}
          </Typography>
        </Box>
      </TableCell>
    </TableRow>
  );
}

function TranslationValueTableRow({
  languages,
  row,
  onDelete,
  onEdit,
}: {
  languages: TranslationLanguage[];
  row: TranslationValueRow;
  onDelete: (row: TranslationValueRow) => void;
  onEdit: (row: TranslationValueRow) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover>
      <TableCell>
        <Box sx={{ display: 'flex', alignItems: 'center', pl: 3 }}>
          <Box component="span" sx={{ mr: 1, width: 10, height: 1, bgcolor: 'divider' }} />
          <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
            {row.item_key}
          </Typography>
        </Box>
      </TableCell>
      {languages.map((language) => (
        <TableCell key={language.code} sx={{ maxWidth: 320 }}>
          <Typography variant="body2" noWrap>
            {row.values[language.code] ?? '-'}
          </Typography>
        </TableCell>
      ))}
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

function useTranslationValueTableHead(languages: TranslationLanguage[]) {
  const { t } = useTranslate('admin');

  return useMemo<TableHeadCellProps[]>(
    () => [
      { id: 'item_key', label: t('translations.fields.itemKey'), width: 260 },
      ...languages.map((language) => ({
        id: language.code,
        label: language.native_name,
        width: 260,
      })),
      { id: 'enabled', label: t('common.status'), width: 120 },
      { id: '', width: 120 },
    ],
    [languages, t]
  );
}

type TranslationGroup = {
  groupKey: string;
  rows: TranslationValueRow[];
};

function rowsByGroup(rows: TranslationValueRow[]): TranslationGroup[] {
  const groups = new Map<string, TranslationValueRow[]>();
  for (const row of rows) {
    groups.set(row.group_key, [...(groups.get(row.group_key) ?? []), row]);
  }
  return Array.from(groups.entries()).map(([groupKey, groupRows]) => ({ groupKey, rows: groupRows }));
}


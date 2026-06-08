'use client';

import type { UserGroup } from 'src/types/user-group';
import type { UseTableReturn, TableHeadCellProps } from 'src/components/table';

import Box from '@mui/material/Box';
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

import { isDefaultUserGroup } from './user-group-management-utils';
import {
  EnabledLabel,
  BooleanLabel,
  TableLoadingRows,
  ManagementTableHead,
} from './shared';

type Props = {
  loading: boolean;
  rows: UserGroup[];
  table: UseTableReturn;
  total: number;
  onAssign: (group: UserGroup) => void;
  onDelete: (group: UserGroup) => void;
  onEdit: (group: UserGroup) => void;
  onMembers: (group: UserGroup) => void;
};

export function UserGroupTable(props: Props) {
  const { t } = useTranslate('admin');
  const head = tableHead(t);

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 1040 }}>
          <ManagementTableHead head={head} />
          <TableBody>
            {props.loading ? (
              <TableLoadingRows head={head} rows={props.table.rowsPerPage} />
            ) : (
              props.rows.map((row) => <UserGroupTableRow key={row.code} row={row} props={props} />)
            )}
            <TableNoData title={t('common.noData')} notFound={!props.loading && props.rows.length === 0} />
          </TableBody>
        </Table>
      </Scrollbar>
      <TablePaginationCustom
        page={props.table.page}
        count={props.total}
        rowsPerPage={props.table.rowsPerPage}
        onPageChange={props.table.onChangePage}
        onRowsPerPageChange={props.table.onChangeRowsPerPage}
      />
    </>
  );
}

function UserGroupTableRow({ row, props }: { row: UserGroup; props: Props }) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover>
      <TableCell>
        <Stack spacing={0.25}>
          <Typography variant="subtitle2">{row.name}</Typography>
          <Typography variant="caption" color="text.secondary" sx={{ fontFamily: 'monospace' }}>
            {row.code}
          </Typography>
        </Stack>
      </TableCell>
      <TableCell>{row.description || '-'}</TableCell>
      <TableCell>{row.sort_order}</TableCell>
      <TableCell>
        <EnabledLabel enabled={row.is_active} />
      </TableCell>
      <TableCell>
        <BooleanLabel enabled={row.is_system} trueText={t('common.system')} falseText={t('common.custom')} />
      </TableCell>
      <TableCell align="left" sx={tableStickyActionCellSx}>
        <UserGroupActions row={row} props={props} />
      </TableCell>
    </TableRow>
  );
}

function UserGroupActions({ row, props }: { row: UserGroup; props: Props }) {
  const { t } = useTranslate('admin');
  const isDefault = isDefaultUserGroup(row);

  return (
    <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
      <Tooltip title={row.is_active ? t('userGroups.assignUsers') : t('userGroups.disabledCannotAssign')}>
        <span>
          <IconButton disabled={!row.is_active} onClick={() => props.onAssign(row)}>
            <Iconify icon="solar:user-plus-bold" />
          </IconButton>
        </span>
      </Tooltip>
      <Tooltip title={t('userGroups.members')}>
        <IconButton onClick={() => props.onMembers(row)}>
          <Iconify icon="solar:users-group-rounded-bold" />
        </IconButton>
      </Tooltip>
      <Tooltip title={t('common.edit')}>
        <IconButton onClick={() => props.onEdit(row)}>
          <Iconify icon="solar:pen-bold" />
        </IconButton>
      </Tooltip>
      <Tooltip title={isDefault ? t('userGroups.defaultCannotDelete') : t('common.delete')}>
        <span>
          <IconButton color="error" disabled={isDefault} onClick={() => props.onDelete(row)}>
            <Iconify icon="solar:trash-bin-trash-bold" />
          </IconButton>
        </span>
      </Tooltip>
    </Box>
  );
}

function tableHead(t: (key: string) => string): TableHeadCellProps[] {
  return [
    { id: 'name', label: t('common.name'), width: 240 },
    { id: 'description', label: t('common.description') },
    { id: 'sort_order', label: t('common.sortOrder'), width: 110 },
    { id: 'is_active', label: t('common.status'), width: 120 },
    { id: 'system', label: t('common.type'), width: 120 },
    withStickyActionHeadCell({ id: 'actions', label: t('common.actions'), width: 184, align: 'left' }),
  ];
}

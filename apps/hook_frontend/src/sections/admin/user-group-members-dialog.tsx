'use client';

import type { SystemUser } from 'src/types/rbac';
import type { UserGroup } from 'src/types/user-group';
import type { TableHeadCellProps } from 'src/components/table';

import { useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Table from '@mui/material/Table';
import Dialog from '@mui/material/Dialog';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';

import { useTranslate } from 'src/locales/use-locales';
import { useUserGroupMembers } from 'src/actions/user-groups';

import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { useTable, TableNoData, TablePaginationCustom } from 'src/components/table';

import { formatUserDateTime } from './user-management-utils';
import {
  EnabledLabel,
  TableLoadingRows,
  ManagementTableHead,
} from './shared';
import {
  toUserGroupFilters,
  AdminFiltersToolbar,
  DEFAULT_ADMIN_FILTERS,
} from './admin-filters-toolbar';

type Props = {
  group: UserGroup | null;
  onClose: () => void;
};

export function UserGroupMembersDialog({ group, onClose }: Props) {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'username' });
  const [filters, setFilters] = useState(DEFAULT_ADMIN_FILTERS);
  const members = useUserGroupMembers(
    group?.code,
    table.page,
    table.rowsPerPage,
    toUserGroupFilters(filters)
  );
  const head = memberTableHead(t);
  const handleFiltersChange = useCallback(
    (nextFilters: typeof DEFAULT_ADMIN_FILTERS) => {
      table.onResetPage();
      setFilters(nextFilters);
    },
    [table]
  );

  return (
    <Dialog open={!!group} fullWidth maxWidth="md" onClose={onClose}>
      <DialogTitle sx={titleSx}>
        <Box sx={{ flexGrow: 1, minWidth: 0 }}>
          <Typography variant="h6">{t('userGroups.membersTitle', { name: group?.name ?? '' })}</Typography>
          <Typography variant="caption" color="text.secondary" sx={{ fontFamily: 'monospace' }}>
            {group?.code ?? ''}
          </Typography>
        </Box>
        <Tooltip title={t('common.close')}>
          <IconButton onClick={onClose}>
            <Iconify icon="mingcute:close-line" />
          </IconButton>
        </Tooltip>
      </DialogTitle>
      <DialogContent dividers sx={{ p: 0 }}>
        <AdminFiltersToolbar
          filters={filters}
          searchPlaceholder={t('filters.searchUsers')}
          onChange={handleFiltersChange}
        />
        <Scrollbar>
          <Table sx={{ minWidth: 760 }}>
            <ManagementTableHead head={head} />
            <TableBody>
              {members.isLoading ? (
                <TableLoadingRows head={head} rows={table.rowsPerPage} />
              ) : (
                members.items.map((user) => <MemberRow key={user.id} user={user} />)
              )}
              <TableNoData title={t('common.noData')} notFound={!members.isLoading && members.items.length === 0} />
            </TableBody>
          </Table>
        </Scrollbar>
        <TablePaginationCustom
          page={table.page}
          count={members.total}
          rowsPerPage={table.rowsPerPage}
          onPageChange={table.onChangePage}
          onRowsPerPageChange={table.onChangeRowsPerPage}
        />
      </DialogContent>
    </Dialog>
  );
}

function MemberRow({ user }: { user: SystemUser }) {
  return (
    <TableRow hover>
      <TableCell>
        <Typography variant="subtitle2">{user.username}</Typography>
        <Typography variant="caption" color="text.secondary" sx={{ fontFamily: 'monospace' }}>
          {user.id}
        </Typography>
      </TableCell>
      <TableCell>{user.email}</TableCell>
      <TableCell>
        <EnabledLabel enabled={user.is_active} />
      </TableCell>
      <TableCell sx={{ color: 'text.secondary', whiteSpace: 'nowrap' }}>
        {formatUserDateTime(user.created_at)}
      </TableCell>
    </TableRow>
  );
}

function memberTableHead(t: (key: string) => string): TableHeadCellProps[] {
  return [
    { id: 'username', label: t('common.username'), width: 260 },
    { id: 'email', label: t('common.email') },
    { id: 'is_active', label: t('common.status'), width: 120 },
    { id: 'created_at', label: t('fields.createdAt'), width: 180 },
  ];
}

const titleSx = {
  display: 'flex',
  alignItems: 'flex-start',
  gap: 1,
};

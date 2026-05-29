'use client';

import type { UserGroup } from 'src/types/user-group';
import type { Role, SystemUser } from 'src/types/rbac';
import type { IconifyProps } from 'src/components/iconify';
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

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { TableNoData, TablePaginationCustom } from 'src/components/table';

import { displayUserGroup } from './user-group-utils';
import { providerColor, providerLabel } from '../profile/provider-utils';
import {
  EnabledLabel,
  BooleanLabel,
  TableLoadingRows,
  ManagementTableHead,
} from './shared';
import {
  displayRole,
  walletBalanceText,
  userRateLimitText,
  formatUserDateTime,
  walletConsumedText,
} from './user-management-utils';

type Props = {
  rows: SystemUser[];
  roles: Role[];
  userGroups: UserGroup[];
  total: number;
  loading: boolean;
  table: UseTableReturn;
  onEdit: (user: SystemUser) => void;
  onWallet: (user: SystemUser) => void;
  onTokens: (user: SystemUser) => void;
  onDelete: (user: SystemUser) => void;
};

export function UserTable(props: Props) {
  const { t } = useTranslate('admin');
  const head = tableHead(t);

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 1620 }}>
          <ManagementTableHead head={head} />
          <TableBody>
            {props.loading ? (
              <TableLoadingRows head={head} rows={props.table.rowsPerPage} />
            ) : (
              props.rows.map((row) => <UserTableRow key={row.id} row={row} props={props} />)
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

function UserTableRow({ row, props }: { row: SystemUser; props: Props }) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover>
      <TableCell>
        <UserCell user={row} />
      </TableCell>
      <TableCell>{row.email}</TableCell>
      <TableCell>
        <ProviderBadges user={row} />
      </TableCell>
      <TableCell>{displayRole(row.role, props.roles)}</TableCell>
      <TableCell>
        <UserGroupBadge user={row} groups={props.userGroups} />
      </TableCell>
      <TableCell>
        <WalletCell user={row} />
      </TableCell>
      <TableCell>{userRateLimitText(row, t)}</TableCell>
      <TableCell>
        <EnabledLabel enabled={row.is_active} />
      </TableCell>
      <TableCell>
        <BooleanLabel enabled={row.system} trueText={t('common.system')} falseText={t('common.local')} />
      </TableCell>
      <TableCell sx={{ color: 'text.secondary', whiteSpace: 'nowrap' }}>
        {formatUserDateTime(row.created_at)}
      </TableCell>
      <TableCell sx={{ color: 'text.secondary', whiteSpace: 'nowrap' }}>
        {formatUserDateTime(row.last_login_at)}
      </TableCell>
      <TableCell align="right">
        <RowActions row={row} props={props} />
      </TableCell>
    </TableRow>
  );
}

function UserGroupBadge({ user, groups }: { user: SystemUser; groups: UserGroup[] }) {
  return (
    <Label color="info" variant="soft">
      {displayUserGroup(user.group_code, groups)}
    </Label>
  );
}

function UserCell({ user }: { user: SystemUser }) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={0.25}>
      <Typography variant="subtitle2">{user.username}</Typography>
      {!user.password_set && (
        <Label color="warning" variant="soft">
          {t('users.passwordNotSet')}
        </Label>
      )}
      <Typography variant="caption" color="text.secondary" sx={{ fontFamily: 'monospace' }}>
        {user.id}
      </Typography>
    </Stack>
  );
}

function ProviderBadges({ user }: { user: SystemUser }) {
  const { t } = useTranslate('admin');

  if (user.identities.length === 0) {
    return (
      <Typography variant="caption" color="text.secondary">
        {t('common.none')}
      </Typography>
    );
  }

  return (
    <Stack direction="row" spacing={0.75} useFlexGap flexWrap="wrap">
      {user.identities.map((identity) => (
        <Label key={identity.id} color={providerColor(identity.provider)} variant="soft">
          {providerLabel(identity.provider)}
        </Label>
      ))}
    </Stack>
  );
}

function WalletCell({ user }: { user: SystemUser }) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={0.25}>
      <Typography variant="body2">{t('users.balance', { amount: walletBalanceText(user, t) })}</Typography>
      <Typography variant="caption" color="text.secondary">
        {t('users.consumed', { amount: walletConsumedText(user) })}
      </Typography>
    </Stack>
  );
}

function RowActions({ row, props }: { row: SystemUser; props: Props }) {
  const { t } = useTranslate('admin');

  return (
    <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
      <ActionButton title={t('common.edit')} disabled={row.system} onClick={() => props.onEdit(row)} icon="solar:pen-bold" />
      <ActionButton title={t('users.manageWallet')} disabled={row.system} onClick={() => props.onWallet(row)} icon="solar:wad-of-money-bold" />
      <ActionButton title={t('users.manageTokens')} disabled={row.system} onClick={() => props.onTokens(row)} icon="solar:shield-keyhole-bold-duotone" />
      <ActionButton title={t('common.delete')} disabled={row.system} onClick={() => props.onDelete(row)} icon="solar:trash-bin-trash-bold" color="error" />
    </Box>
  );
}

function ActionButton({
  icon,
  title,
  color,
  disabled,
  onClick,
}: {
  icon: IconifyProps['icon'];
  title: string;
  color?: 'error';
  disabled: boolean;
  onClick: VoidFunction;
}) {
  return (
    <Tooltip title={title}>
      <span>
        <IconButton color={color} disabled={disabled} onClick={onClick}>
          <Iconify icon={icon} />
        </IconButton>
      </span>
    </Tooltip>
  );
}

function tableHead(t: (key: string) => string): TableHeadCellProps[] {
  return [
    { id: 'username', label: t('common.username'), width: 240 },
    { id: 'email', label: t('common.email'), width: 220 },
    { id: 'providers', label: t('users.providers'), width: 190 },
    { id: 'role', label: t('common.role'), width: 150 },
    { id: 'group_code', label: t('fields.userGroup'), width: 150 },
    { id: 'wallet', label: t('fields.wallet'), width: 190 },
    { id: 'statistics', label: t('fields.statistics'), width: 150 },
    { id: 'is_active', label: t('common.status'), width: 110 },
    { id: 'system', label: t('common.type'), width: 110 },
    { id: 'created_at', label: t('fields.createdAt'), width: 180 },
    { id: 'last_login_at', label: t('fields.lastLoginAt'), width: 180 },
    { id: '', width: 184 },
  ];
}

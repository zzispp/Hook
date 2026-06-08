'use client';

import type { Provider, ProviderApiKey } from 'src/types/provider';
import type { MemberOption, ProviderGroupKind } from './provider-groups-utils';
import type { ProviderGroup, ProviderKeyGroup } from 'src/types/provider-group';

import Box from '@mui/material/Box';
import Table from '@mui/material/Table';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { TableNoData } from 'src/components/table';
import { Scrollbar } from 'src/components/scrollbar';

import { TableLoadingRows, ManagementTableHead } from './shared';
import {
  memberLabels,
  groupMemberIds,
  providerMemberOptions,
  providerKeyMemberOptions,
} from './provider-groups-utils';

type GroupRow = ProviderGroup | ProviderKeyGroup;

export function ProviderGroupsTable({
  kind,
  rows,
  loading,
  providers,
  keysByProvider,
  onEdit,
  onDelete,
}: {
  kind: ProviderGroupKind;
  rows: GroupRow[];
  loading: boolean;
  providers: Pick<Provider, 'id' | 'name' | 'provider_type'>[];
  keysByProvider: Record<string, ProviderApiKey[]>;
  onEdit: (group: GroupRow) => void;
  onDelete: (group: GroupRow) => void;
}) {
  const { t } = useTranslate('admin');
  const tableHead = groupTableHead(t, kind);
  const memberOptions = groupMemberOptions(kind, providers, keysByProvider);

  return (
    <Scrollbar>
      <Table sx={{ minWidth: 920 }}>
        <ManagementTableHead head={tableHead} />
        <TableBody>
          {loading ? (
            <TableLoadingRows head={tableHead} />
          ) : (
            rows.map((row) => (
              <ProviderGroupTableRow
                key={row.id}
                kind={kind}
                row={row}
                memberOptions={memberOptions}
                onEdit={onEdit}
                onDelete={onDelete}
              />
            ))
          )}
          <TableNoData title={t('common.noData')} notFound={!loading && rows.length === 0} />
        </TableBody>
      </Table>
    </Scrollbar>
  );
}

function ProviderGroupTableRow({
  kind,
  row,
  memberOptions,
  onEdit,
  onDelete,
}: {
  kind: ProviderGroupKind;
  row: GroupRow;
  memberOptions: MemberOption[];
  onEdit: (group: GroupRow) => void;
  onDelete: (group: GroupRow) => void;
}) {
  const { t } = useTranslate('admin');
  const memberNames = memberLabels(groupMemberIds(row, kind), memberOptions);

  return (
    <TableRow hover>
      <TableCell>
        <Typography variant="subtitle2">{row.name}</Typography>
        <Typography variant="caption" color="text.secondary">
          {row.description || t('common.none')}
        </Typography>
      </TableCell>
      <TableCell>{memberSummary(memberNames, t)}</TableCell>
      <TableCell>{row.sort_order}</TableCell>
      <TableCell>{row.updated_at}</TableCell>
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

function groupMemberOptions(
  kind: ProviderGroupKind,
  providers: Pick<Provider, 'id' | 'name' | 'provider_type'>[],
  keysByProvider: Record<string, ProviderApiKey[]>
) {
  return kind === 'provider'
    ? providerMemberOptions(providers)
    : providerKeyMemberOptions(providers, keysByProvider);
}

function groupTableHead(
  t: (key: string, options?: Record<string, unknown>) => string,
  kind: ProviderGroupKind
) {
  return [
    { id: 'name', label: t(kind === 'provider' ? 'providers.providerGroups' : 'providers.providerKeyGroups') },
    { id: 'members', label: t(kind === 'provider' ? 'providers.providerGroupMembers' : 'providers.providerKeyGroupMembers') },
    { id: 'sort_order', label: t('common.sortOrder'), width: 120 },
    { id: 'updated_at', label: t('fields.updatedAt'), width: 200 },
    { id: '', width: 96 },
  ];
}

function memberSummary(names: string[], t: (key: string, options?: Record<string, unknown>) => string) {
  if (names.length === 0) return t('providers.emptyGroupMembers');
  if (names.length > 2) return t('providers.selectedGroupMemberCount', { count: names.length });
  return names.join(', ');
}

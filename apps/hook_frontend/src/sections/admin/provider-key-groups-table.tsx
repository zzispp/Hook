'use client';

import type { Provider, ProviderApiKey } from 'src/types/provider';
import type { ProviderKeyGroup } from 'src/types/provider-key-group';

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
  providerKeyMemberOptions,
  providerKeyGroupMemberIds,
} from './provider-key-groups-utils';

export function ProviderKeyGroupsTable({
  rows,
  loading,
  providers,
  keysByProvider,
  onEdit,
  onDelete,
}: {
  rows: ProviderKeyGroup[];
  loading: boolean;
  providers: Pick<Provider, 'id' | 'name' | 'provider_type'>[];
  keysByProvider: Record<string, ProviderApiKey[]>;
  onEdit: (group: ProviderKeyGroup) => void;
  onDelete: (group: ProviderKeyGroup) => void;
}) {
  const { t } = useTranslate('admin');
  const tableHead = keyGroupTableHead(t);
  const memberOptions = providerKeyMemberOptions(providers, keysByProvider);

  return (
    <Scrollbar>
      <Table sx={{ minWidth: 920 }}>
        <ManagementTableHead head={tableHead} />
        <TableBody>
          {loading ? (
            <TableLoadingRows head={tableHead} />
          ) : (
            rows.map((row) => (
              <ProviderKeyGroupTableRow
                key={row.id}
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

function ProviderKeyGroupTableRow({
  row,
  memberOptions,
  onEdit,
  onDelete,
}: {
  row: ProviderKeyGroup;
  memberOptions: { id: string; label: string }[];
  onEdit: (group: ProviderKeyGroup) => void;
  onDelete: (group: ProviderKeyGroup) => void;
}) {
  const { t } = useTranslate('admin');
  const memberNames = memberLabels(providerKeyGroupMemberIds(row), memberOptions);

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

function keyGroupTableHead(t: (key: string, options?: Record<string, unknown>) => string) {
  return [
    { id: 'name', label: t('providers.providerKeyGroups') },
    { id: 'members', label: t('providers.providerKeyGroupMembers') },
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

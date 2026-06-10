'use client';

import type { Provider } from 'src/types/provider';
import type { ProviderGroup } from 'src/types/provider-group';
import type { UseTableReturn, TableHeadCellProps } from 'src/components/table';

import { useMemo } from 'react';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
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

import { providerGroupMemberIds } from './provider-groups-utils';
import { EnabledLabel, TableLoadingRows, ManagementTableHead } from './shared';
import { providerTypeLabel, providerOriginLabel } from './provider-management-utils';

export function ProviderTable({
  rows,
  groups,
  total,
  loading,
  table,
  selectedId,
  onSelect,
  onEdit,
  onDelete,
  onAssociateGroups,
}: {
  rows: Provider[];
  groups: ProviderGroup[];
  total: number;
  loading: boolean;
  table: UseTableReturn;
  selectedId?: string;
  onSelect: (provider: Provider) => void;
  onEdit: (provider: Provider) => void;
  onDelete: (provider: Provider) => void;
  onAssociateGroups: (provider: Provider) => void;
}) {
  const { t } = useTranslate('admin');
  const tableHead = providerTableHead(t);
  const groupedRows = useMemo(() => providerRowsByGroup(rows, groups, t), [groups, rows, t]);

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 1280 }}>
          <ManagementTableHead head={tableHead} />
          <TableBody>
            {loading ? (
              <TableLoadingRows head={tableHead} rows={table.rowsPerPage} />
            ) : (
              groupedRows.map((group) => [
                <ProviderGroupHeaderRow
                  key={group.id}
                  name={group.name}
                  count={group.providers.length}
                />,
                ...group.providers.map((entry) => (
                  <ProviderTableRow
                    key={`${group.id}-${entry.provider.id}`}
                    row={entry.provider}
                    priority={entry.priority}
                    selected={entry.provider.id === selectedId}
                    onSelect={onSelect}
                    onEdit={onEdit}
                    onDelete={onDelete}
                    onAssociateGroups={onAssociateGroups}
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

function ProviderGroupHeaderRow({ name, count }: { name: string; count: number }) {
  const { t } = useTranslate('admin');

  return (
    <TableRow>
      <TableCell colSpan={6} sx={{ bgcolor: 'background.neutral', py: 1.25 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <Iconify width={18} icon="solar:file-bold-duotone" />
          <Typography variant="subtitle2">{name}</Typography>
          <Typography variant="caption" sx={{ color: 'text.secondary' }}>
            {t('providers.groupProviderCount', { count })}
          </Typography>
        </Box>
      </TableCell>
    </TableRow>
  );
}

function ProviderTableRow({
  row,
  priority,
  selected,
  onSelect,
  onEdit,
  onDelete,
  onAssociateGroups,
}: {
  row: Provider;
  priority: number;
  selected: boolean;
  onSelect: (provider: Provider) => void;
  onEdit: (provider: Provider) => void;
  onDelete: (provider: Provider) => void;
  onAssociateGroups: (provider: Provider) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover selected={selected} sx={{ cursor: 'pointer' }} onClick={() => onSelect(row)}>
      <TableCell>
        <Typography variant="subtitle2">{row.name}</Typography>
        <Typography variant="caption" color="text.secondary">
          {providerTypeLabel(row.provider_type, t)}
        </Typography>
      </TableCell>
      <TableCell>
        <Chip
          size="small"
          color={row.provider_origin === 'quick_import' ? 'info' : 'default'}
          label={providerOriginLabel(row.provider_origin, t)}
        />
      </TableCell>
      <TableCell>
        <Stack direction="row" flexWrap="wrap" sx={{ gap: 0.75 }}>
          <Chip
            size="small"
            label={`${t('providers.maxRetries')}: ${optionalValue(row.max_retries)}`}
          />
        </Stack>
      </TableCell>
      <TableCell>{priority}</TableCell>
      <TableCell>
        <EnabledLabel enabled={row.is_active} />
      </TableCell>
      <TableCell align="left" sx={tableStickyActionCellSx}>
        <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
          <Tooltip title={t('actions.associateProviderGroups')}>
            <IconButton
              onClick={(event) => {
                event.stopPropagation();
                onAssociateGroups(row);
              }}
            >
              <Iconify icon="eva:link-2-fill" />
            </IconButton>
          </Tooltip>
          <Tooltip title={t('common.edit')}>
            <IconButton
              onClick={(event) => {
                event.stopPropagation();
                onEdit(row);
              }}
            >
              <Iconify icon="solar:pen-bold" />
            </IconButton>
          </Tooltip>
          <Tooltip title={t('common.delete')}>
            <IconButton
              color="error"
              onClick={(event) => {
                event.stopPropagation();
                onDelete(row);
              }}
            >
              <Iconify icon="solar:trash-bin-trash-bold" />
            </IconButton>
          </Tooltip>
        </Box>
      </TableCell>
    </TableRow>
  );
}

function providerTableHead(
  t: (key: string, options?: Record<string, unknown>) => string
): TableHeadCellProps[] {
  return [
    { id: 'name', label: t('providers.name'), width: 220 },
    { id: 'provider_origin', label: t('providers.providerOrigin'), width: 130 },
    { id: 'request_config', label: t('providers.requestConfig') },
    { id: 'priority', label: t('providers.priority'), width: 100 },
    { id: 'status', label: t('common.status'), width: 120 },
    withStickyActionHeadCell({
      id: 'actions',
      label: t('common.actions'),
      width: 136,
      align: 'left',
    }),
  ];
}

function optionalValue(value?: number | null) {
  return value === null || value === undefined ? '-' : value;
}

function providerRowsByGroup(
  providers: Provider[],
  groups: ProviderGroup[],
  t: (key: string, options?: Record<string, unknown>) => string
) {
  const providersById = new Map(providers.map((provider) => [provider.id, provider]));
  const groupedProviderIds = new Set(groups.flatMap(providerGroupMemberIds));
  const boundGroups = sortedGroups(groups)
    .map((group) => ({
      id: group.id,
      name: group.name,
      providers: providerEntriesForGroup(group, providersById),
    }))
    .filter((group) => group.providers.length > 0);
  const unboundProviders = sortProviderEntriesByPriority(
    providers
      .filter((provider) => !groupedProviderIds.has(provider.id))
      .map((provider) => ({ provider, priority: provider.priority }))
  );

  return unboundProviders.length === 0
    ? boundGroups
    : [
        ...boundGroups,
        {
          id: '__unbound_provider_group__',
          name: t('providers.unboundProviderGroup'),
          providers: unboundProviders,
        },
      ];
}

function providerEntriesForGroup(group: ProviderGroup, providersById: Map<string, Provider>) {
  const entries = group.provider_members.flatMap((member) => {
    const provider = providersById.get(member.provider_id);
    return provider ? [{ provider, priority: member.priority }] : [];
  });
  return sortProviderEntriesByPriority(entries);
}

function sortProviderEntriesByPriority(entries: { provider: Provider; priority: number }[]) {
  return [...entries].sort(compareProviderEntriesByPriority);
}

function compareProviderEntriesByPriority(
  left: { provider: Provider; priority: number },
  right: { provider: Provider; priority: number }
) {
  return (
    left.priority - right.priority ||
    left.provider.name.localeCompare(right.provider.name) ||
    left.provider.id.localeCompare(right.provider.id)
  );
}

function sortedGroups(groups: ProviderGroup[]) {
  return [...groups].sort(
    (left, right) => left.sort_order - right.sort_order || left.name.localeCompare(right.name)
  );
}

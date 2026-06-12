'use client';

import type { ProviderKeyGroup } from 'src/types/provider-key-group';

import Checkbox from '@mui/material/Checkbox';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import ListItemText from '@mui/material/ListItemText';

import { useTranslate } from 'src/locales/use-locales';

import { ManagementDialog } from './shared';
import { selectedValues, selectedGroupLabel } from './provider-key-groups-utils';

export function ProviderKeyGroupAssociationDialog({
  open,
  targetName,
  groups,
  selectedIds,
  submitting,
  onClose,
  onSubmit,
  onSelectedIdsChange,
}: {
  open: boolean;
  targetName: string;
  groups: ProviderKeyGroup[];
  selectedIds: string[];
  submitting: boolean;
  onClose: () => void;
  onSubmit: () => void;
  onSelectedIdsChange: (ids: string[]) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <ManagementDialog
      open={open}
      title={t('dialogs.associateProviderKeyGroups')}
      submitting={submitting}
      description={t('helper.associateProviderKeyGroups', { name: targetName })}
      onClose={onClose}
      onSubmit={onSubmit}
    >
      <TextField
        select
        fullWidth
        label={t('providers.providerKeyGroups')}
        value={selectedIds}
        SelectProps={{
          multiple: true,
          renderValue: (selected) =>
            selectedGroupLabel(selected as string[], groups, t('providers.emptyGroupAssociations'), (count) =>
              t('providers.selectedGroupAssociationCount', { count })
            ),
        }}
        onChange={(event) => onSelectedIdsChange(selectedValues(event.target.value))}
      >
        {groups.length === 0 ? <MenuItem disabled value="">{t('common.noData')}</MenuItem> : null}
        {groups.map((group) => (
          <MenuItem key={group.id} value={group.id}>
            <Checkbox checked={selectedIds.includes(group.id)} />
            <ListItemText primary={group.name} secondary={group.description || undefined} />
          </MenuItem>
        ))}
      </TextField>
    </ManagementDialog>
  );
}

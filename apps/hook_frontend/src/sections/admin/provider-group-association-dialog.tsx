'use client';

import type { ProviderGroupKind } from './provider-groups-utils';
import type { ProviderGroup, ProviderKeyGroup } from 'src/types/provider-group';

import Checkbox from '@mui/material/Checkbox';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import ListItemText from '@mui/material/ListItemText';

import { useTranslate } from 'src/locales/use-locales';

import { ManagementDialog } from './shared';
import { selectedValues, selectedGroupLabel } from './provider-groups-utils';

type AssociationGroup = ProviderGroup | ProviderKeyGroup;

export function ProviderGroupAssociationDialog({
  kind,
  open,
  targetName,
  groups,
  selectedIds,
  submitting,
  onClose,
  onSubmit,
  onSelectedIdsChange,
}: {
  kind: ProviderGroupKind;
  open: boolean;
  targetName: string;
  groups: AssociationGroup[];
  selectedIds: string[];
  submitting: boolean;
  onClose: () => void;
  onSubmit: () => void;
  onSelectedIdsChange: (ids: string[]) => void;
}) {
  const { t } = useTranslate('admin');
  const titleKey = kind === 'provider' ? 'dialogs.associateProviderGroups' : 'dialogs.associateProviderKeyGroups';
  const helperKey = kind === 'provider' ? 'helper.associateProviderGroups' : 'helper.associateProviderKeyGroups';
  const labelKey = kind === 'provider' ? 'providers.providerGroups' : 'providers.providerKeyGroups';

  return (
    <ManagementDialog
      open={open}
      title={t(titleKey)}
      submitting={submitting}
      description={t(helperKey, { name: targetName })}
      onClose={onClose}
      onSubmit={onSubmit}
    >
      <TextField
        select
        fullWidth
        label={t(labelKey)}
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

'use client';

import type { Provider, ProviderApiKey } from 'src/types/provider';
import type { ProviderGroupKind, ProviderGroupForm } from './provider-groups-utils';

import Stack from '@mui/material/Stack';
import Checkbox from '@mui/material/Checkbox';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import ListItemText from '@mui/material/ListItemText';

import { useTranslate } from 'src/locales/use-locales';

import { TextFieldRow, ManagementDialog } from './shared';
import {
  selectedValues,
  selectedMemberLabel,
  providerMemberOptions,
  providerKeyMemberOptions,
} from './provider-groups-utils';

export function ProviderGroupDialog({
  kind,
  open,
  editing,
  form,
  providers,
  keysByProvider,
  submitting,
  onClose,
  onSubmit,
  onFormChange,
}: {
  kind: ProviderGroupKind;
  open: boolean;
  editing: boolean;
  form: ProviderGroupForm;
  providers: Pick<Provider, 'id' | 'name' | 'provider_type'>[];
  keysByProvider: Record<string, ProviderApiKey[]>;
  submitting: boolean;
  onClose: () => void;
  onSubmit: () => void;
  onFormChange: (form: ProviderGroupForm) => void;
}) {
  const { t } = useTranslate('admin');
  const titleKey = groupDialogTitleKey(kind, editing);

  return (
    <ManagementDialog
      open={open}
      title={t(titleKey)}
      submitting={submitting}
      submitDisabled={!form.name.trim()}
      onClose={onClose}
      onSubmit={onSubmit}
    >
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          required
          label={t('common.name')}
          value={form.name}
          onChange={(name) => onFormChange({ ...form, name })}
        />
        <TextFieldRow
          type="number"
          label={t('common.sortOrder')}
          value={form.sort_order}
          onChange={(sortOrder) => onFormChange({ ...form, sort_order: sortOrder })}
        />
      </Stack>
      <TextField
        fullWidth
        multiline
        minRows={3}
        label={t('common.description')}
        value={form.description}
        onChange={(event) => onFormChange({ ...form, description: event.target.value })}
      />
      <MemberSelect
        kind={kind}
        form={form}
        providers={providers}
        keysByProvider={keysByProvider}
        onFormChange={onFormChange}
      />
    </ManagementDialog>
  );
}

function MemberSelect({
  kind,
  form,
  providers,
  keysByProvider,
  onFormChange,
}: {
  kind: ProviderGroupKind;
  form: ProviderGroupForm;
  providers: Pick<Provider, 'id' | 'name' | 'provider_type'>[];
  keysByProvider: Record<string, ProviderApiKey[]>;
  onFormChange: (form: ProviderGroupForm) => void;
}) {
  const { t } = useTranslate('admin');
  const options = kind === 'provider'
    ? providerMemberOptions(providers)
    : providerKeyMemberOptions(providers, keysByProvider);
  const emptyText = t(kind === 'provider' ? 'providers.noProviders' : 'providers.noApiKeys');

  return (
    <TextField
      select
      fullWidth
      label={t(kind === 'provider' ? 'providers.providerGroupMembers' : 'providers.providerKeyGroupMembers')}
      value={form.member_ids}
      helperText={t(kind === 'provider' ? 'helper.providerGroupMembers' : 'helper.providerKeyGroupMembers')}
      SelectProps={{
        multiple: true,
        renderValue: (selected) =>
          selectedMemberLabel(selected as string[], options, t('providers.emptyGroupMembers'), (count) =>
            t('providers.selectedGroupMemberCount', { count })
          ),
      }}
      onChange={(event) => onFormChange({ ...form, member_ids: selectedValues(event.target.value) })}
    >
      {options.length === 0 ? <MenuItem disabled value="">{emptyText}</MenuItem> : null}
      {options.map((option) => (
        <MenuItem key={option.id} value={option.id}>
          <Checkbox checked={form.member_ids.includes(option.id)} />
          <ListItemText primary={option.label} secondary={option.secondary} />
        </MenuItem>
      ))}
    </TextField>
  );
}

function groupDialogTitleKey(kind: ProviderGroupKind, editing: boolean) {
  if (kind === 'provider') {
    return editing ? 'dialogs.editProviderGroup' : 'dialogs.createProviderGroup';
  }
  return editing ? 'dialogs.editProviderKeyGroup' : 'dialogs.createProviderKeyGroup';
}

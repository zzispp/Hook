'use client';

import type { Provider, ProviderApiKey } from 'src/types/provider';
import type { ProviderGroupKind, ProviderGroupForm } from './provider-groups-utils';

import Stack from '@mui/material/Stack';
import Checkbox from '@mui/material/Checkbox';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import ListItemText from '@mui/material/ListItemText';

import { useTranslate } from 'src/locales/use-locales';

import { TextFieldRow, ManagementDialog } from './shared';
import {
  formMemberIds,
  selectedValues,
  selectedMemberLabel,
  updateMemberPriority,
  updateSelectedMembers,
  providerMemberOptions,
  providerKeyMemberOptions,
  defaultProviderMemberPriority,
  defaultProviderKeyMemberPriority,
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
  providers: Pick<Provider, 'id' | 'name' | 'provider_type' | 'priority'>[];
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
      <GroupBasicFields form={form} onFormChange={onFormChange} />
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

function GroupBasicFields({
  form,
  onFormChange,
}: {
  form: ProviderGroupForm;
  onFormChange: (form: ProviderGroupForm) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <>
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
    </>
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
  providers: Pick<Provider, 'id' | 'name' | 'provider_type' | 'priority'>[];
  keysByProvider: Record<string, ProviderApiKey[]>;
  onFormChange: (form: ProviderGroupForm) => void;
}) {
  const { t } = useTranslate('admin');
  const options =
    kind === 'provider'
      ? providerMemberOptions(providers)
      : providerKeyMemberOptions(providers, keysByProvider);
  const emptyText = t(kind === 'provider' ? 'providers.noProviders' : 'providers.noApiKeys');
  const memberIds = formMemberIds(form);
  const defaultPriorityForId = (id: string) =>
    kind === 'provider'
      ? defaultProviderMemberPriority(providers, id)
      : defaultProviderKeyMemberPriority(keysByProvider, id);

  return (
    <Stack spacing={1.5}>
      <MemberSelectField
        kind={kind}
        memberIds={memberIds}
        options={options}
        emptyText={emptyText}
        onSelectedIdsChange={(selectedIds) =>
          onFormChange({
            ...form,
            members: updateSelectedMembers(form.members, selectedIds, defaultPriorityForId),
          })
        }
      />
      <MemberPriorityFields form={form} options={options} onFormChange={onFormChange} />
    </Stack>
  );
}

function MemberSelectField({
  kind,
  memberIds,
  options,
  emptyText,
  onSelectedIdsChange,
}: {
  kind: ProviderGroupKind;
  memberIds: string[];
  options: { id: string; label: string; secondary?: string }[];
  emptyText: string;
  onSelectedIdsChange: (ids: string[]) => void;
}) {
  const { t } = useTranslate('admin');
  const labelKey =
    kind === 'provider' ? 'providers.providerGroupMembers' : 'providers.providerKeyGroupMembers';
  const helperKey =
    kind === 'provider' ? 'helper.providerGroupMembers' : 'helper.providerKeyGroupMembers';

  return (
    <TextField
      select
      fullWidth
      label={t(labelKey)}
      value={memberIds}
      helperText={t(helperKey)}
      SelectProps={{
        multiple: true,
        renderValue: (selected) =>
          selectedMemberLabel(
            selected as string[],
            options,
            t('providers.emptyGroupMembers'),
            (count) => t('providers.selectedGroupMemberCount', { count })
          ),
      }}
      onChange={(event) => onSelectedIdsChange(selectedValues(event.target.value))}
    >
      {options.length === 0 ? <EmptyMemberOption text={emptyText} /> : null}
      {options.map((option) => (
        <MemberOptionItem key={option.id} option={option} checked={memberIds.includes(option.id)} />
      ))}
    </TextField>
  );
}

function EmptyMemberOption({ text }: { text: string }) {
  return (
    <MenuItem disabled value="">
      {text}
    </MenuItem>
  );
}

function MemberOptionItem({
  option,
  checked,
}: {
  option: { id: string; label: string; secondary?: string };
  checked: boolean;
}) {
  return (
    <MenuItem value={option.id}>
      <Checkbox checked={checked} />
      <ListItemText primary={option.label} secondary={option.secondary} />
    </MenuItem>
  );
}

function MemberPriorityFields({
  form,
  options,
  onFormChange,
}: {
  form: ProviderGroupForm;
  options: { id: string; label: string }[];
  onFormChange: (form: ProviderGroupForm) => void;
}) {
  const { t } = useTranslate('admin');
  if (form.members.length === 0) return null;

  const labels = new Map(options.map((option) => [option.id, option.label]));

  return (
    <Stack spacing={1}>
      {form.members.map((member) => (
        <Stack
          key={member.id}
          direction={{ xs: 'column', sm: 'row' }}
          alignItems={{ xs: 'stretch', sm: 'center' }}
          spacing={1}
        >
          <Typography variant="body2" sx={{ flex: 1, minWidth: 0 }}>
            {labels.get(member.id) ?? member.id}
          </Typography>
          <TextField
            size="small"
            type="number"
            label={t('providers.priority')}
            value={member.priority}
            onChange={(event) =>
              onFormChange({
                ...form,
                members: updateMemberPriority(form.members, member.id, event.target.value),
              })
            }
            sx={{ width: { xs: 1, sm: 160 } }}
          />
        </Stack>
      ))}
    </Stack>
  );
}

function groupDialogTitleKey(kind: ProviderGroupKind, editing: boolean) {
  if (kind === 'provider') {
    return editing ? 'dialogs.editProviderGroup' : 'dialogs.createProviderGroup';
  }
  return editing ? 'dialogs.editProviderKeyGroup' : 'dialogs.createProviderKeyGroup';
}

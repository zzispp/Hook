'use client';

import type { Provider, ProviderApiKey } from 'src/types/provider';
import type { ProviderKeyGroupForm } from './provider-key-groups-utils';

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
  providerKeyMemberOptions,
  defaultProviderKeyMemberPriority,
} from './provider-key-groups-utils';

export function ProviderKeyGroupDialog({
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
  open: boolean;
  editing: boolean;
  form: ProviderKeyGroupForm;
  providers: Pick<Provider, 'id' | 'name' | 'provider_type' | 'priority'>[];
  keysByProvider: Record<string, ProviderApiKey[]>;
  submitting: boolean;
  onClose: () => void;
  onSubmit: () => void;
  onFormChange: (form: ProviderKeyGroupForm) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <ManagementDialog
      open={open}
      title={t(editing ? 'dialogs.editProviderKeyGroup' : 'dialogs.createProviderKeyGroup')}
      submitting={submitting}
      submitDisabled={!form.name.trim()}
      onClose={onClose}
      onSubmit={onSubmit}
    >
      <KeyGroupBasicFields form={form} onFormChange={onFormChange} />
      <KeyMemberSelect
        form={form}
        providers={providers}
        keysByProvider={keysByProvider}
        onFormChange={onFormChange}
      />
    </ManagementDialog>
  );
}

function KeyGroupBasicFields({
  form,
  onFormChange,
}: {
  form: ProviderKeyGroupForm;
  onFormChange: (form: ProviderKeyGroupForm) => void;
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

function KeyMemberSelect({
  form,
  providers,
  keysByProvider,
  onFormChange,
}: {
  form: ProviderKeyGroupForm;
  providers: Pick<Provider, 'id' | 'name' | 'provider_type' | 'priority'>[];
  keysByProvider: Record<string, ProviderApiKey[]>;
  onFormChange: (form: ProviderKeyGroupForm) => void;
}) {
  const { t } = useTranslate('admin');
  const options = providerKeyMemberOptions(providers, keysByProvider);
  const memberIds = formMemberIds(form);
  const defaultPriorityForId = (id: string) => defaultProviderKeyMemberPriority(keysByProvider, id);

  return (
    <Stack spacing={1.5}>
      <KeyMemberSelectField
        memberIds={memberIds}
        options={options}
        onSelectedIdsChange={(selectedIds) =>
          onFormChange({
            ...form,
            members: updateSelectedMembers(form.members, selectedIds, defaultPriorityForId),
          })
        }
      />
      <MemberPriorityFields form={form} options={options} onFormChange={onFormChange} />
      {options.length === 0 ? (
        <Typography variant="caption" color="text.secondary">
          {t('providers.noApiKeys')}
        </Typography>
      ) : null}
    </Stack>
  );
}

function KeyMemberSelectField({
  memberIds,
  options,
  onSelectedIdsChange,
}: {
  memberIds: string[];
  options: { id: string; label: string; secondary?: string }[];
  onSelectedIdsChange: (ids: string[]) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TextField
      select
      fullWidth
      label={t('providers.providerKeyGroupMembers')}
      value={memberIds}
      helperText={t('helper.providerKeyGroupMembers')}
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
      {options.length === 0 ? <EmptyMemberOption text={t('providers.noApiKeys')} /> : null}
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
  form: ProviderKeyGroupForm;
  options: { id: string; label: string }[];
  onFormChange: (form: ProviderKeyGroupForm) => void;
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

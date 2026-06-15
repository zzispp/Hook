'use client';

import type { UserGroup } from 'src/types/user-group';
import type { RoutingProfile } from 'src/types/routing';
import type { GlobalModelResponse } from 'src/types/model';
import type { ProviderKeyGroup } from 'src/types/provider-key-group';
import type { useGroupDialog } from './billing-group-management-state';
import type { BillingAccessMode } from './billing-group-management-utils';

import Checkbox from '@mui/material/Checkbox';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import ToggleButton from '@mui/material/ToggleButton';
import ListItemText from '@mui/material/ListItemText';
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup';

import { useTranslate } from 'src/locales/use-locales';
import { useRoutingProfiles } from 'src/actions/routing';

import { SwitchRow, ManagementDialog } from './shared';
import { userGroupSelectionLabel } from './user-group-utils';
import { BillingGroupBasicFields } from './billing-group-basic-fields';

export function BillingGroupDialog({
  dialog,
  models,
  providerKeyGroups,
  userGroups,
}: {
  dialog: ReturnType<typeof useGroupDialog>;
  models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[];
  providerKeyGroups: ProviderKeyGroup[];
  userGroups: UserGroup[];
}) {
  const { t } = useTranslate('admin');
  const profiles = useRoutingProfiles();

  return (
    <ManagementDialog
      open={dialog.open}
      title={dialog.editing ? t('dialogs.editBillingGroup') : t('dialogs.createBillingGroup')}
      submitting={dialog.submitting}
      onClose={dialog.closeDialog}
      onSubmit={dialog.submit}
    >
      <BillingGroupBasicFields dialog={dialog} />
      <ModelSelect dialog={dialog} models={models} />
      <AccessModeControl dialog={dialog} />
      <AccessGroupSelect dialog={dialog} providerKeyGroups={providerKeyGroups} />
      <RoutingProfileSelect dialog={dialog} profiles={profiles.items} />
      <UserGroupSelect dialog={dialog} userGroups={userGroups} />
      <SwitchRow
        checked={dialog.form.is_active}
        label={t('common.enabled')}
        onChange={(checked) => dialog.setForm((form) => ({ ...form, is_active: checked }))}
      />
    </ManagementDialog>
  );
}

function RoutingProfileSelect({
  dialog,
  profiles,
}: {
  dialog: ReturnType<typeof useGroupDialog>;
  profiles: RoutingProfile[];
}) {
  const { t } = useTranslate('admin');

  return (
    <TextField
      select
      fullWidth
      label={t('fields.routingProfile')}
      value={dialog.form.routing_profile_id}
      helperText={t('helper.billingRoutingProfile')}
      onChange={(event) =>
        dialog.setForm((form) => ({
          ...form,
          routing_profile_id: event.target.value as typeof form.routing_profile_id,
        }))
      }
    >
      <MenuItem value="">{t('routing.profileInherited')}</MenuItem>
      {profiles.map((profile) => (
        <MenuItem key={profile.id} value={profile.id}>
          {profile.name}
        </MenuItem>
      ))}
    </TextField>
  );
}

function AccessModeControl({ dialog }: { dialog: ReturnType<typeof useGroupDialog> }) {
  const { t } = useTranslate('admin');

  return (
    <ToggleButtonGroup
      exclusive
      fullWidth
      value={dialog.form.access_mode}
      onChange={(_event, accessMode) => accessMode && setAccessMode(dialog, accessMode)}
    >
      <ToggleButton value="unrestricted">{t('billingGroups.accessModeUnrestricted')}</ToggleButton>
      <ToggleButton value="provider_key_groups">{t('billingGroups.accessModeProviderKeyGroups')}</ToggleButton>
    </ToggleButtonGroup>
  );
}

function AccessGroupSelect({
  dialog,
  providerKeyGroups,
}: {
  dialog: ReturnType<typeof useGroupDialog>;
  providerKeyGroups: ProviderKeyGroup[];
}) {
  if (dialog.form.access_mode === 'provider_key_groups') {
    return (
      <GroupSelect
        labelKey="fields.allowedProviderKeyGroups"
        helperKey="helper.billingProviderKeyGroupAccess"
        ids={dialog.form.allowed_provider_key_group_ids}
        groups={providerKeyGroups}
        onChange={(ids) => dialog.setForm((form) => ({ ...form, allowed_provider_key_group_ids: ids }))}
      />
    );
  }
  return null;
}

function GroupSelect({
  labelKey,
  helperKey,
  ids,
  groups,
  onChange,
}: {
  labelKey: string;
  helperKey: string;
  ids: string[];
  groups: ProviderKeyGroup[];
  onChange: (ids: string[]) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TextField
      select
      fullWidth
      label={t(labelKey)}
      value={ids}
      helperText={t(helperKey)}
      SelectProps={{
        multiple: true,
        renderValue: (selected) => groupSelectionLabel(selected as string[], groups, t),
      }}
      onChange={(event) => onChange(selectedValues(event.target.value))}
    >
      {groups.length === 0 ? <MenuItem disabled value="">{t('common.noData')}</MenuItem> : null}
      {groups.map((group) => (
        <MenuItem key={group.id} value={group.id}>
          <Checkbox checked={ids.includes(group.id)} />
          <ListItemText primary={group.name} secondary={group.description || undefined} />
        </MenuItem>
      ))}
    </TextField>
  );
}

function UserGroupSelect({
  dialog,
  userGroups,
}: {
  dialog: ReturnType<typeof useGroupDialog>;
  userGroups: UserGroup[];
}) {
  const { t } = useTranslate('admin');

  return (
    <TextField
      select
      fullWidth
      label={t('fields.visibleUserGroups')}
      value={dialog.form.visible_user_group_codes}
      helperText={t('helper.groupUserGroupVisibility')}
      SelectProps={{
        multiple: true,
        renderValue: (selected) => userGroupSelectionLabel(selected as string[], userGroups, t),
      }}
      onChange={(event) =>
        dialog.setForm((form) => ({
          ...form,
          visible_user_group_codes: selectedValues(event.target.value),
        }))
      }
    >
      {userGroups.length === 0 ? <MenuItem disabled value="">{t('userGroups.noActiveGroups')}</MenuItem> : null}
      {userGroups.map((group) => (
        <MenuItem key={group.code} value={group.code}>
          <Checkbox checked={dialog.form.visible_user_group_codes.includes(group.code)} />
          <ListItemText primary={group.name} secondary={group.code} />
        </MenuItem>
      ))}
    </TextField>
  );
}

function ModelSelect({
  dialog,
  models,
}: {
  dialog: ReturnType<typeof useGroupDialog>;
  models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[];
}) {
  const { t } = useTranslate('admin');

  return (
    <TextField
      select
      fullWidth
      label={t('fields.allowedModels')}
      value={dialog.form.allowed_model_ids}
      helperText={t('helper.groupModelAccess')}
      SelectProps={{
        multiple: true,
        renderValue: (selected) => modelSelectionLabel(selected as string[], models, t),
      }}
      onChange={(event) => dialog.setForm((form) => ({ ...form, allowed_model_ids: selectedValues(event.target.value) }))}
    >
      {models.length === 0 ? <MenuItem disabled value="">{t('tokens.noModels')}</MenuItem> : null}
      {models.map((model) => (
        <MenuItem key={model.id} value={model.id}>
          <Checkbox checked={dialog.form.allowed_model_ids.includes(model.id)} />
          <ListItemText primary={model.display_name || model.name} secondary={model.name} />
        </MenuItem>
      ))}
    </TextField>
  );
}

function setAccessMode(dialog: ReturnType<typeof useGroupDialog>, accessMode: BillingAccessMode) {
  dialog.setForm((form) => ({
    ...form,
    access_mode: accessMode,
    allowed_provider_key_group_ids: accessMode === 'provider_key_groups' ? form.allowed_provider_key_group_ids : [],
  }));
}

function selectedValues(value: string | string[]) {
  return Array.isArray(value) ? value : value.split(',').filter(Boolean);
}

function modelSelectionLabel(
  ids: string[],
  models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[],
  t: (key: string, options?: Record<string, unknown>) => string
) {
  if (ids.length === 0) return t('billingGroups.allModels');
  if (ids.length > 2) return t('billingGroups.selectedModelCount', { count: ids.length });
  const labels = new Map(models.map((model) => [model.id, model.display_name || model.name]));
  return ids.map((id) => labels.get(id) ?? id).join(', ');
}

function groupSelectionLabel(
  ids: string[],
  groups: ProviderKeyGroup[],
  t: (key: string, options?: Record<string, unknown>) => string
) {
  if (ids.length === 0) return t('billingGroups.noGroupSelected');
  if (ids.length > 2) return t('billingGroups.selectedGroupCount', { count: ids.length });
  const labels = new Map(groups.map((group) => [group.id, group.name]));
  return ids.map((id) => labels.get(id) ?? id).join(', ');
}

'use client';

import type { AdminT } from './shared';
import type { Provider } from 'src/types/provider';
import type { UserGroup } from 'src/types/user-group';
import type { UserForm } from './user-management-utils';
import type { GlobalModelResponse } from 'src/types/model';
import type { Role, SystemUser, UserQuotaMode } from 'src/types/rbac';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Checkbox from '@mui/material/Checkbox';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import Autocomplete from '@mui/material/Autocomplete';
import ListItemText from '@mui/material/ListItemText';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';

import { providerTypeLabel } from './provider-management-utils';
import { SwitchRow, TextFieldRow, ManagementDialog } from './shared';
import { providerColor, providerLabel } from '../profile/provider-utils';

type UserDialogState = {
  open: boolean;
  creating: boolean;
  editing: SystemUser | null;
  form: UserForm;
  submitting: boolean;
  close: VoidFunction;
  submit: VoidFunction;
  unlinkIdentity: (identityId: string) => void;
  unlinkingIdentityId: string | null;
  setForm: React.Dispatch<React.SetStateAction<UserForm>>;
};

type Props = {
  dialog: UserDialogState;
  roles: Role[];
  userGroups: UserGroup[];
  models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[];
  providers: Pick<Provider, 'id' | 'name' | 'provider_type'>[];
};

export function UserFormDialog({ dialog, roles, userGroups, models, providers }: Props) {
  const { t } = useTranslate('admin');

  return (
    <ManagementDialog
      open={dialog.open}
      title={dialog.editing ? t('dialogs.editUser') : t('dialogs.createUser')}
      submitting={dialog.submitting}
      onClose={dialog.close}
      onSubmit={dialog.submit}
    >
      <IdentityFields dialog={dialog} roles={roles} userGroups={userGroups} />
      <IdentityProviderDetails dialog={dialog} />
      <AccessFields dialog={dialog} />
      <RestrictionFields dialog={dialog} models={models} providers={providers} />
      <SwitchRow
        label={t('common.active')}
        checked={dialog.form.is_active}
        onChange={(isActive) => dialog.setForm((form) => ({ ...form, is_active: isActive }))}
      />
    </ManagementDialog>
  );
}

function IdentityProviderDetails({ dialog }: { dialog: UserDialogState }) {
  const { t } = useTranslate('admin');
  const identities = dialog.editing?.identities ?? [];

  if (!dialog.editing) {
    return null;
  }

  return (
    <Stack spacing={1}>
      <Typography variant="subtitle2">{t('users.providers')}</Typography>
      {identities.length === 0 ? (
        <Typography variant="body2" color="text.secondary">
          {t('common.none')}
        </Typography>
      ) : (
        identities.map((identity) => (
          <Stack
            key={identity.id}
            direction={{ xs: 'column', sm: 'row' }}
            spacing={1}
            alignItems={{ sm: 'center' }}
          >
            <Label color={providerColor(identity.provider)} variant="soft">
              {providerLabel(identity.provider)}
            </Label>
            <Stack sx={{ minWidth: 0, flex: 1 }}>
              <Typography variant="body2" noWrap>
                {identity.display_name || identity.email || identity.provider_subject}
              </Typography>
              <Typography variant="caption" color="text.secondary" noWrap>
                {identity.provider_subject}
              </Typography>
            </Stack>
            <Button
              color="error"
              variant="text"
              loading={dialog.unlinkingIdentityId === identity.id}
              onClick={() => dialog.unlinkIdentity(identity.id)}
            >
              {t('users.unlinkProvider')}
            </Button>
          </Stack>
        ))
      )}
    </Stack>
  );
}

function IdentityFields({ dialog, roles, userGroups }: Pick<Props, 'dialog' | 'roles' | 'userGroups'>) {
  const { t } = useTranslate('admin');

  return (
    <>
      <TextFieldRow
        required
        label={t('common.username')}
        value={dialog.form.username}
        onChange={(value) => dialog.setForm((form) => ({ ...form, username: value }))}
      />
      <TextFieldRow
        required
        label={t('common.email')}
        value={dialog.form.email}
        onChange={(value) => dialog.setForm((form) => ({ ...form, email: value }))}
      />
      <TextFieldRow
        required
        select
        label={t('common.role')}
        value={dialog.form.role}
        onChange={(value) => dialog.setForm((form) => ({ ...form, role: value }))}
      >
        {roles.map((role) => (
          <MenuItem key={role.code} value={role.code}>
            {role.name} ({role.code})
          </MenuItem>
        ))}
      </TextFieldRow>
      <TextFieldRow
        required
        select
        label={t('fields.userGroup')}
        value={dialog.form.group_code}
        onChange={(value) => dialog.setForm((form) => ({ ...form, group_code: value }))}
      >
        {userGroups.map((group) => (
          <MenuItem key={group.code} value={group.code}>
            {group.name} ({group.code})
          </MenuItem>
        ))}
      </TextFieldRow>
      <TextFieldRow
        required
        type="password"
        label={dialog.editing ? t('fields.newPassword') : t('common.password')}
        value={dialog.form.password}
        helperText={dialog.editing ? t('helper.updatePasswordRequired') : undefined}
        onChange={(value) => dialog.setForm((form) => ({ ...form, password: value }))}
      />
    </>
  );
}

function RestrictionFields({
  dialog,
  models,
  providers,
}: {
  dialog: UserDialogState;
  models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[];
  providers: Pick<Provider, 'id' | 'name' | 'provider_type'>[];
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={2}>
      <AccessSearchSelect
        label={t('fields.allowedProviders')}
        helperText={t('helper.userProviderAccess')}
        noOptionsText={providers.length === 0 ? t('providers.noProviders') : t('common.noResults')}
        options={providerOptions(providers, t)}
        value={dialog.form.allowed_provider_ids}
        onChange={(allowedProviderIds) =>
          dialog.setForm((form) => ({ ...form, allowed_provider_ids: allowedProviderIds }))
        }
      />
      <AccessSearchSelect
        label={t('fields.allowedModels')}
        helperText={t('helper.userModelAccess')}
        noOptionsText={models.length === 0 ? t('tokens.noModels') : t('common.noResults')}
        options={modelOptions(models)}
        value={dialog.form.allowed_model_ids}
        onChange={(allowedModelIds) =>
          dialog.setForm((form) => ({ ...form, allowed_model_ids: allowedModelIds }))
        }
      />
    </Stack>
  );
}

type AccessOption = {
  value: string;
  label: string;
  description?: string;
};

function AccessSearchSelect({
  label,
  value,
  options,
  helperText,
  noOptionsText,
  onChange,
}: {
  label: string;
  value: string[];
  options: AccessOption[];
  helperText: string;
  noOptionsText: string;
  onChange: (value: string[]) => void;
}) {
  const selected = selectedOptions(value, options);

  return (
    <Autocomplete
      multiple
      fullWidth
      disableCloseOnSelect
      options={options}
      value={selected}
      getOptionLabel={(option) => option.label}
      isOptionEqualToValue={(option, current) => option.value === current.value}
      noOptionsText={noOptionsText}
      onChange={(_event, next) => onChange(next.map((option) => option.value))}
      renderInput={(params) => <TextField {...params} label={label} helperText={helperText} />}
      renderOption={(props, option, { selected: checked }) => (
        <MenuItem {...props} key={option.value} value={option.value}>
          <Checkbox checked={checked} />
          <ListItemText primary={option.label} secondary={option.description} />
        </MenuItem>
      )}
    />
  );
}

function selectedOptions(value: string[], options: AccessOption[]) {
  return value.map(
    (id) => options.find((option) => option.value === id) ?? { value: id, label: id }
  );
}

function providerOptions(providers: Pick<Provider, 'id' | 'name' | 'provider_type'>[], t: AdminT) {
  return providers.map((provider) => ({
    value: provider.id,
    label: provider.name,
    description: providerTypeLabel(provider.provider_type, t),
  }));
}

function modelOptions(models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[]) {
  return models.map((model) => ({
    value: model.id,
    label: model.display_name || model.name,
    description: model.name,
  }));
}

function AccessFields({ dialog }: { dialog: UserDialogState }) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
      <TextFieldRow
        type="number"
        label={t('fields.rateLimitRpm')}
        value={dialog.form.rate_limit_rpm}
        helperText={t('helper.userRateLimit')}
        slotProps={{ htmlInput: { min: 0, step: 1 } }}
        onChange={(value) => dialog.setForm((form) => ({ ...form, rate_limit_rpm: value }))}
      />
      <TextFieldRow
        select
        label={t('fields.quotaMode')}
        value={dialog.form.quota_mode}
        helperText={t('helper.userQuotaMode')}
        onChange={(value) =>
          dialog.setForm((form) => ({ ...form, quota_mode: value as UserQuotaMode }))
        }
      >
        <MenuItem value="wallet">{t('users.walletLimited')}</MenuItem>
        <MenuItem value="unlimited">{t('users.unlimited')}</MenuItem>
      </TextFieldRow>
    </Stack>
  );
}

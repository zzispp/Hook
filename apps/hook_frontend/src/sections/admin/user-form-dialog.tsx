'use client';

import type { UserForm } from './user-management-utils';
import type { Role, SystemUser, UserQuotaMode } from 'src/types/rbac';

import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow, ManagementDialog } from './shared';

type UserDialogState = {
  open: boolean;
  creating: boolean;
  editing: SystemUser | null;
  form: UserForm;
  submitting: boolean;
  close: VoidFunction;
  submit: VoidFunction;
  setForm: React.Dispatch<React.SetStateAction<UserForm>>;
};

type Props = {
  dialog: UserDialogState;
  roles: Role[];
};

export function UserFormDialog({ dialog, roles }: Props) {
  const { t } = useTranslate('admin');

  return (
    <ManagementDialog
      open={dialog.open}
      title={dialog.editing ? t('dialogs.editUser') : t('dialogs.createUser')}
      submitting={dialog.submitting}
      onClose={dialog.close}
      onSubmit={dialog.submit}
    >
      <IdentityFields dialog={dialog} roles={roles} />
      <AccessFields dialog={dialog} />
      <SwitchRow
        label={t('common.active')}
        checked={dialog.form.is_active}
        onChange={(isActive) => dialog.setForm((form) => ({ ...form, is_active: isActive }))}
      />
    </ManagementDialog>
  );
}

function IdentityFields({ dialog, roles }: Props) {
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
        type="password"
        label={dialog.editing ? t('fields.newPassword') : t('common.password')}
        value={dialog.form.password}
        helperText={dialog.editing ? t('helper.updatePasswordRequired') : undefined}
        onChange={(value) => dialog.setForm((form) => ({ ...form, password: value }))}
      />
    </>
  );
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
        onChange={(value) => dialog.setForm((form) => ({ ...form, quota_mode: value as UserQuotaMode }))}
      >
        <MenuItem value="wallet">{t('users.walletLimited')}</MenuItem>
        <MenuItem value="unlimited">{t('users.unlimited')}</MenuItem>
      </TextFieldRow>
    </Stack>
  );
}

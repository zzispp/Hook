'use client';

import type { useUserGroupDialog } from './user-group-management-state';

import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/locales/use-locales';

import { isDefaultUserGroup } from './user-group-management-utils';
import { SwitchRow, TextFieldRow, ManagementDialog } from './shared';

type Props = {
  dialog: ReturnType<typeof useUserGroupDialog>;
};

export function UserGroupDialog({ dialog }: Props) {
  const { t } = useTranslate('admin');
  const isDefault = isDefaultUserGroup(dialog.editing);

  return (
    <ManagementDialog
      open={dialog.open}
      title={dialog.editing ? t('dialogs.editUserGroup') : t('dialogs.createUserGroup')}
      submitting={dialog.submitting}
      onClose={dialog.close}
      onSubmit={dialog.submit}
    >
      <TextFieldRow
        required
        disabled={!!dialog.editing}
        label={t('common.code')}
        value={dialog.form.code}
        onChange={(value) => dialog.setForm((form) => ({ ...form, code: value }))}
      />
      <TextFieldRow
        required
        label={t('common.name')}
        value={dialog.form.name}
        onChange={(value) => dialog.setForm((form) => ({ ...form, name: value }))}
      />
      <TextField
        fullWidth
        multiline
        minRows={3}
        label={t('common.description')}
        value={dialog.form.description}
        onChange={(event) =>
          dialog.setForm((form) => ({ ...form, description: event.target.value }))
        }
      />
      <TextFieldRow
        type="number"
        label={t('common.sortOrder')}
        value={dialog.form.sort_order}
        onChange={(value) => dialog.setForm((form) => ({ ...form, sort_order: value }))}
      />
      <SwitchRow
        checked={dialog.form.is_active}
        disabled={isDefault}
        label={t('common.enabled')}
        helperText={isDefault ? t('userGroups.defaultCannotDisable') : undefined}
        onChange={(checked) => dialog.setForm((form) => ({ ...form, is_active: checked }))}
      />
    </ManagementDialog>
  );
}

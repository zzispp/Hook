'use client';

import type { ModelAccessMode } from 'src/types/api-token';
import type {
  TokenScope,
  UserOption,
  TokenDialogState,
  TokenModelOption,
  BillingGroupOption,
} from './api-token-management-types';

import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { accountingCurrencyLabel } from 'src/utils/money-boundary';

import { useTranslate } from 'src/locales/use-locales';

import { TextFieldRow, ManagementDialog } from '../admin/shared';

export function ApiTokenDialog({
  scope,
  dialog,
  groups,
  models,
  users = [],
  fixedUserId,
}: {
  scope: TokenScope;
  dialog: TokenDialogState;
  groups: BillingGroupOption[];
  models: TokenModelOption[];
  users?: UserOption[];
  fixedUserId?: string;
}) {
  const { t } = useTranslate('admin');
  const creating = !dialog.editing;

  return (
    <ManagementDialog
      open={dialog.open}
      title={dialog.editing ? t('dialogs.editApiToken') : t('dialogs.createApiToken')}
      submitting={dialog.submitting}
      onClose={dialog.closeDialog}
      onSubmit={dialog.submit}
    >
      <TextFieldRow
        required
        label={t('common.name')}
        value={dialog.form.name}
        onChange={(value) => dialog.setForm((form) => ({ ...form, name: value }))}
      />
      {creating && scope === 'admin' && !fixedUserId ? <AdminOwnerFields dialog={dialog} users={users} /> : null}
      {creating ? <CreateOnlyFields dialog={dialog} groups={groups} /> : null}
      <LimitFields dialog={dialog} />
      <ModelFields dialog={dialog} models={models} />
    </ManagementDialog>
  );
}

function AdminOwnerFields({ dialog, users }: { dialog: TokenDialogState; users: UserOption[] }) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
      <TextFieldRow
        select
        label={t('fields.tokenType')}
        value={dialog.form.token_type}
        onChange={(value) => {
          dialog.clearError('user_id');
          dialog.setForm((form) => ({ ...form, token_type: value as 'user' | 'independent' }));
        }}
      >
        <MenuItem value="independent">{t('tokens.independentToken')}</MenuItem>
        <MenuItem value="user">{t('tokens.userToken')}</MenuItem>
      </TextFieldRow>
      {dialog.form.token_type === 'user' ? <UserSelect dialog={dialog} users={users} /> : null}
    </Stack>
  );
}

function UserSelect({ dialog, users }: { dialog: TokenDialogState; users: UserOption[] }) {
  const { t } = useTranslate('admin');

  return (
    <TextFieldRow
      select
      required
      label={t('common.user')}
      value={dialog.form.user_id}
      error={Boolean(dialog.errors.user_id)}
      helperText={dialog.errors.user_id ? t(dialog.errors.user_id) : undefined}
      onChange={(value) => {
        dialog.clearError('user_id');
        dialog.setForm((form) => ({ ...form, user_id: value }));
      }}
    >
      {users.length === 0 ? (
        <MenuItem disabled value="">
          {t('tokens.noUsers')}
        </MenuItem>
      ) : null}
      {users.map((user) => (
        <MenuItem key={user.id} value={user.id}>
          {user.username} ({user.email})
        </MenuItem>
      ))}
    </TextFieldRow>
  );
}

function CreateOnlyFields({ dialog, groups }: { dialog: TokenDialogState; groups: BillingGroupOption[] }) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
      <TextFieldRow
        select
        required
        label={t('common.group')}
        value={dialog.form.group_code}
        onChange={(value) => dialog.setForm((form) => groupForm(form, value, groups))}
      >
        {groups.length === 0 ? (
          <MenuItem disabled value="">
            {t('tokens.noGroups')}
          </MenuItem>
        ) : null}
        {groups.map((group) => (
          <MenuItem key={group.code} value={group.code}>
            {group.name}
          </MenuItem>
        ))}
      </TextFieldRow>
      <TextFieldRow
        type="datetime-local"
        label={t('fields.expiresAt')}
        value={dialog.form.expires_at}
        helperText={t('helper.unlimitedExpiresAt')}
        slotProps={{ inputLabel: { shrink: true } }}
        onChange={(value) => dialog.setForm((form) => ({ ...form, expires_at: value }))}
      />
    </Stack>
  );
}

function groupForm(
  form: TokenDialogState['form'],
  groupCode: string,
  groups: BillingGroupOption[]
) {
  const allowedModelIds = groups.find((group) => group.code === groupCode)?.allowed_model_ids ?? [];
  return {
    ...form,
    group_code: groupCode,
    allowed_model_ids: filterModelIds(form.allowed_model_ids, allowedModelIds),
  };
}

function filterModelIds(selectedIds: string[], allowedIds: string[]) {
  return allowedIds.length === 0 ? selectedIds : selectedIds.filter((id) => allowedIds.includes(id));
}

function LimitFields({ dialog }: { dialog: TokenDialogState }) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
      <TextFieldRow
        type="number"
        label={t('fields.rateLimitRpm')}
        value={dialog.form.rate_limit_rpm}
        helperText={t('helper.systemRateLimit')}
        onChange={(value) => dialog.setForm((form) => ({ ...form, rate_limit_rpm: value }))}
      />
      <TextFieldRow
        type="number"
        label={accountingCurrencyLabel(t('fields.quotaLimit'))}
        value={dialog.form.quota_limit}
        helperText={t('helper.unlimitedQuota')}
        onChange={(value) => dialog.setForm((form) => ({ ...form, quota_limit: value }))}
      />
    </Stack>
  );
}

function ModelFields({
  dialog,
  models,
}: {
  dialog: TokenDialogState;
  models: TokenModelOption[];
}) {
  const { t } = useTranslate('admin');

  return (
    <>
      <TextFieldRow
        select
        label={t('fields.modelAccessMode')}
        value={dialog.form.model_access_mode}
        onChange={(value) => dialog.setForm((form) => ({ ...form, model_access_mode: value as ModelAccessMode }))}
      >
        <MenuItem value="all">{t('tokens.allModels')}</MenuItem>
        <MenuItem value="limited">{t('tokens.limitedModels')}</MenuItem>
      </TextFieldRow>
      {dialog.form.model_access_mode === 'limited' ? <ModelSelect dialog={dialog} models={models} /> : null}
    </>
  );
}

function ModelSelect({ dialog, models }: { dialog: TokenDialogState; models: TokenModelOption[] }) {
  const { t } = useTranslate('admin');

  return (
    <TextField
      select
      fullWidth
      label={t('fields.allowedModels')}
      value={dialog.form.allowed_model_ids}
      SelectProps={{ multiple: true }}
      onChange={(event) => dialog.setForm((form) => ({ ...form, allowed_model_ids: selectedModelIds(event.target.value) }))}
    >
      {models.length === 0 ? (
        <MenuItem disabled value="">
          {t('tokens.noModels')}
        </MenuItem>
      ) : null}
      {models.map((model) => (
        <MenuItem key={model.id} value={model.id}>
          {model.display_name || model.name}
        </MenuItem>
      ))}
    </TextField>
  );
}

function selectedModelIds(value: string | string[]) {
  return Array.isArray(value) ? value : value.split(',').filter(Boolean);
}

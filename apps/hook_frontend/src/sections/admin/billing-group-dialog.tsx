'use client';

import type { GlobalModelResponse } from 'src/types/model';
import type { useGroupDialog } from './billing-group-management-state';

import Stack from '@mui/material/Stack';
import Checkbox from '@mui/material/Checkbox';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import ListItemText from '@mui/material/ListItemText';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow, ManagementDialog } from './shared';

export function BillingGroupDialog({
  dialog,
  models,
}: {
  dialog: ReturnType<typeof useGroupDialog>;
  models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[];
}) {
  const { t } = useTranslate('admin');

  return (
    <ManagementDialog
      open={dialog.open}
      title={dialog.editing ? t('dialogs.editBillingGroup') : t('dialogs.createBillingGroup')}
      submitting={dialog.submitting}
      onClose={dialog.closeDialog}
      onSubmit={dialog.submit}
    >
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
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
      </Stack>
      <TextField
        fullWidth
        multiline
        minRows={3}
        label={t('common.description')}
        value={dialog.form.description}
        onChange={(event) => dialog.setForm((form) => ({ ...form, description: event.target.value }))}
      />
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          required
          type="number"
          label={t('fields.billingMultiplier')}
          value={dialog.form.billing_multiplier}
          onChange={(value) => dialog.setForm((form) => ({ ...form, billing_multiplier: value }))}
        />
        <TextFieldRow
          type="number"
          label={t('common.sortOrder')}
          value={dialog.form.sort_order}
          onChange={(value) => dialog.setForm((form) => ({ ...form, sort_order: value }))}
        />
      </Stack>
      <ModelSelect dialog={dialog} models={models} />
      <SwitchRow
        checked={dialog.form.is_active}
        label={t('common.enabled')}
        onChange={(checked) => dialog.setForm((form) => ({ ...form, is_active: checked }))}
      />
    </ManagementDialog>
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
      {models.length === 0 ? (
        <MenuItem disabled value="">
          {t('tokens.noModels')}
        </MenuItem>
      ) : null}
      {models.map((model) => (
        <MenuItem key={model.id} value={model.id}>
          <Checkbox checked={dialog.form.allowed_model_ids.includes(model.id)} />
          <ListItemText primary={model.display_name || model.name} secondary={model.name} />
        </MenuItem>
      ))}
    </TextField>
  );
}

function selectedValues(value: string | string[]) {
  return Array.isArray(value) ? value : value.split(',').filter(Boolean);
}

function modelSelectionLabel(
  ids: string[],
  models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[],
  t: (key: string, options?: Record<string, unknown>) => string
) {
  if (ids.length === 0) {
    return t('billingGroups.allModels');
  }
  if (ids.length > 2) {
    return t('billingGroups.selectedModelCount', { count: ids.length });
  }
  return ids.map((id) => modelLabel(id, models)).join(', ');
}

function modelLabel(id: string, models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[]) {
  const model = models.find((item) => item.id === id);
  return model?.display_name || model?.name || id;
}

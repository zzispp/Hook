'use client';

import type { useGroupDialog } from './billing-group-management-state';

import Stack from '@mui/material/Stack';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/locales/use-locales';

import { TextFieldRow } from './shared';

export function BillingGroupBasicFields({ dialog }: { dialog: ReturnType<typeof useGroupDialog> }) {
  const { t } = useTranslate('admin');

  return (
    <>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          required
          disabled={!!dialog.editing}
          label={t('common.code')}
          value={dialog.form.code}
          onChange={(code) => dialog.setForm((form) => ({ ...form, code }))}
        />
        <TextFieldRow
          required
          label={t('common.name')}
          value={dialog.form.name}
          onChange={(name) => dialog.setForm((form) => ({ ...form, name }))}
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
    </>
  );
}

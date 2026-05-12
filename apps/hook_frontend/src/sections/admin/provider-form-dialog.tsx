'use client';

import type { useProviderDialog } from './provider-management-state';

import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow, ManagementDialog } from './shared';
import {
  providerTypeLabel,
  PROVIDER_TYPE_OPTIONS,
  DEFAULT_PROVIDER_MAX_RETRIES,
} from './provider-management-utils';

type ProviderDialogState = ReturnType<typeof useProviderDialog>;

export function ProviderFormDialog({ dialog }: { dialog: ProviderDialogState }) {
  const { t } = useTranslate('admin');

  return (
    <ManagementDialog
      open={dialog.open}
      title={dialog.editing ? t('dialogs.editProvider') : t('dialogs.createProvider')}
      submitting={dialog.submitting}
      onClose={dialog.closeDialog}
      onSubmit={dialog.submit}
    >
      <ProviderIdentityFields dialog={dialog} />
      <ProviderRequestConfigFields dialog={dialog} />
      <ProviderPriorityField dialog={dialog} />
      <ProviderSwitchFields dialog={dialog} />
    </ManagementDialog>
  );
}

function ProviderIdentityFields({ dialog }: { dialog: ProviderDialogState }) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
      <TextFieldRow
        required
        label={t('providers.name')}
        value={dialog.form.name}
        onChange={(value) => dialog.setForm((form) => ({ ...form, name: value }))}
      />
      <TextFieldRow
        select
        required
        label={t('common.type')}
        value={dialog.form.provider_type}
        onChange={(value) => dialog.setForm((form) => ({ ...form, provider_type: value as typeof form.provider_type }))}
      >
        {PROVIDER_TYPE_OPTIONS.map((providerType) => (
          <MenuItem key={providerType} value={providerType}>
            {providerTypeLabel(providerType, t)}
          </MenuItem>
        ))}
      </TextFieldRow>
    </Stack>
  );
}

function ProviderRequestConfigFields({ dialog }: { dialog: ProviderDialogState }) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
      <TextFieldRow
        type="number"
        label={t('providers.maxRetries')}
        value={dialog.form.max_retries}
        placeholder={String(DEFAULT_PROVIDER_MAX_RETRIES)}
        helperText={t('providers.defaultWhenBlank')}
        onChange={(value) => dialog.setForm((form) => ({ ...form, max_retries: value }))}
      />
    </Stack>
  );
}

function ProviderPriorityField({ dialog }: { dialog: ProviderDialogState }) {
  const { t } = useTranslate('admin');

  return (
    <TextFieldRow
      type="number"
      label={t('providers.priority')}
      value={dialog.form.priority}
      onChange={(value) => dialog.setForm((form) => ({ ...form, priority: value }))}
    />
  );
}

function ProviderSwitchFields({ dialog }: { dialog: ProviderDialogState }) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
      <SwitchRow
        checked={dialog.form.enable_format_conversion}
        label={t('providers.enableFormatConversion')}
        onChange={(checked) => dialog.setForm((form) => ({ ...form, enable_format_conversion: checked }))}
      />
      <SwitchRow
        checked={dialog.form.keep_priority_on_conversion}
        label={t('providers.keepPriorityOnConversion')}
        onChange={(checked) => dialog.setForm((form) => ({ ...form, keep_priority_on_conversion: checked }))}
      />
      <SwitchRow
        checked={dialog.form.is_active}
        label={t('common.enabled')}
        onChange={(checked) => dialog.setForm((form) => ({ ...form, is_active: checked }))}
      />
    </Stack>
  );
}

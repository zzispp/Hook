'use client';

import type { ProviderGroup } from 'src/types/provider-group';
import type { useProviderDialog } from './provider-management-state';

import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow, ManagementDialog } from './shared';
import {
  providerTypeLabel,
  PROVIDER_TYPE_OPTIONS,
  DEFAULT_PROVIDER_MAX_RETRIES,
  DEFAULT_PROVIDER_REQUEST_TIMEOUT_SECONDS,
  DEFAULT_PROVIDER_STREAM_IDLE_TIMEOUT_SECONDS,
  DEFAULT_PROVIDER_STREAM_FIRST_BYTE_TIMEOUT_SECONDS,
} from './provider-management-utils';

type ProviderDialogState = ReturnType<typeof useProviderDialog>;

export function ProviderFormDialog({
  dialog,
  groups,
}: {
  dialog: ProviderDialogState;
  groups: ProviderGroup[];
}) {
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
      {!dialog.editing ? <ProviderGroupField dialog={dialog} groups={groups} /> : null}
      <ProviderRequestConfigFields dialog={dialog} />
      <ProviderPriorityField dialog={dialog} />
      <ProviderSwitchFields dialog={dialog} />
    </ManagementDialog>
  );
}

function ProviderGroupField({
  dialog,
  groups,
}: {
  dialog: ProviderDialogState;
  groups: ProviderGroup[];
}) {
  const { t } = useTranslate('admin');

  return (
    <TextFieldRow
      select
      label={t('providers.providerGroup')}
      value={dialog.form.provider_group_id}
      onChange={(value) => dialog.setForm((form) => ({ ...form, provider_group_id: value }))}
    >
      <MenuItem value="">{t('providers.unclassifiedProviderGroup')}</MenuItem>
      {groups.map((group) => (
        <MenuItem key={group.id} value={group.id}>
          {group.name}
        </MenuItem>
      ))}
    </TextFieldRow>
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
      <TextFieldRow
        type="number"
        label={t('providers.requestTimeoutSeconds')}
        value={dialog.form.request_timeout_seconds}
        placeholder={String(DEFAULT_PROVIDER_REQUEST_TIMEOUT_SECONDS)}
        helperText={t('providers.defaultWhenBlank')}
        onChange={(value) =>
          dialog.setForm((form) => ({ ...form, request_timeout_seconds: value }))
        }
      />
      <TextFieldRow
        type="number"
        label={t('providers.streamFirstByteTimeoutSeconds')}
        value={dialog.form.stream_first_byte_timeout_seconds}
        placeholder={String(DEFAULT_PROVIDER_STREAM_FIRST_BYTE_TIMEOUT_SECONDS)}
        helperText={t('providers.defaultWhenBlank')}
        onChange={(value) =>
          dialog.setForm((form) => ({ ...form, stream_first_byte_timeout_seconds: value }))
        }
      />
      <TextFieldRow
        type="number"
        label={t('providers.streamIdleTimeoutSeconds')}
        value={dialog.form.stream_idle_timeout_seconds}
        placeholder={String(DEFAULT_PROVIDER_STREAM_IDLE_TIMEOUT_SECONDS)}
        helperText={t('providers.defaultWhenBlank')}
        onChange={(value) =>
          dialog.setForm((form) => ({ ...form, stream_idle_timeout_seconds: value }))
        }
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

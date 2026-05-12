'use client';

import type { useProviderChildDialogs } from './provider-management-state';

import Stack from '@mui/material/Stack';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow, ManagementDialog } from './shared';

export function ProviderApiKeyDialog({
  dialogs,
}: {
  dialogs: ReturnType<typeof useProviderChildDialogs>;
  providerId?: string;
}) {
  const { t } = useTranslate('admin');
  const editing = !!dialogs.editingApiKey;

  return (
    <ManagementDialog
      open={dialogs.apiKeyOpen}
      title={editing ? t('dialogs.editProviderKey') : t('dialogs.createProviderKey')}
      submitting={dialogs.submitting}
      onClose={dialogs.closeApiKey}
      onSubmit={dialogs.submitApiKey}
    >
      <ApiKeyBasicFields dialogs={dialogs} />
      <ApiKeyLimitFields dialogs={dialogs} />
      <ApiKeyTimeRangeFields dialogs={dialogs} />
      <ApiKeySwitches dialogs={dialogs} />
    </ManagementDialog>
  );
}

function ApiKeyBasicFields({ dialogs }: { dialogs: ReturnType<typeof useProviderChildDialogs> }) {
  const { t } = useTranslate('admin');
  const editing = !!dialogs.editingApiKey;

  return (
    <>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          required
          label={t('fields.keyName')}
          value={dialogs.apiKeyForm.name}
          onChange={(value) => dialogs.setApiKeyForm((form) => ({ ...form, name: value }))}
        />
        <TextFieldRow
          required={!editing}
          type="password"
          label={t('fields.apiKey')}
          helperText={editing ? t('providers.keyUpdateSecretHint') : undefined}
          value={dialogs.apiKeyForm.api_key}
          onChange={(value) => dialogs.setApiKeyForm((form) => ({ ...form, api_key: value }))}
        />
      </Stack>
      <TextField
        fullWidth
        multiline
        minRows={2}
        label={t('providers.note')}
        value={dialogs.apiKeyForm.note}
        onChange={(event) =>
          dialogs.setApiKeyForm((form) => ({ ...form, note: event.target.value }))
        }
      />
    </>
  );
}

function ApiKeyLimitFields({ dialogs }: { dialogs: ReturnType<typeof useProviderChildDialogs> }) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
      <TextFieldRow
        type="number"
        label={t('providers.priority')}
        value={dialogs.apiKeyForm.internal_priority}
        onChange={(value) =>
          dialogs.setApiKeyForm((form) => ({ ...form, internal_priority: value }))
        }
      />
      <TextFieldRow
        type="number"
        label={t('providers.rpmLimit')}
        helperText={t('providers.adaptiveWhenBlank')}
        value={dialogs.apiKeyForm.rpm_limit}
        onChange={(value) => dialogs.setApiKeyForm((form) => ({ ...form, rpm_limit: value }))}
      />
    </Stack>
  );
}

function ApiKeyTimeRangeFields({
  dialogs,
}: {
  dialogs: ReturnType<typeof useProviderChildDialogs>;
}) {
  const { t } = useTranslate('admin');

  return (
    <>
      <SwitchRow
        checked={dialogs.apiKeyForm.time_range_enabled}
        label={t('providers.timeRangeEnabled')}
        onChange={(checked) =>
          dialogs.setApiKeyForm((form) => ({ ...form, time_range_enabled: checked }))
        }
      />
      {dialogs.apiKeyForm.time_range_enabled && (
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
          <TextFieldRow
            label={t('providers.timeRangeStart')}
            value={dialogs.apiKeyForm.time_range_start}
            onChange={(value) =>
              dialogs.setApiKeyForm((form) => ({ ...form, time_range_start: value }))
            }
          />
          <TextFieldRow
            label={t('providers.timeRangeEnd')}
            value={dialogs.apiKeyForm.time_range_end}
            onChange={(value) =>
              dialogs.setApiKeyForm((form) => ({ ...form, time_range_end: value }))
            }
          />
        </Stack>
      )}
    </>
  );
}

function ApiKeySwitches({ dialogs }: { dialogs: ReturnType<typeof useProviderChildDialogs> }) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
      <SwitchRow
        checked={dialogs.apiKeyForm.is_active}
        label={t('common.enabled')}
        onChange={(checked) => dialogs.setApiKeyForm((form) => ({ ...form, is_active: checked }))}
      />
    </Stack>
  );
}

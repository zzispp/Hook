'use client';

import type { useProviderQuickImportSyncDialog } from './provider-management-state';

import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { TextFieldRow, ManagementDialog } from './shared';
import { ProviderQuickImportSyncConfigFields } from './provider-quick-import-sync-section';

type SyncDialogState = ReturnType<typeof useProviderQuickImportSyncDialog>;

export function ProviderQuickImportSyncDialog({ dialog }: { dialog: SyncDialogState }) {
  const { t } = useTranslate('admin');
  const disabled = dialog.loading || !dialog.form.hasSource;

  return (
    <ManagementDialog
      open={!!dialog.provider}
      title={t('providers.quickImportSyncSettingsTitle', {
        name: dialog.provider?.name ?? '',
      })}
      submitting={dialog.submitting}
      submitDisabled={dialog.loading || !dialog.valid}
      onClose={dialog.close}
      onSubmit={dialog.submit}
    >
      <Stack spacing={2} divider={<Divider flexItem />}>
        <Stack spacing={1.5}>
          <Typography variant="subtitle2">{t('providers.quickImportSourceSection')}</Typography>
          {!dialog.form.hasSource ? (
            <Alert severity="info" variant="outlined">
              {t('providers.quickImportSourceNotConfigured')}
            </Alert>
          ) : null}
          <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
            <TextFieldRow
              disabled={disabled}
              label={t('providers.quickImportBaseUrl')}
              value={dialog.form.baseUrl}
              onChange={(value) => dialog.setForm((form) => ({ ...form, baseUrl: value }))}
            />
            <TextFieldRow
              disabled={disabled}
              label={t('providers.quickImportUserId')}
              value={dialog.form.userId}
              onChange={(value) => dialog.setForm((form) => ({ ...form, userId: value }))}
            />
          </Stack>
          <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
            <TextFieldRow
              disabled={disabled}
              type="password"
              label={t('providers.quickImportSystemToken')}
              value={dialog.form.systemAccessToken}
              helperText={t('providers.quickImportSystemTokenUpdateHint')}
              onChange={(value) =>
                dialog.setForm((form) => ({ ...form, systemAccessToken: value }))
              }
            />
            <TextFieldRow
              disabled={disabled}
              type="number"
              label={t('providers.quickImportRechargeMultiplier')}
              value={dialog.form.rechargeMultiplier}
              onChange={(value) =>
                dialog.setForm((form) => ({ ...form, rechargeMultiplier: value }))
              }
            />
          </Stack>
        </Stack>
        <Stack spacing={1.5}>
          <Typography variant="subtitle2">{t('providers.quickImportSyncSection')}</Typography>
          <ProviderQuickImportSyncConfigFields
            disabled={disabled}
            form={dialog.form.sync}
            onChange={(sync) => dialog.setForm((form) => ({ ...form, sync }))}
          />
        </Stack>
      </Stack>
    </ManagementDialog>
  );
}

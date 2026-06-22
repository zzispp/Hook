'use client';

import type { QuickImportAuthTab } from './provider-quick-import-source';
import type { useProviderQuickImportSyncDialog } from './provider-management-state';

import Tab from '@mui/material/Tab';
import Tabs from '@mui/material/Tabs';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { TextFieldRow, ManagementDialog } from './shared';
import { ProviderQuickImportSyncConfigFields } from './provider-quick-import-sync-section';
import { ProviderQuickImportSub2apiTokenHelp } from './provider-quick-import-sub2api-token-help';

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
              label={t('providers.quickImportType')}
              value={dialog.form.sourceKind || '-'}
              onChange={() => undefined}
            />
            <TextFieldRow
              disabled={disabled}
              label={t('providers.quickImportBaseUrl')}
              value={dialog.form.baseUrl}
              onChange={(value) => dialog.setForm((form) => ({ ...form, baseUrl: value }))}
            />
          </Stack>
          {dialog.form.sourceKind === 'sub2api' ? (
            <Stack spacing={2}>
              <Tabs
                value={dialog.form.sub2apiAuthTab}
                onChange={(_event, value: QuickImportAuthTab) =>
                  dialog.setForm((form) => ({ ...form, sub2apiAuthTab: value }))
                }
              >
                <Tab value="password" label={t('providers.quickImportSub2apiPasswordImport')} />
                <Tab value="token" label={t('providers.quickImportSub2apiTokenImport')} />
              </Tabs>
              {dialog.form.sub2apiAuthTab === 'token' ? (
                <>
                  <ProviderQuickImportSub2apiTokenHelp
                    disabled={disabled}
                    onApply={({ authToken, refreshToken, tokenExpiresAt }) =>
                      dialog.setForm((form) => ({ ...form, authToken, refreshToken, tokenExpiresAt }))
                    }
                  />
                  <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
                    <TextFieldRow
                      disabled={disabled}
                      label={t('providers.quickImportSub2apiAuthToken')}
                      value={dialog.form.authToken}
                      helperText={dialog.form.hasAuthToken ? t('providers.quickImportSystemTokenUpdateHint') : undefined}
                      onChange={(value) => dialog.setForm((form) => ({ ...form, authToken: value }))}
                    />
                    <TextFieldRow
                      disabled={disabled}
                      type="password"
                      label={t('providers.quickImportSub2apiRefreshToken')}
                      value={dialog.form.refreshToken}
                      helperText={dialog.form.hasRefreshToken ? t('providers.quickImportPasswordUpdateHint') : undefined}
                      onChange={(value) => dialog.setForm((form) => ({ ...form, refreshToken: value }))}
                    />
                    <TextFieldRow
                      disabled={disabled}
                      label={t('providers.quickImportSub2apiTokenExpiresAt')}
                      value={dialog.form.tokenExpiresAt}
                      onChange={(value) => dialog.setForm((form) => ({ ...form, tokenExpiresAt: value }))}
                    />
                    <TextFieldRow
                      disabled={disabled}
                      type="number"
                      label={t('providers.quickImportRechargeMultiplier')}
                      value={dialog.form.rechargeMultiplier}
                      onChange={(value) => dialog.setForm((form) => ({ ...form, rechargeMultiplier: value }))}
                    />
                  </Stack>
                </>
              ) : (
                <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
                  <TextFieldRow
                    disabled={disabled}
                    label={t('fields.email')}
                    value={dialog.form.email}
                    onChange={(value) => dialog.setForm((form) => ({ ...form, email: value }))}
                  />
                  <TextFieldRow
                    disabled={disabled}
                    type="password"
                    label={t('fields.password')}
                    value={dialog.form.password}
                    helperText={dialog.form.hasPassword ? t('providers.quickImportPasswordUpdateHint') : undefined}
                    onChange={(value) => dialog.setForm((form) => ({ ...form, password: value }))}
                  />
                  <TextFieldRow
                    disabled={disabled}
                    type="number"
                    label={t('providers.quickImportRechargeMultiplier')}
                    value={dialog.form.rechargeMultiplier}
                    onChange={(value) => dialog.setForm((form) => ({ ...form, rechargeMultiplier: value }))}
                  />
                </Stack>
              )}
            </Stack>
          ) : (
            <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
              <TextFieldRow
                disabled={disabled}
                label={t('providers.quickImportUserId')}
                value={dialog.form.userId}
                onChange={(value) => dialog.setForm((form) => ({ ...form, userId: value }))}
              />
              <TextFieldRow
                disabled={disabled}
                type="password"
                label={t('providers.quickImportSystemToken')}
                value={dialog.form.systemAccessToken}
                helperText={t('providers.quickImportSystemTokenUpdateHint')}
                onChange={(value) => dialog.setForm((form) => ({ ...form, systemAccessToken: value }))}
              />
              <TextFieldRow
                disabled={disabled}
                type="number"
                label={t('providers.quickImportRechargeMultiplier')}
                value={dialog.form.rechargeMultiplier}
                onChange={(value) => dialog.setForm((form) => ({ ...form, rechargeMultiplier: value }))}
              />
            </Stack>
          )}
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

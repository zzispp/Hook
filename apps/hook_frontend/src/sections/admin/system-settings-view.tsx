'use client';

import type { SystemSettingsForm } from './system-settings-utils';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_TITLES } from 'src/layouts/dashboard/dashboard-menu-values';
import { useSystemSettings } from 'src/actions/system-settings';

import { Iconify } from 'src/components/iconify';

import { useSystemSettingsForm } from './system-settings-state';
import { SwitchRow, TextFieldRow, RefreshButton, AdminBreadcrumbs } from './shared';

export function SystemSettingsView() {
  const { t } = useTranslate('admin');
  const settings = useSystemSettings();
  const form = useSystemSettingsForm(settings.data, t);

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        heading={DASHBOARD_MENU_TITLES.systemSettings}
        action={
          <Stack direction="row" spacing={1}>
            <RefreshButton loading={settings.isLoading} onClick={() => void settings.refresh()} />
            <Button
              variant="contained"
              loading={form.submitting}
              startIcon={<Iconify icon="solar:check-circle-bold" />}
              onClick={form.submit}
            >
              {t('common.save')}
            </Button>
          </Stack>
        }
      />

      <Card sx={{ p: 3 }}>
        <Stack spacing={3}>
          <SiteSection form={form.form} setForm={form.setForm} />
          <Divider />
          <BaseSection form={form.form} setForm={form.setForm} />
          <Divider />
          <TokenSection form={form.form} setForm={form.setForm} />
        </Stack>
      </Card>
    </DashboardContent>
  );
}

function SiteSection({
  form,
  setForm,
}: {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
}) {
  const { t } = useTranslate('admin');

  return (
    <SettingsSection title={t('systemSettings.sections.site')}>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          required
          label={t('systemSettings.fields.siteName')}
          value={form.site_name}
          onChange={(value) => setForm((current) => ({ ...current, site_name: value }))}
        />
        <TextFieldRow
          label={t('systemSettings.fields.siteSubtitle')}
          value={form.site_subtitle}
          onChange={(value) => setForm((current) => ({ ...current, site_subtitle: value }))}
        />
      </Stack>
    </SettingsSection>
  );
}

function BaseSection({
  form,
  setForm,
}: {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
}) {
  const { t } = useTranslate('admin');

  return (
    <SettingsSection title={t('systemSettings.sections.base')}>
      <Stack spacing={2}>
        <SwitchRow
          checked={form.allow_registration}
          label={t('systemSettings.fields.allowRegistration')}
          onChange={(checked) => setForm((current) => ({ ...current, allow_registration: checked }))}
        />
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.defaultUserGrant')}
          value={form.default_user_grant}
          helperText={t('systemSettings.helper.defaultUserGrant')}
          onChange={(value) => setForm((current) => ({ ...current, default_user_grant: value }))}
        />
      </Stack>
    </SettingsSection>
  );
}

function TokenSection({
  form,
  setForm,
}: {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
}) {
  const { t } = useTranslate('admin');

  return (
    <SettingsSection title={t('systemSettings.sections.tokens')}>
      <Stack spacing={2}>
        <SwitchRow
          checked={form.auto_delete_expired_tokens}
          label={t('systemSettings.fields.autoDeleteExpiredTokens')}
          onChange={(checked) => setForm((current) => ({ ...current, auto_delete_expired_tokens: checked }))}
        />
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.defaultRateLimitRpm')}
          value={form.default_rate_limit_rpm}
          helperText={t('systemSettings.helper.defaultRateLimitRpm')}
          onChange={(value) => setForm((current) => ({ ...current, default_rate_limit_rpm: value }))}
        />
      </Stack>
    </SettingsSection>
  );
}

function SettingsSection({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <Stack spacing={2}>
      <Typography variant="subtitle1">{title}</Typography>
      {children}
    </Stack>
  );
}

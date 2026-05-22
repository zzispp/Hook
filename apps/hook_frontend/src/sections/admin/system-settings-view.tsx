'use client';

import type { SystemSettingsForm } from './system-settings-utils';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Avatar from '@mui/material/Avatar';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';

import { labelWithAccountingCurrency } from 'src/utils/money-boundary';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useSystemSettings } from 'src/actions/system-settings';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { Iconify } from 'src/components/iconify';
import { logoImageSource } from 'src/components/logo/logo-utils';

import { SettingsSection } from './system-settings-section';
import { emailConfigComplete } from './system-settings-utils';
import { useSystemSettingsForm } from './system-settings-state';
import { EmailSettingsSection } from './system-settings-email-section';
import { CleanupSettingsSection } from './system-settings-cleanup-section';
import { RequestRecordSection } from './system-settings-request-record-section';
import { SwitchRow, TextFieldRow, RefreshButton, AdminBreadcrumbs } from './shared';

export function SystemSettingsView() {
  const { t } = useTranslate('admin');
  const settings = useSystemSettings();
  const form = useSystemSettingsForm(settings.data, t);

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.systemSettings}
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
          <EmailSettingsSection form={form.form} setForm={form.setForm} />
          <Divider />
          <TokenSection form={form.form} setForm={form.setForm} />
          <Divider />
          <RequestRecordSection form={form.form} setForm={form.setForm} />
          <Divider />
          <CleanupSettingsSection form={form.form} setForm={form.setForm} />
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
      <LogoField form={form} setForm={setForm} />
    </SettingsSection>
  );
}

function LogoField({
  form,
  setForm,
}: {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
}) {
  const { t } = useTranslate('admin');
  const logoSrc = logoImageSource(form.site_logo_base64);

  return (
    <Stack
      spacing={2}
      direction={{ xs: 'column', md: 'row' }}
      sx={{ mt: 2, alignItems: 'center' }}
    >
      <Avatar variant="rounded" src={logoSrc} sx={{ width: 64, height: 64 }} />
      <Stack spacing={1} sx={{ flex: 1 }}>
        <Stack spacing={1} direction="row">
          <Button
            component="label"
            variant="outlined"
            startIcon={<Iconify icon="solar:import-bold" />}
          >
            {t('systemSettings.fields.siteLogoUpload')}
            <input
              hidden
              type="file"
              accept="image/*"
              onChange={(event) => readLogoFile(event, setForm)}
            />
          </Button>
        </Stack>
      </Stack>
    </Stack>
  );
}

function readLogoFile(
  event: React.ChangeEvent<HTMLInputElement>,
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>
) {
  const file = event.target.files?.[0];
  event.target.value = '';
  if (!file) {
    return;
  }

  const reader = new FileReader();
  reader.onload = () => {
    const result = reader.result;
    if (typeof result !== 'string') {
      return;
    }
    setForm((current) => ({ ...current, site_logo_base64: result }));
  };
  reader.readAsDataURL(file);
}

function BaseSection({
  form,
  setForm,
}: {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
}) {
  const { t } = useTranslate('admin');
  const emailVerificationReady = form.email_config_enabled && emailConfigComplete(form);
  const emailVerificationDisabled =
    !emailVerificationReady && !form.registration_email_verification_enabled;
  const passwordResetDisabled = !emailVerificationReady && !form.password_reset_enabled;

  return (
    <SettingsSection title={t('systemSettings.sections.base')}>
      <Stack spacing={2}>
        <SwitchRow
          checked={form.allow_registration}
          label={t('systemSettings.fields.allowRegistration')}
          onChange={(checked) =>
            setForm((current) => ({ ...current, allow_registration: checked }))
          }
        />
        <SwitchRow
          checked={form.login_captcha_enabled}
          label={t('systemSettings.fields.loginCaptchaEnabled')}
          onChange={(checked) =>
            setForm((current) => ({ ...current, login_captcha_enabled: checked }))
          }
        />
        <SwitchRow
          checked={form.registration_captcha_enabled}
          label={t('systemSettings.fields.registrationCaptchaEnabled')}
          onChange={(checked) =>
            setForm((current) => ({ ...current, registration_captcha_enabled: checked }))
          }
        />
        <SwitchRow
          checked={form.support_ticket_captcha_enabled}
          label={t('systemSettings.fields.supportTicketCaptchaEnabled')}
          helperText={t('systemSettings.helper.supportTicketCaptchaEnabled')}
          onChange={(checked) =>
            setForm((current) => ({ ...current, support_ticket_captcha_enabled: checked }))
          }
        />
        <SwitchRow
          checked={form.registration_email_verification_enabled}
          disabled={emailVerificationDisabled}
          label={t('systemSettings.fields.registrationEmailVerificationEnabled')}
          helperText={
            emailVerificationReady
              ? undefined
              : t('systemSettings.helper.registrationEmailVerificationRequiresEmailConfig')
          }
          onChange={(checked) =>
            setForm((current) => ({
              ...current,
              registration_email_verification_enabled: checked,
            }))
          }
        />
        <SwitchRow
          checked={form.password_reset_enabled}
          disabled={passwordResetDisabled}
          label={t('systemSettings.fields.passwordResetEnabled')}
          helperText={
            emailVerificationReady
              ? undefined
              : t('systemSettings.helper.passwordResetRequiresEmailConfig')
          }
          onChange={(checked) =>
            setForm((current) => ({
              ...current,
              password_reset_enabled: checked,
            }))
          }
        />
        <TextFieldRow
          type="number"
          label={labelWithAccountingCurrency(t('systemSettings.fields.defaultUserGrant'))}
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
          onChange={(checked) =>
            setForm((current) => ({ ...current, auto_delete_expired_tokens: checked }))
          }
        />
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.tokenLimitPerUser')}
          value={form.token_limit_per_user}
          helperText={t('systemSettings.helper.tokenLimitPerUser')}
          onChange={(value) => setForm((current) => ({ ...current, token_limit_per_user: value }))}
        />
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.tokenExpiryCheckIntervalMinutes')}
          value={form.token_expiry_check_interval_minutes}
          helperText={t('systemSettings.helper.tokenExpiryCheckIntervalMinutes')}
          onChange={(value) =>
            setForm((current) => ({ ...current, token_expiry_check_interval_minutes: value }))
          }
        />
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.defaultRateLimitRpm')}
          value={form.default_rate_limit_rpm}
          helperText={t('systemSettings.helper.defaultRateLimitRpm')}
          onChange={(value) =>
            setForm((current) => ({ ...current, default_rate_limit_rpm: value }))
          }
        />
      </Stack>
    </SettingsSection>
  );
}

'use client';

import type { SystemSettingsForm } from './system-settings-utils';

import { useMemo, useCallback } from 'react';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Avatar from '@mui/material/Avatar';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';

import { useTranslate } from 'src/locales/use-locales';
import { useUserGroups } from 'src/actions/user-groups';
import { DashboardContent } from 'src/layouts/dashboard';
import { usePaymentChannels } from 'src/actions/recharge';
import { useSystemSettings } from 'src/actions/system-settings';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { Iconify } from 'src/components/iconify';
import { logoImageSource } from 'src/components/logo/logo-utils';

import { SettingsSection } from './system-settings-section';
import { useSystemSettingsForm } from './system-settings-state';
import { EmailSettingsSection } from './system-settings-email-section';
import { TextFieldRow, RefreshButton, AdminBreadcrumbs } from './shared';
import { SystemSettingsBaseSection } from './system-settings-base-section';
import { RechargeSettingsSection } from './system-settings-recharge-section';
import { RequestRecordSection } from './system-settings-request-record-section';
import { enabledUserGroupOptions, USER_GROUP_MAX_PAGE_SIZE } from './user-group-utils';
import {
  usePaymentChannelForms,
  paymentChannelsWithForms,
  saveSystemSettingsAndPaymentChannels,
} from './system-settings-payment-channel-state';

export function SystemSettingsView() {
  const { t } = useTranslate('admin');
  const settings = useSystemSettings();
  const paymentChannels = usePaymentChannels();
  const userGroups = useUserGroups(0, USER_GROUP_MAX_PAGE_SIZE, { is_active: true });
  const channelForms = usePaymentChannelForms(paymentChannels.data ?? []);
  const effectivePaymentChannels = useMemo(
    () => paymentChannelsWithForms(paymentChannels.data ?? [], channelForms.forms),
    [channelForms.forms, paymentChannels.data]
  );
  const saveAllSettings = useCallback(
    (formToSave: SystemSettingsForm) =>
      saveSystemSettingsAndPaymentChannels({
        form: formToSave,
        channels: paymentChannels.data ?? [],
        channelForms: channelForms.forms,
        t,
      }),
    [channelForms.forms, paymentChannels.data, t]
  );
  const validationContext = {
    paymentChannels: effectivePaymentChannels,
    paymentChannelsLoading: paymentChannels.isLoading,
    paymentChannelsError: paymentChannels.error,
    userGroups: userGroups.items,
    userGroupsTotal: userGroups.total,
    userGroupsLoading: userGroups.isLoading,
    userGroupsError: userGroups.error,
  };
  const form = useSystemSettingsForm(settings.data, t, validationContext, {
    save: saveAllSettings,
    afterSave: async () => {
      await paymentChannels.refresh();
    },
  });
  const userGroupOptions = enabledUserGroupOptions(userGroups.items);

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
          <SystemSettingsBaseSection
            form={form.form}
            setForm={form.setForm}
            userGroups={userGroupOptions}
          />
          <Divider />
          <EmailSettingsSection form={form.form} setForm={form.setForm} />
          <Divider />
          <TokenSection form={form.form} setForm={form.setForm} />
          <Divider />
          <RechargeSettingsSection
            form={form.form}
            setForm={form.setForm}
            channels={paymentChannels.data ?? []}
            channelForms={channelForms.forms}
            channelsLoading={paymentChannels.isLoading}
            channelsErrorMessage={paymentChannels.error?.message}
            setChannelForm={channelForms.setForm}
          />
          <Divider />
          <RequestRecordSection form={form.form} setForm={form.setForm} />
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
      <TextFieldRow
        label={t('systemSettings.fields.publicBaseUrl')}
        value={form.public_base_url}
        onChange={(value) => setForm((current) => ({ ...current, public_base_url: value }))}
        sx={{ mt: 2 }}
      />
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
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.tokenLimitPerUser')}
          value={form.token_limit_per_user}
          helperText={t('systemSettings.helper.tokenLimitPerUser')}
          onChange={(value) => setForm((current) => ({ ...current, token_limit_per_user: value }))}
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

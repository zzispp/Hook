'use client';

import type { SystemSettingsForm } from './system-settings-utils';

import { useMemo, useState, useCallback } from 'react';

import Tab from '@mui/material/Tab';
import Card from '@mui/material/Card';
import Tabs from '@mui/material/Tabs';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { useTranslate } from 'src/locales/use-locales';
import { useUserGroups } from 'src/actions/user-groups';
import { DashboardContent } from 'src/layouts/dashboard';
import { usePaymentChannels } from 'src/actions/recharge';
import { useSystemSettings } from 'src/actions/system-settings';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { Iconify } from 'src/components/iconify';

import { RefreshButton, AdminBreadcrumbs } from './shared';
import { useSystemSettingsForm } from './system-settings-state';
import { EmailSettingsSection } from './system-settings-email-section';
import { SystemSettingsSiteSection } from './system-settings-site-section';
import { SystemSettingsTokenSection } from './system-settings-token-section';
import { RechargeSettingsSection } from './system-settings-recharge-section';
import { RequestRecordSection } from './system-settings-request-record-section';
import { enabledUserGroupOptions, USER_GROUP_MAX_PAGE_SIZE } from './user-group-utils';
import { SystemSettingsRegistrationSection } from './system-settings-registration-section';
import {
  usePaymentChannelForms,
  paymentChannelsWithForms,
  saveSystemSettingsAndPaymentChannels,
} from './system-settings-payment-channel-state';

type SystemSettingsTab =
  | 'site'
  | 'registration'
  | 'email'
  | 'tokens'
  | 'recharge'
  | 'requestRecord';

const SYSTEM_SETTINGS_TABS: ReadonlyArray<{
  value: SystemSettingsTab;
  labelKey: string;
}> = [
  { value: 'site', labelKey: 'systemSettings.sections.site' },
  { value: 'registration', labelKey: 'systemSettings.sections.registration' },
  { value: 'email', labelKey: 'systemSettings.sections.email' },
  { value: 'tokens', labelKey: 'systemSettings.sections.tokens' },
  { value: 'recharge', labelKey: 'systemSettings.sections.recharge' },
  { value: 'requestRecord', labelKey: 'systemSettings.sections.requestRecord' },
];

export function SystemSettingsView() {
  const { t } = useTranslate('admin');
  const [tab, setTab] = useState<SystemSettingsTab>('site');
  const state = useSystemSettingsPageState(t);

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.systemSettings}
        action={
          <HeaderActions
            loading={state.settings.isLoading}
            submitting={state.form.submitting}
            onRefresh={state.settings.refresh}
            onSubmit={state.form.submit}
          />
        }
      />

      <SettingsTabs tab={tab} setTab={setTab} />
      <Card sx={{ p: 3 }}>
        <SettingsTabPanel tab={tab} {...panelProps(state)} />
      </Card>
    </DashboardContent>
  );
}

function useSystemSettingsPageState(t: ReturnType<typeof useTranslate>['t']) {
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

  return { settings, paymentChannels, channelForms, form, userGroupOptions };
}

function HeaderActions({
  loading,
  submitting,
  onRefresh,
  onSubmit,
}: {
  loading: boolean;
  submitting: boolean;
  onRefresh: () => unknown;
  onSubmit: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction="row" spacing={1}>
      <RefreshButton loading={loading} onClick={() => void onRefresh()} />
      <Button
        variant="contained"
        loading={submitting}
        startIcon={<Iconify icon="solar:check-circle-bold" />}
        onClick={onSubmit}
      >
        {t('common.save')}
      </Button>
    </Stack>
  );
}

function panelProps(state: ReturnType<typeof useSystemSettingsPageState>) {
  return {
    form: state.form.form,
    setForm: state.form.setForm,
    userGroups: state.userGroupOptions,
    channels: state.paymentChannels.data ?? [],
    channelForms: state.channelForms.forms,
    channelsLoading: state.paymentChannels.isLoading,
    channelsErrorMessage: state.paymentChannels.error?.message,
    setChannelForm: state.channelForms.setForm,
  };
}

function SettingsTabs({
  tab,
  setTab,
}: {
  tab: SystemSettingsTab;
  setTab: (value: SystemSettingsTab) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Tabs
      value={tab}
      variant="scrollable"
      allowScrollButtonsMobile
      onChange={(_, value: SystemSettingsTab) => setTab(value)}
      sx={{ mb: 3 }}
    >
      {SYSTEM_SETTINGS_TABS.map((item) => (
        <Tab key={item.value} value={item.value} label={t(item.labelKey)} />
      ))}
    </Tabs>
  );
}

function SettingsTabPanel(props: {
  tab: SystemSettingsTab;
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
  userGroups: ReturnType<typeof enabledUserGroupOptions>;
  channels: Parameters<typeof RechargeSettingsSection>[0]['channels'];
  channelForms: Parameters<typeof RechargeSettingsSection>[0]['channelForms'];
  channelsLoading: boolean;
  channelsErrorMessage?: string;
  setChannelForm: Parameters<typeof RechargeSettingsSection>[0]['setChannelForm'];
}) {
  if (props.tab === 'site') {
    return <SystemSettingsSiteSection form={props.form} setForm={props.setForm} />;
  }
  if (props.tab === 'registration') {
    return (
      <SystemSettingsRegistrationSection
        form={props.form}
        setForm={props.setForm}
        userGroups={props.userGroups}
      />
    );
  }
  if (props.tab === 'email') {
    return <EmailSettingsSection form={props.form} setForm={props.setForm} />;
  }
  if (props.tab === 'tokens') {
    return <SystemSettingsTokenSection form={props.form} setForm={props.setForm} />;
  }
  if (props.tab === 'recharge') {
    return <RechargeSettingsSection {...rechargeProps(props)} />;
  }
  return <RequestRecordSection form={props.form} setForm={props.setForm} />;
}

function rechargeProps(props: Parameters<typeof SettingsTabPanel>[0]) {
  return {
    form: props.form,
    setForm: props.setForm,
    channels: props.channels,
    channelForms: props.channelForms,
    channelsLoading: props.channelsLoading,
    channelsErrorMessage: props.channelsErrorMessage,
    setChannelForm: props.setChannelForm,
  };
}

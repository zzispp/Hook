'use client';

import type { PaymentChannel } from 'src/types/recharge';
import type { SystemSettingsForm } from './system-settings-utils';
import type { PaymentChannelForm } from './system-settings-payment-channel-panel';

import Stack from '@mui/material/Stack';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow } from './shared';
import { SettingsSection } from './system-settings-section';
import { PaymentChannelConfigPanel } from './system-settings-payment-channel-panel';

type Props = {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
  channels: PaymentChannel[];
  channelForms: Record<string, PaymentChannelForm>;
  channelsLoading: boolean;
  channelsErrorMessage?: string;
  setChannelForm: (code: string, form: PaymentChannelForm) => void;
};

export function RechargeSettingsSection({
  form,
  setForm,
  channels,
  channelForms,
  channelsLoading,
  channelsErrorMessage,
  setChannelForm,
}: Props) {
  const { t } = useTranslate('admin');

  return (
    <SettingsSection
      title={t('systemSettings.sections.recharge')}
      description={t('systemSettings.recharge.description')}
    >
      <Stack spacing={2}>
        <SwitchRow
          checked={form.recharge_enabled}
          label={t('systemSettings.fields.rechargeEnabled')}
          onChange={(checked) => setForm((current) => ({ ...current, recharge_enabled: checked }))}
        />
        <SwitchRow
          checked={form.recharge_captcha_enabled}
          label={t('systemSettings.fields.rechargeCaptchaEnabled')}
          helperText={t('systemSettings.helper.rechargeCaptchaEnabled')}
          onChange={(checked) =>
            setForm((current) => ({ ...current, recharge_captcha_enabled: checked }))
          }
        />
        <RechargeAmountFields form={form} setForm={setForm} />
        <Typography variant="body2" color="text.secondary">
          {t('systemSettings.recharge.preview', { ratio: ratioLabel(form.recharge_arrival_ratio) })}
        </Typography>
        <Divider />
        <PaymentChannelList
          loading={channelsLoading}
          errorMessage={channelsErrorMessage}
          channels={channels}
          channelForms={channelForms}
          setChannelForm={setChannelForm}
        />
      </Stack>
    </SettingsSection>
  );
}

function RechargeAmountFields({ form, setForm }: Pick<Props, 'form' | 'setForm'>) {
  const { t } = useTranslate('admin');
  const setField = (field: keyof SystemSettingsForm, value: string) =>
    setForm((current) => ({ ...current, [field]: value }));

  return (
    <>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.rechargeArrivalRatio')}
          value={form.recharge_arrival_ratio}
          onChange={(value) => setField('recharge_arrival_ratio', value)}
          slotProps={{ htmlInput: { min: 0, step: 0.01 } }}
        />
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.rechargeOrderExpireMinutes')}
          value={form.recharge_order_expire_minutes}
          onChange={(value) => setField('recharge_order_expire_minutes', value)}
          slotProps={{ htmlInput: { min: 1, step: 1 } }}
        />
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.rechargeMaxUnpaidOrders')}
          value={form.recharge_max_unpaid_orders}
          onChange={(value) => setField('recharge_max_unpaid_orders', value)}
          slotProps={{ htmlInput: { min: 1, step: 1 } }}
        />
      </Stack>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.rechargeMinAmount')}
          value={form.recharge_min_amount}
          onChange={(value) => setField('recharge_min_amount', value)}
          slotProps={{ htmlInput: { min: 0.01, step: 0.01 } }}
        />
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.rechargeMaxAmount')}
          value={form.recharge_max_amount}
          onChange={(value) => setField('recharge_max_amount', value)}
          slotProps={{ htmlInput: { min: 0.01, step: 0.01 } }}
        />
      </Stack>
    </>
  );
}

function PaymentChannelList({
  loading,
  errorMessage,
  channels,
  channelForms,
  setChannelForm,
}: {
  loading: boolean;
  errorMessage?: string;
  channels: PaymentChannel[];
  channelForms: Record<string, PaymentChannelForm>;
  setChannelForm: (code: string, form: PaymentChannelForm) => void;
}) {
  const { t } = useTranslate('admin');

  if (errorMessage) {
    return <Typography color="error">{errorMessage}</Typography>;
  }

  if (loading) {
    return <Typography color="text.secondary">{t('common.loading')}</Typography>;
  }

  if (channels.length === 0) {
    return (
      <Typography variant="body2" color="text.secondary">
        {t('systemSettings.recharge.noPaymentChannels')}
      </Typography>
    );
  }

  return (
    <Stack spacing={1}>
      <Typography variant="subtitle2">{t('systemSettings.recharge.paymentChannels')}</Typography>
      {channels.map((channel) => {
        const form = channelForms[channel.code];
        if (!form) {
          return null;
        }
        return (
          <PaymentChannelConfigPanel
            key={channel.code}
            channel={channel}
            form={form}
            t={t}
            onChange={(value) => setChannelForm(channel.code, value)}
          />
        );
      })}
    </Stack>
  );
}

function ratioLabel(value: string) {
  const ratio = Number(value || 0);
  if (!Number.isFinite(ratio)) return value;
  return ratio.toString();
}

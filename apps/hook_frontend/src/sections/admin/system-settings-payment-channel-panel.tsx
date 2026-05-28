'use client';

import type { AdminT } from './shared';
import type { PaymentChannel } from 'src/types/recharge';

import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { Label } from 'src/components/label';

import { SwitchRow, TextFieldRow } from './shared';

export type PaymentChannelForm = {
  enabled: boolean;
  merchantId: string;
  apiBaseUrl: string;
  apiKey: string;
};

type Props = {
  channel: PaymentChannel;
  form: PaymentChannelForm;
  t: AdminT;
  onChange: (value: PaymentChannelForm) => void;
};

export function PaymentChannelConfigPanel({ channel, form, t, onChange }: Props) {
  const setField = (field: keyof PaymentChannelForm, value: string | boolean) => {
    onChange({ ...form, [field]: value });
  };

  return (
    <Stack spacing={2} sx={panelSx}>
      <PaymentChannelHeader channel={channel} t={t} />
      <SwitchRow
        checked={form.enabled}
        label={t('common.enabled')}
        onChange={(checked) => setField('enabled', checked)}
      />
      <EpayConfigFields form={form} setField={setField} t={t} secretSet={channel.secret_set} />
    </Stack>
  );
}

export function paymentChannelFormFromChannel(channel: PaymentChannel): PaymentChannelForm {
  return {
    enabled: channel.enabled,
    merchantId: configString(channel, 'merchant_id'),
    apiBaseUrl: configString(channel, 'api_base_url'),
    apiKey: '',
  };
}

function PaymentChannelHeader({ channel, t }: Pick<Props, 'channel' | 't'>) {
  return (
    <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1} justifyContent="space-between">
      <Stack spacing={0.5}>
        <Typography variant="subtitle2">
          {channel.name} ({channel.code})
        </Typography>
        <Typography variant="caption" color="text.secondary">
          {t('systemSettings.recharge.epayConfig')}
        </Typography>
      </Stack>
      <Label color={channel.enabled ? 'success' : 'default'} variant="soft">
        {channel.enabled ? t('common.enabled') : t('common.disabled')}
      </Label>
    </Stack>
  );
}

function EpayConfigFields({
  form,
  setField,
  t,
  secretSet,
}: {
  form: PaymentChannelForm;
  setField: (field: keyof PaymentChannelForm, value: string | boolean) => void;
  t: AdminT;
  secretSet: boolean;
}) {
  return (
    <Stack spacing={2}>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          required
          label={t('systemSettings.recharge.merchantId')}
          value={form.merchantId}
          onChange={(value) => setField('merchantId', value)}
        />
        <TextFieldRow
          required
          label={t('systemSettings.recharge.apiBaseUrl')}
          value={form.apiBaseUrl}
          placeholder="https://pay.example.com"
          onChange={(value) => setField('apiBaseUrl', value)}
        />
      </Stack>
      <TextFieldRow
        type="password"
        label={t('systemSettings.recharge.apiKey')}
        value={form.apiKey}
        helperText={secretSet ? t('systemSettings.recharge.apiKeySet') : undefined}
        slotProps={{ htmlInput: { autoComplete: 'new-password' } }}
        onChange={(value) => setField('apiKey', value)}
      />
    </Stack>
  );
}

function configString(channel: PaymentChannel, key: string) {
  const value = channel.config[key];
  return typeof value === 'string' ? value : '';
}

const panelSx = {
  p: 2,
  border: (theme: { palette: { divider: string } }) => `1px solid ${theme.palette.divider}`,
  borderRadius: 1,
};

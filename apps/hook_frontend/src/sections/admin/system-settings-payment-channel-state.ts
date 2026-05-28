'use client';

import type { AdminT } from './shared';
import type { PaymentChannel } from 'src/types/recharge';
import type { SystemSettingsForm } from './system-settings-utils';
import type { PaymentChannelForm } from './system-settings-payment-channel-panel';

import { useRef, useMemo, useState, useEffect, useCallback } from 'react';

import { updateSystemSettings } from 'src/actions/system-settings';

import { settingsPayload } from './system-settings-utils';
import { paymentChannelFormFromChannel } from './system-settings-payment-channel-panel';
import {
  hasChangedPaymentChannels,
  saveChangedPaymentChannels,
  validateChangedPaymentChannels,
} from './system-settings-payment-channel-save';

export function usePaymentChannelForms(channels: PaymentChannel[]) {
  const [forms, setForms] = useState<Record<string, PaymentChannelForm>>({});
  const baselineRef = useRef('');
  const baselineKey = useMemo(
    () => channels.map((channel) => `${channel.code}:${channel.updated_at}`).join('|'),
    [channels]
  );

  useEffect(() => {
    if (baselineRef.current === baselineKey) {
      return;
    }
    baselineRef.current = baselineKey;
    setForms(
      Object.fromEntries(
        channels.map((channel) => [channel.code, paymentChannelFormFromChannel(channel)])
      )
    );
  }, [baselineKey, channels]);

  const setForm = useCallback((code: string, channelForm: PaymentChannelForm) => {
    setForms((current) => ({ ...current, [code]: channelForm }));
  }, []);

  return { forms, setForm };
}

export async function saveSystemSettingsAndPaymentChannels({
  form,
  channels,
  channelForms,
  t,
}: {
  form: SystemSettingsForm;
  channels: PaymentChannel[];
  channelForms: Record<string, PaymentChannelForm>;
  t: AdminT;
}) {
  if (!hasChangedPaymentChannels(channels, channelForms)) {
    return updateSystemSettings(settingsPayload(form));
  }
  validateChangedPaymentChannels({
    channels,
    forms: channelForms,
    publicBaseUrl: form.public_base_url,
    t,
  });
  await updateSystemSettings(settingsPayload({ ...form, recharge_enabled: false }));
  await saveChangedPaymentChannels({
    channels,
    forms: channelForms,
    publicBaseUrl: form.public_base_url,
    t,
  });
  return updateSystemSettings(settingsPayload(form));
}

export function paymentChannelsWithForms(
  channels: PaymentChannel[],
  forms: Record<string, PaymentChannelForm>
) {
  return channels.map((channel) => {
    const form = forms[channel.code];
    if (!form) {
      return channel;
    }
    return {
      ...channel,
      enabled: form.enabled,
      secret_set: channel.secret_set || Boolean(form.apiKey.trim()),
    };
  });
}

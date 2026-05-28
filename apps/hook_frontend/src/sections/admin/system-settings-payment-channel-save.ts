import type { AdminT } from './shared';
import type { PaymentChannelForm } from './system-settings-payment-channel-panel';
import type { PaymentChannel, PaymentChannelUpdateInput } from 'src/types/recharge';

import { updatePaymentChannel } from 'src/actions/recharge';

import { publicBaseUrlIsValid } from './system-settings-url-validation';

export async function saveChangedPaymentChannels(options: {
  channels: PaymentChannel[];
  forms: Record<string, PaymentChannelForm>;
  publicBaseUrl: string;
  t: AdminT;
}) {
  validateChangedPaymentChannels(options);
  for (const channel of options.channels) {
    const form = options.forms[channel.code];
    if (!form || !paymentChannelChanged(channel, form)) {
      continue;
    }
    await updatePaymentChannel(channel.code, paymentChannelPayload(form));
  }
}

export function validateChangedPaymentChannels(options: {
  channels: PaymentChannel[];
  forms: Record<string, PaymentChannelForm>;
  publicBaseUrl: string;
  t: AdminT;
}) {
  for (const channel of options.channels) {
    const form = options.forms[channel.code];
    if (!form || !paymentChannelChanged(channel, form)) {
      continue;
    }
    const validationError = validatePaymentChannelForm(form, options.publicBaseUrl, options.t);
    if (validationError) {
      throw new Error(validationError);
    }
  }
}

export function hasChangedPaymentChannels(
  channels: PaymentChannel[],
  forms: Record<string, PaymentChannelForm>
) {
  return channels.some((channel) => {
    const form = forms[channel.code];
    return form ? paymentChannelChanged(channel, form) : false;
  });
}

export function paymentChannelChanged(channel: PaymentChannel, form: PaymentChannelForm) {
  return (
    form.enabled !== channel.enabled ||
    form.merchantId.trim() !== configString(channel, 'merchant_id') ||
    form.apiBaseUrl.trim() !== configString(channel, 'api_base_url') ||
    Boolean(form.apiKey.trim())
  );
}

function validatePaymentChannelForm(form: PaymentChannelForm, publicBaseUrl: string, t: AdminT) {
  if (!form.enabled) {
    return '';
  }
  const trimmedPublicBaseUrl = publicBaseUrl.trim();
  if (!trimmedPublicBaseUrl) {
    return t('systemSettings.recharge.publicBaseUrlRequiredBeforeEnablingChannel');
  }
  if (!publicBaseUrlIsValid(trimmedPublicBaseUrl)) {
    return t('systemSettings.recharge.publicBaseUrlInvalidBeforeEnablingChannel');
  }
  return '';
}

function paymentChannelPayload(form: PaymentChannelForm): PaymentChannelUpdateInput {
  const payload: PaymentChannelUpdateInput = {
    enabled: form.enabled,
    config: {
      merchant_id: form.merchantId.trim(),
      api_base_url: form.apiBaseUrl.trim(),
    },
  };
  if (form.apiKey.trim()) {
    payload.api_key = form.apiKey.trim();
  }
  return payload;
}

function configString(channel: PaymentChannel, key: string) {
  const value = channel.config[key];
  return typeof value === 'string' ? value : '';
}

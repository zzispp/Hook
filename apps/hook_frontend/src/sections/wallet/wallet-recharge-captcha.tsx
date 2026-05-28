'use client';

import type { TFunction } from 'i18next';

import Alert from '@mui/material/Alert';

import { AuthCaptcha } from 'src/auth/components/cap-widget';

export type CaptchaState = {
  token: string | null;
  resetKey: number;
  onTokenChange: (value: string | null) => void;
  reset: VoidFunction;
};

export type RechargeCaptchaConfig = {
  enabled?: boolean;
  loading: boolean;
  errorMessage?: string;
};

export function WalletRechargeCaptcha({
  t,
  config,
  captcha,
}: {
  t: TFunction<'admin'>;
  config: RechargeCaptchaConfig;
  captcha: CaptchaState;
}) {
  return (
    <>
      <AuthCaptcha
        enabled={config.enabled === true}
        resetKey={captcha.resetKey}
        labels={adminCaptchaLabels(t)}
        onTokenChange={captcha.onTokenChange}
      />
      {config.errorMessage ? <Alert severity="error">{config.errorMessage}</Alert> : null}
    </>
  );
}

function adminCaptchaLabels(t: TFunction<'admin'>) {
  return {
    initial: t('captcha.initial'),
    verifying: t('captcha.verifying'),
    solved: t('captcha.solved'),
    error: t('captcha.error'),
  };
}

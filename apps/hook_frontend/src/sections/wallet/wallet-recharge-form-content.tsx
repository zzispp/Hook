'use client';

import type { TFunction } from 'i18next';
import type { UserRechargePackage, PublicPaymentChannel } from 'src/types/recharge';
import type { SelectorState, WalletRechargeFormProps, WalletRechargeDialogProps } from './wallet-recharge-types';

import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';

import { formatCny } from 'src/sections/recharge/recharge-display';

import { WalletRechargePackageCard } from './wallet-recharge-package-card';
import { type CaptchaState, WalletRechargeCaptcha } from './wallet-recharge-captcha';
import { WalletPaymentSelector, paymentChannelForMethod } from './wallet-payment-selector';

export function WalletRechargeFormContent({
  packages,
  channels,
  channelsLoading,
  enabled,
  t,
  captchaConfig,
  captcha,
  selector,
  amount,
  onAmountChange,
  ...props
}: WalletRechargeFormProps) {
  return (
    <>
      <WalletPaymentSelector
        t={t}
        channels={channels}
        methodCode={selector.methodCode}
        disabled={!enabled || channelsLoading}
        onMethodChange={selector.setMethodCode}
      />
      <WalletRechargeCaptcha t={t} config={captchaConfig} captcha={captcha} />
      <AmountRechargeForm
        {...props}
        t={t}
        enabled={enabled}
        channels={channels}
        captchaConfig={captchaConfig}
        amount={amount}
        selector={selector}
        captcha={captcha}
        onAmountChange={onAmountChange}
      />
      {packages.length > 0 ? (
        <>
          <Divider />
          <PackageRechargePanel
            {...props}
            t={t}
            enabled={enabled}
            packages={packages}
            channels={channels}
            channelsLoading={channelsLoading}
            captchaConfig={captchaConfig}
            selector={selector}
            captcha={captcha}
          />
        </>
      ) : null}
    </>
  );
}

function AmountRechargeForm({
  t,
  enabled,
  ratio,
  minAmount,
  maxAmount,
  channels,
  purchasingId,
  captchaConfig,
  amount,
  selector,
  captcha,
  onAmountChange,
  onPurchaseAmount,
}: Pick<
  WalletRechargeDialogProps,
  | 't'
  | 'enabled'
  | 'ratio'
  | 'minAmount'
  | 'maxAmount'
  | 'channels'
  | 'purchasingId'
  | 'captchaConfig'
  | 'onPurchaseAmount'
> & {
  amount: string;
  channels: PublicPaymentChannel[];
  selector: SelectorState;
  captcha: CaptchaState;
  onAmountChange: (value: string) => void;
}) {
  const parsedAmount = Number(amount || 0);
  const validAmount = amountIsValid(parsedAmount, minAmount, maxAmount);
  const channelCode = paymentChannelForMethod(channels, selector.methodCode);
  const paymentReady = Boolean(channelCode && selector.methodCode);
  const captchaReady = isCaptchaReady(captchaConfig, captcha.token);
  const canSubmit = enabled && validAmount && paymentReady && captchaReady;

  return (
    <Stack spacing={2}>
      <Typography variant="subtitle2">{t('wallet.recharge.customAmount')}</Typography>
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}>
        <TextField
          fullWidth
          type="number"
          label={t('wallet.recharge.customAmount')}
          value={amount}
          onChange={(event) => onAmountChange(event.target.value)}
          slotProps={{ htmlInput: { min: minAmount, max: maxAmount, step: 0.01 } }}
        />
        <TextField
          fullWidth
          disabled
          label={t('wallet.recharge.estimatedPayable')}
          value={formatCny(validAmount ? parsedAmount * ratio : 0)}
        />
      </Stack>
      <Button
        variant="contained"
        loading={purchasingId === 'custom'}
        disabled={!canSubmit || Boolean(purchasingId)}
        onClick={() => {
          onPurchaseAmount(parsedAmount, channelCode, selector.methodCode, captcha.token ?? undefined);
          captcha.reset();
        }}
        sx={{ alignSelf: { xs: 'stretch', sm: 'flex-start' } }}
      >
        {buttonText(t, paymentReady, captchaReady, canSubmit)}
      </Button>
    </Stack>
  );
}

function PackageRechargePanel(props: WalletRechargeDialogProps & { selector: SelectorState; captcha: CaptchaState }) {
  return (
    <Stack spacing={2}>
      <Typography variant="subtitle2">{props.t('wallet.recharge.packagesTitle')}</Typography>
      <Grid container spacing={2}>
        {props.packages.map((item) => (
          <Grid key={item.id} size={{ xs: 12, md: 6, lg: 4 }}>
            <PackageCard {...props} item={item} />
          </Grid>
        ))}
      </Grid>
    </Stack>
  );
}

function PackageCard({
  t,
  item,
  enabled,
  channels,
  captchaConfig,
  captcha,
  purchasingId,
  onPurchasePackage,
  selector,
}: Pick<
  WalletRechargeDialogProps,
  't' | 'enabled' | 'channels' | 'captchaConfig' | 'purchasingId' | 'onPurchasePackage'
> & {
  selector: SelectorState;
  captcha: CaptchaState;
  item: UserRechargePackage;
}) {
  const channelCode = paymentChannelForMethod(channels, selector.methodCode);
  const paymentReady = Boolean(channelCode && selector.methodCode);
  const captchaReady = isCaptchaReady(captchaConfig, captcha.token);

  return (
    <WalletRechargePackageCard
      t={t}
      item={item}
      enabled={enabled}
      paymentReady={paymentReady && captchaReady}
      waitingCaptcha={paymentReady && !captchaReady}
      purchasingId={purchasingId}
      onPurchase={(selected) => {
        onPurchasePackage(selected, channelCode, selector.methodCode, captcha.token ?? undefined);
        captcha.reset();
      }}
    />
  );
}

function isCaptchaReady(config: WalletRechargeDialogProps['captchaConfig'], token: string | null) {
  if (config.loading || config.errorMessage || config.enabled === undefined) {
    return false;
  }
  return !config.enabled || Boolean(token);
}

function amountIsValid(amount: number, minAmount: number, maxAmount: number) {
  return Number.isFinite(amount) && amount >= minAmount && amount <= maxAmount;
}

function buttonText(
  t: TFunction<'admin'>,
  paymentReady: boolean,
  captchaReady: boolean,
  canSubmit: boolean
) {
  if (canSubmit) {
    return t('wallet.recharge.submitPayment');
  }
  if (!paymentReady) {
    return t('wallet.recharge.selectPaymentFirst');
  }
  if (!captchaReady) {
    return t('wallet.recharge.completeCaptchaFirst');
  }
  return t('wallet.recharge.invalidAmount');
}

'use client';

import type { CaptchaState } from './wallet-recharge-captcha';
import type { WalletRechargeDialogProps } from './wallet-recharge-types';

import { useState, useEffect } from 'react';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { WalletRechargeFormContent } from './wallet-recharge-form-content';
import { WalletRechargePendingPanel } from './wallet-recharge-pending-panel';
import { RechargeRatioWarning, RechargeDisabledNotice } from './wallet-recharge-mode';

export function WalletRechargeDialog(props: WalletRechargeDialogProps) {
  const [methodCode, setMethodCode] = useState('');
  const [amount, setAmount] = useState(() => initialAmount(props.minAmount));
  const [captchaToken, setCaptchaToken] = useState<string | null>(null);
  const [captchaResetKey, setCaptchaResetKey] = useState(0);
  const selector = { methodCode, setMethodCode };
  const captcha: CaptchaState = {
    token: captchaToken,
    resetKey: captchaResetKey,
    onTokenChange: setCaptchaToken,
    reset: () => {
      setCaptchaToken(null);
      setCaptchaResetKey((value) => value + 1);
    },
  };

  useEffect(() => {
    if (!props.open) {
      return;
    }
    setAmount(initialAmount(props.minAmount));
    setCaptchaToken(null);
    setCaptchaResetKey((value) => value + 1);
  }, [props.open, props.minAmount]);

  return (
    <Dialog fullWidth maxWidth="md" open={props.open} onClose={props.onClose}>
      <DialogTitle>{props.t('wallet.recharge.title')}</DialogTitle>
      <DialogContent>
        <Stack spacing={3} sx={{ pt: 1 }}>
          <RechargeRatioWarning t={props.t} ratio={props.ratio} onRefresh={props.onRefresh} />
          {!props.enabled ? <RechargeDisabledNotice t={props.t} /> : null}
          {props.pendingPaymentOrderNo ? (
            <WalletRechargePendingPanel {...props} />
          ) : (
            <WalletRechargeFormContent
              {...props}
              amount={amount}
              selector={selector}
              captcha={captcha}
              onAmountChange={setAmount}
            />
          )}
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={props.onClose}>{props.t('common.close')}</Button>
      </DialogActions>
    </Dialog>
  );
}

function initialAmount(minAmount: number) {
  return String(minAmount || 0.01);
}

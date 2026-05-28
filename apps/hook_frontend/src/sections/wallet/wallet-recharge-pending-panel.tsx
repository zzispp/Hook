'use client';

import type { WalletRechargeDialogProps } from './wallet-recharge-types';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';
import CircularProgress from '@mui/material/CircularProgress';

export function WalletRechargePendingPanel({
  t,
  checkingPayment,
  pendingPaymentOrderNo,
  onCheckPayment,
}: Pick<
  WalletRechargeDialogProps,
  't' | 'checkingPayment' | 'pendingPaymentOrderNo' | 'onCheckPayment'
>) {
  return (
    <Stack spacing={2} alignItems="center" sx={{ py: 4, textAlign: 'center' }}>
      <CircularProgress size={36} />
      <Typography variant="h6">{t('wallet.recharge.paymentPendingTitle')}</Typography>
      <Typography variant="body2" color="text.secondary">
        {t('wallet.recharge.paymentPendingDescription')}
      </Typography>
      <Typography variant="caption" color="text.secondary">
        {t('wallet.recharge.orderNo')}: {pendingPaymentOrderNo}
      </Typography>
      <Button variant="contained" loading={checkingPayment} onClick={onCheckPayment}>
        {t('wallet.recharge.confirmPaid')}
      </Button>
    </Stack>
  );
}

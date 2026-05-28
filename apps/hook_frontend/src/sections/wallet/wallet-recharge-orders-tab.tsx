'use client';

import type { TFunction } from 'i18next';
import type { RechargeOrder } from 'src/types/recharge';

import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { Label } from 'src/components/label';

import {
  formatCny,
  orderStatusColor,
  formatRechargeDate,
  rechargeOrderStatusLabel,
} from 'src/sections/recharge/recharge-display';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
  orders: RechargeOrder[];
};

export function WalletRechargeOrdersTab({ t, locale, orders }: Props) {
  return (
    <Stack spacing={2} sx={{ px: 2.5, pb: 2.5 }}>
      {orders.length === 0 ? <Typography color="text.secondary">{t('wallet.recharge.emptyOrders')}</Typography> : null}
      {orders.map((order) => (
        <OrderRow key={order.id} t={t} locale={locale} order={order} />
      ))}
    </Stack>
  );
}

function OrderRow({ t, locale, order }: { t: TFunction<'admin'>; locale: string; order: RechargeOrder }) {
  return (
    <Stack spacing={1.5} sx={{ p: 2, border: (theme) => `1px solid ${theme.palette.divider}`, borderRadius: 1 }}>
      <Stack direction={{ xs: 'column', sm: 'row' }} justifyContent="space-between" spacing={1}>
        <Typography variant="subtitle2">{order.package_name}</Typography>
        <Label color={orderStatusColor(order.status)} variant="soft">
          {rechargeOrderStatusLabel(t, order.status)}
        </Label>
      </Stack>
      <Stack direction="row" spacing={1} useFlexGap flexWrap="wrap">
        <Chip size="small" label={`${t('wallet.recharge.orderNo')}: ${order.order_no}`} />
        <Chip size="small" label={`${t('wallet.recharge.estimatedPayable')}: ${formatCny(order.payable_amount)}`} />
        <Chip size="small" label={`${t('wallet.recharge.paymentMethod')}: ${order.payment_method || '-'}`} />
        <Chip size="small" label={`${t('wallet.recharge.expiresAt')}: ${formatRechargeDate(order.expires_at, locale)}`} />
      </Stack>
    </Stack>
  );
}

'use client';

import type { TFunction } from 'i18next';
import type { RechargeOrder, UserRechargePackage } from 'src/types/recharge';

import Grid from '@mui/material/Grid';
import Chip from '@mui/material/Chip';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { EmptyContent } from 'src/components/empty-content';

import {
  formatCny,
  formatUsd,
  orderStatusColor,
  formatRechargeDate,
  rechargeOrderStatusLabel,
} from 'src/sections/recharge/recharge-display';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
  loading: boolean;
  enabled: boolean;
  ratio: number;
  packages: UserRechargePackage[];
  orders: RechargeOrder[];
  purchasingId: string | null;
  onPurchase: (item: UserRechargePackage) => void;
  onRefresh: VoidFunction;
};

export function WalletRechargePanel(props: Props) {
  const hasPackages = props.packages.length > 0;

  return (
    <Stack spacing={3} sx={{ px: 2.5, pb: 2.5 }}>
      <RechargeNotice {...props} />
      {hasPackages ? <PackageGrid {...props} /> : <PackageEmpty loading={props.loading} t={props.t} />}
      <OrderPreview t={props.t} locale={props.locale} orders={props.orders} />
    </Stack>
  );
}

function RechargeNotice({ t, enabled, ratio, onRefresh }: Pick<Props, 't' | 'enabled' | 'ratio' | 'onRefresh'>) {
  return (
    <Alert
      severity={enabled ? 'info' : 'warning'}
      action={
        <Button color="inherit" size="small" startIcon={<Iconify icon="solar:restart-bold" />} onClick={onRefresh}>
          {t('wallet.recharge.refresh')}
        </Button>
      }
    >
      <Stack spacing={0.5}>
        <Typography variant="body2">{enabled ? t('wallet.recharge.description') : t('wallet.recharge.disabled')}</Typography>
        <Typography variant="caption" sx={{ color: 'text.secondary' }}>
          {t('wallet.recharge.ratio', { ratio })}
        </Typography>
      </Stack>
    </Alert>
  );
}

function PackageGrid(props: Props) {
  return (
    <Grid container spacing={2}>
      {props.packages.map((item) => (
        <Grid key={item.id} size={{ xs: 12, md: 6, lg: 4 }}>
          <PackageCard {...props} item={item} />
        </Grid>
      ))}
    </Grid>
  );
}

function PackageCard({
  t,
  item,
  enabled,
  purchasingId,
  onPurchase,
}: Pick<Props, 't' | 'enabled' | 'purchasingId' | 'onPurchase'> & {
  item: UserRechargePackage;
}) {
  return (
    <Stack sx={{ height: 1, p: 2, border: (theme) => `1px solid ${theme.palette.divider}`, borderRadius: 1 }} spacing={2}>
      <Stack spacing={0.5}>
        <Typography variant="subtitle1">{item.name}</Typography>
        {item.description ? (
          <Typography variant="body2" color="text.secondary">
            {item.description}
          </Typography>
        ) : null}
      </Stack>
      <Divider />
      <Stack spacing={1}>
        <AmountRow label={t('wallet.recharge.rechargeAmount')} value={formatUsd(item.recharge_amount)} />
        <AmountRow label={t('wallet.recharge.giftAmount')} value={formatUsd(item.gift_amount)} />
        <AmountRow label={t('wallet.recharge.totalArrival')} value={formatUsd(item.total_arrival_amount)} strong />
        <AmountRow label={t('wallet.recharge.estimatedPayable')} value={formatCny(item.estimated_payable_amount)} strong />
      </Stack>
      <Button
        fullWidth
        variant="contained"
        loading={purchasingId === item.id}
        disabled={!enabled || Boolean(purchasingId)}
        startIcon={<Iconify icon="solar:cart-plus-bold" />}
        onClick={() => onPurchase(item)}
      >
        {t('wallet.recharge.buy')}
      </Button>
    </Stack>
  );
}

function AmountRow({ label, value, strong = false }: { label: string; value: string; strong?: boolean }) {
  return (
    <Stack direction="row" justifyContent="space-between" spacing={2}>
      <Typography variant="body2" color="text.secondary">
        {label}
      </Typography>
      <Typography variant="body2" sx={{ fontWeight: strong ? 700 : 400 }}>
        {value}
      </Typography>
    </Stack>
  );
}

function PackageEmpty({ t, loading }: { t: TFunction<'admin'>; loading: boolean }) {
  if (loading) {
    return <Typography color="text.secondary">{t('common.loading')}</Typography>;
  }

  return <EmptyContent filled title={t('wallet.recharge.emptyPackages')} sx={{ py: 6 }} />;
}

function OrderPreview({ t, locale, orders }: { t: TFunction<'admin'>; locale: string; orders: RechargeOrder[] }) {
  return (
    <Stack spacing={2}>
      <Typography variant="h6">{t('wallet.recharge.ordersTitle')}</Typography>
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
        <Chip size="small" label={`${t('wallet.recharge.expiresAt')}: ${formatRechargeDate(order.expires_at, locale)}`} />
      </Stack>
      {order.status === 'pending' ? <Alert severity="warning">{t('wallet.recharge.paymentUnavailable')}</Alert> : null}
    </Stack>
  );
}

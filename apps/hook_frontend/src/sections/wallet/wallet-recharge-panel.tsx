'use client';

import type { TFunction } from 'i18next';
import type { UserRechargePackage } from 'src/types/recharge';

import { useState } from 'react';

import Tab from '@mui/material/Tab';
import Grid from '@mui/material/Grid';
import Tabs from '@mui/material/Tabs';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';

import { Iconify } from 'src/components/iconify';
import { EmptyContent } from 'src/components/empty-content';

import { formatCny, formatUsd } from 'src/sections/recharge/recharge-display';

type Props = {
  t: TFunction<'admin'>;
  loading: boolean;
  enabled: boolean;
  ratio: number;
  packages: UserRechargePackage[];
  purchasingId: string | null;
  onPurchase: (item: UserRechargePackage) => void;
  onRefresh: VoidFunction;
};

type RechargeMode = 'amount' | 'package';

export function WalletRechargePanel(props: Props) {
  const [mode, setMode] = useState<RechargeMode>('amount');

  return (
    <Stack spacing={3} sx={{ p: 2.5 }}>
      <RechargeHeader t={props.t} mode={mode} onModeChange={setMode} />
      <RechargeRatioWarning t={props.t} ratio={props.ratio} onRefresh={props.onRefresh} />
      {!props.enabled ? <RechargeDisabledNotice t={props.t} /> : null}
      {mode === 'amount' ? <AmountRechargePanel {...props} /> : <PackageRechargePanel {...props} />}
    </Stack>
  );
}

function RechargeHeader({
  t,
  mode,
  onModeChange,
}: {
  t: TFunction<'admin'>;
  mode: RechargeMode;
  onModeChange: (mode: RechargeMode) => void;
}) {
  return (
    <Stack spacing={1.5}>
      <Typography variant="h6">{t('wallet.recharge.title')}</Typography>
      <Tabs value={mode} onChange={(_, value: RechargeMode) => onModeChange(value)}>
        <Tab value="amount" label={t('wallet.recharge.amountMode')} />
        <Tab value="package" label={t('wallet.recharge.packageMode')} />
      </Tabs>
    </Stack>
  );
}

function RechargeRatioWarning({ t, ratio, onRefresh }: Pick<Props, 't' | 'ratio' | 'onRefresh'>) {
  return (
    <Alert
      severity="warning"
      action={
        <Button color="inherit" size="small" startIcon={<Iconify icon="solar:restart-bold" />} onClick={onRefresh}>
          {t('wallet.recharge.refresh')}
        </Button>
      }
    >
      {t('wallet.recharge.ratio', { ratio })}
    </Alert>
  );
}

function RechargeDisabledNotice({ t }: Pick<Props, 't'>) {
  return <Alert severity="warning">{t('wallet.recharge.disabled')}</Alert>;
}

function AmountRechargePanel({ t, enabled, ratio }: Pick<Props, 't' | 'enabled' | 'ratio'>) {
  return (
    <Stack spacing={2}>
      <Alert severity="info">{t('wallet.recharge.customRechargeUnavailable')}</Alert>
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}>
        <TextField
          fullWidth
          disabled
          type="number"
          label={t('wallet.recharge.customAmount')}
          value=""
          placeholder={t('wallet.recharge.customAmountPlaceholder')}
        />
        <TextField fullWidth disabled label={t('wallet.recharge.estimatedPayable')} value={formatCny(0 * ratio)} />
      </Stack>
      <Button
        variant="contained"
        disabled
        startIcon={<Iconify icon="solar:wad-of-money-bold" />}
        sx={{ alignSelf: { xs: 'stretch', sm: 'flex-start' } }}
      >
        {enabled ? t('wallet.recharge.paymentUnavailableAction') : t('wallet.recharge.disabled')}
      </Button>
    </Stack>
  );
}

function PackageRechargePanel(props: Props) {
  const hasPackages = props.packages.length > 0;

  return hasPackages ? <PackageGrid {...props} /> : <PackageEmpty loading={props.loading} t={props.t} />;
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

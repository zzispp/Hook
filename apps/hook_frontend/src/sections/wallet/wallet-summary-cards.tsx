'use client';

import type { TFunction } from 'i18next';
import type { WalletSummary } from 'src/types/wallet';
import type { IconifyProps } from 'src/components/iconify';

import Card from '@mui/material/Card';
import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { Iconify } from 'src/components/iconify';

import { formatWalletMoney } from './wallet-display';

type WalletMetric = {
  label: string;
  value: string;
  icon: IconifyProps['icon'];
};

export function WalletSummaryCards({
  t,
  wallet,
}: {
  t: TFunction<'admin'>;
  wallet?: WalletSummary;
}) {
  const metrics = walletMetrics(t, wallet);

  return (
    <Grid container spacing={3} sx={{ mb: 3 }}>
      {metrics.map((metric) => (
        <Grid key={metric.label} size={{ xs: 12, sm: 6, md: 3 }}>
          <WalletSummaryCard metric={metric} />
        </Grid>
      ))}
    </Grid>
  );
}

function WalletSummaryCard({ metric }: { metric: WalletMetric }) {
  return (
    <Card sx={{ p: 2.5 }}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={2}>
        <Stack spacing={0.5}>
          <Typography variant="overline" sx={{ color: 'text.secondary' }}>
            {metric.label}
          </Typography>
          <Typography variant="h5">{metric.value}</Typography>
        </Stack>
        <Iconify icon={metric.icon} width={28} sx={{ color: 'primary.main' }} />
      </Stack>
    </Card>
  );
}

function walletMetrics(t: TFunction<'admin'>, wallet?: WalletSummary): WalletMetric[] {
  return [
    {
      label: t('wallet.metrics.availableBalance'),
      value: formatWalletMoney(wallet?.balance),
      icon: 'solar:wad-of-money-bold',
    },
    {
      label: t('wallet.metrics.rechargeBalance'),
      value: formatWalletMoney(wallet?.recharge_balance),
      icon: 'solar:bill-list-bold',
    },
    {
      label: t('wallet.metrics.giftBalance'),
      value: formatWalletMoney(wallet?.gift_balance),
      icon: 'solar:cup-star-bold',
    },
    {
      label: t('wallet.metrics.totalConsumed'),
      value: formatWalletMoney(wallet?.total_consumed),
      icon: 'solar:cart-3-bold',
    },
  ];
}

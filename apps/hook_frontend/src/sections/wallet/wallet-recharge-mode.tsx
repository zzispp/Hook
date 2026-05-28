'use client';

import type { TFunction } from 'i18next';

import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';

import { Iconify } from 'src/components/iconify';

type HeaderProps = {
  t: TFunction<'admin'>;
};

export function RechargeHeader({ t }: HeaderProps) {
  return (
    <Stack spacing={1.5}>
      <Typography variant="h6">{t('wallet.recharge.title')}</Typography>
    </Stack>
  );
}

export function RechargeRatioWarning({
  t,
  ratio,
  onRefresh,
}: {
  t: TFunction<'admin'>;
  ratio: number;
  onRefresh: VoidFunction;
}) {
  return (
    <Alert
      severity="warning"
      action={
        <Button
          color="inherit"
          size="small"
          startIcon={<Iconify icon="solar:restart-bold" />}
          onClick={onRefresh}
        >
          {t('wallet.recharge.refresh')}
        </Button>
      }
    >
      {t('wallet.recharge.ratio', { ratio })}
    </Alert>
  );
}

export function RechargeDisabledNotice({ t }: { t: TFunction<'admin'> }) {
  return <Alert severity="warning">{t('wallet.recharge.disabled')}</Alert>;
}

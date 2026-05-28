'use client';

import type { TFunction } from 'i18next';
import type { UserRechargePackage } from 'src/types/recharge';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';

import { Iconify } from 'src/components/iconify';

import { formatCny, formatUsd } from 'src/sections/recharge/recharge-display';

type Props = {
  t: TFunction<'admin'>;
  item: UserRechargePackage;
  enabled: boolean;
  paymentReady: boolean;
  waitingCaptcha: boolean;
  purchasingId: string | null;
  onPurchase: (item: UserRechargePackage) => void;
};

export function WalletRechargePackageCard(props: Props) {
  const canPurchase = props.enabled && props.paymentReady;

  return (
    <Stack sx={cardSx} spacing={1.75}>
      <PackageTitle item={props.item} />
      <PayableAmount t={props.t} item={props.item} />
      <PackageAmounts t={props.t} item={props.item} />
      <Divider />
      <Button
        fullWidth
        variant="contained"
        loading={props.purchasingId === props.item.id}
        disabled={!canPurchase || Boolean(props.purchasingId)}
        startIcon={<Iconify icon="solar:cart-plus-bold" />}
        onClick={() => props.onPurchase(props.item)}
        sx={{ mt: 'auto' }}
      >
        {canPurchase
          ? props.t('wallet.recharge.submitPayment')
          : props.t(
              props.waitingCaptcha
                ? 'wallet.recharge.completeCaptchaFirst'
                : 'wallet.recharge.selectPaymentFirst'
            )}
      </Button>
    </Stack>
  );
}

function PackageTitle({ item }: { item: UserRechargePackage }) {
  return (
    <Stack spacing={0.5}>
      <Typography variant="subtitle1" sx={titleSx}>
        {item.name}
      </Typography>
      {item.description ? (
        <Typography variant="body2" color="text.secondary" sx={descriptionSx}>
          {item.description}
        </Typography>
      ) : null}
    </Stack>
  );
}

function PayableAmount({ t, item }: Pick<Props, 't' | 'item'>) {
  return (
    <Box sx={payableSx}>
      <Typography variant="caption" color="text.secondary">
        {t('wallet.recharge.estimatedPayable')}
      </Typography>
      <Typography variant="h5" sx={amountTextSx}>
        {formatCny(item.estimated_payable_amount)}
      </Typography>
    </Box>
  );
}

function PackageAmounts({ t, item }: Pick<Props, 't' | 'item'>) {
  return (
    <Stack spacing={0.75} sx={{ flexGrow: 1 }}>
      <AmountRow label={t('wallet.recharge.rechargeAmount')} value={formatUsd(item.recharge_amount)} />
      <AmountRow label={t('wallet.recharge.giftAmount')} value={formatUsd(item.gift_amount)} />
      <AmountRow label={t('wallet.recharge.totalArrival')} value={formatUsd(item.total_arrival_amount)} strong />
    </Stack>
  );
}

function AmountRow({ label, value, strong = false }: { label: string; value: string; strong?: boolean }) {
  return (
    <Stack direction="row" justifyContent="space-between" spacing={2} sx={{ minWidth: 0 }}>
      <Typography variant="body2" color="text.secondary">
        {label}
      </Typography>
      <Typography variant="body2" sx={{ ...amountTextSx, fontWeight: strong ? 700 : 400 }}>
        {value}
      </Typography>
    </Stack>
  );
}

const cardSx = {
  height: 1,
  p: 2,
  border: (theme: { palette: { divider: string } }) => `1px solid ${theme.palette.divider}`,
  borderRadius: 1,
};

const titleSx = {
  overflowWrap: 'anywhere',
};

const descriptionSx = {
  overflowWrap: 'anywhere',
};

const payableSx = {
  p: 1.5,
  borderRadius: 1,
  bgcolor: 'background.neutral',
};

const amountTextSx = {
  overflowWrap: 'anywhere',
  textAlign: 'right',
};

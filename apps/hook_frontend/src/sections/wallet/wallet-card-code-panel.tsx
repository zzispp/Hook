'use client';

import type { TFunction } from 'i18next';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';

import { Iconify } from 'src/components/iconify';

type Props = {
  t: TFunction<'admin'>;
  code: string;
  redeeming: boolean;
  onCodeChange: (value: string) => void;
  onRedeem: VoidFunction;
};

export function WalletCardCodePanel({ t, code, redeeming, onCodeChange, onRedeem }: Props) {
  return (
    <Card sx={{ height: 1, p: 2.5 }}>
      <Stack spacing={2}>
        <Typography variant="h6">{t('wallet.cardCode.title')}</Typography>
        <TextField
          fullWidth
          label={t('wallet.cardCode.code')}
          value={code}
          onChange={(event) => onCodeChange(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Enter') {
              event.preventDefault();
              onRedeem();
            }
          }}
        />
        <Button
          fullWidth
          variant="contained"
          loading={redeeming}
          disabled={!code.trim()}
          startIcon={<Iconify icon="solar:bill-list-bold" />}
          onClick={onRedeem}
        >
          {t('wallet.cardCode.redeem')}
        </Button>
      </Stack>
    </Card>
  );
}

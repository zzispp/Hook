'use client';

import type { TFunction } from 'i18next';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import TextField from '@mui/material/TextField';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { Iconify } from 'src/components/iconify';

type Props = {
  t: TFunction<'admin'>;
  open: boolean;
  code: string;
  redeeming: boolean;
  onClose: VoidFunction;
  onCodeChange: (value: string) => void;
  onRedeem: VoidFunction;
};

export function WalletCardCodeDialog({ t, open, code, redeeming, onClose, onCodeChange, onRedeem }: Props) {
  return (
    <Dialog fullWidth maxWidth="xs" open={open} onClose={onClose}>
      <DialogTitle>{t('wallet.cardCode.title')}</DialogTitle>
      <DialogContent>
        <Stack spacing={2} sx={{ pt: 1 }}>
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
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>{t('common.cancel')}</Button>
        <Button
          variant="contained"
          loading={redeeming}
          disabled={!code.trim()}
          startIcon={<Iconify icon="solar:bill-list-bold" />}
          onClick={onRedeem}
        >
          {t('wallet.cardCode.redeem')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

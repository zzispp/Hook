'use client';

import type { TFunction } from 'i18next';
import type { AdminWallet } from 'src/types/wallet';

import { useState, useEffect } from 'react';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { labelWithAccountingCurrency } from 'src/utils/money-boundary';

import { adminWalletOwner } from './wallet-display';

type Props = {
  t: TFunction<'admin'>;
  open: boolean;
  wallet: AdminWallet | null;
  submitting: boolean;
  onClose: VoidFunction;
  onSubmit: (input: AdminWalletAdjustmentDraft) => void;
};

type AdminWalletAdjustmentDraft = {
  amount: number;
  balance_type: BalanceType;
  adjustment_type: AdjustmentType;
  description?: string;
};

type AdjustmentType = 'increase' | 'deduct';
type BalanceType = 'recharge' | 'gift';

export function AdminWalletAdjustDialog({ t, open, wallet, submitting, onClose, onSubmit }: Props) {
  const [amount, setAmount] = useState('');
  const [balanceType, setBalanceType] = useState<BalanceType>('recharge');
  const [adjustmentType, setAdjustmentType] = useState<AdjustmentType>('increase');
  const [description, setDescription] = useState('');

  useEffect(() => {
    if (!open) {
      setAmount('');
      setBalanceType('recharge');
      setAdjustmentType('increase');
      setDescription('');
    }
  }, [open]);

  return (
    <Dialog fullWidth maxWidth="sm" open={open} onClose={onClose}>
      <DialogTitle>{t('adminWallets.adjust.title')}</DialogTitle>
      <DialogContent>
        <AdjustFields
          t={t}
          wallet={wallet}
          amount={amount}
          balanceType={balanceType}
          adjustmentType={adjustmentType}
          description={description}
          setAmount={setAmount}
          setBalanceType={setBalanceType}
          setAdjustmentType={setAdjustmentType}
          setDescription={setDescription}
        />
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button
          variant="contained"
          loading={submitting}
          onClick={() => submit({ amount, balanceType, adjustmentType, description, onSubmit })}
        >
          {t('common.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function AdjustFields({
  t,
  wallet,
  amount,
  balanceType,
  adjustmentType,
  description,
  setAmount,
  setBalanceType,
  setAdjustmentType,
  setDescription,
}: {
  t: TFunction<'admin'>;
  wallet: AdminWallet | null;
  amount: string;
  balanceType: BalanceType;
  adjustmentType: AdjustmentType;
  description: string;
  setAmount: (value: string) => void;
  setBalanceType: (value: BalanceType) => void;
  setAdjustmentType: (value: AdjustmentType) => void;
  setDescription: (value: string) => void;
}) {
  return (
    <Stack spacing={2.5} sx={{ pt: 1 }}>
      <TextField label={t('adminWallets.fields.owner')} value={wallet ? adminWalletOwner(wallet) : ''} disabled />
      <AdjustmentTypeSelect t={t} value={adjustmentType} onChange={setAdjustmentType} />
      <AmountInput t={t} value={amount} onChange={setAmount} />
      <BalanceTypeSelect t={t} value={balanceType} onChange={setBalanceType} />
      <DescriptionInput t={t} value={description} onChange={setDescription} />
    </Stack>
  );
}

function AdjustmentTypeSelect({
  t,
  value,
  onChange,
}: {
  t: TFunction<'admin'>;
  value: AdjustmentType;
  onChange: (value: AdjustmentType) => void;
}) {
  return (
    <TextField select label={t('adminWallets.adjust.adjustmentType')} value={value} onChange={(event) => onChange(event.target.value as AdjustmentType)}>
      <MenuItem value="increase">{t('adminWallets.adjust.types.increase')}</MenuItem>
      <MenuItem value="deduct">{t('adminWallets.adjust.types.deduct')}</MenuItem>
    </TextField>
  );
}

function AmountInput({ t, value, onChange }: { t: TFunction<'admin'>; value: string; onChange: (value: string) => void }) {
  return (
    <TextField
      required
      type="number"
      label={labelWithAccountingCurrency(t('adminWallets.adjust.amount'))}
      value={value}
      slotProps={{ htmlInput: { min: 0, step: '0.0001' } }}
      onChange={(event) => onChange(event.target.value)}
    />
  );
}

function BalanceTypeSelect({
  t,
  value,
  onChange,
}: {
  t: TFunction<'admin'>;
  value: BalanceType;
  onChange: (value: BalanceType) => void;
}) {
  return (
    <TextField select label={t('adminWallets.adjust.balanceType')} value={value} onChange={(event) => onChange(event.target.value as BalanceType)}>
      <MenuItem value="recharge">{t('wallet.balanceTypeLabels.recharge')}</MenuItem>
      <MenuItem value="gift">{t('wallet.balanceTypeLabels.gift')}</MenuItem>
    </TextField>
  );
}

function DescriptionInput({
  t,
  value,
  onChange,
}: {
  t: TFunction<'admin'>;
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <TextField
      multiline
      minRows={3}
      label={t('wallet.fields.description')}
      value={value}
      onChange={(event) => onChange(event.target.value)}
    />
  );
}

function submit({
  amount,
  balanceType,
  adjustmentType,
  description,
  onSubmit,
}: {
  amount: string;
  balanceType: BalanceType;
  adjustmentType: AdjustmentType;
  description: string;
  onSubmit: Props['onSubmit'];
}) {
  const value = Number(amount);
  if (!Number.isFinite(value) || value <= 0) {
    return;
  }
  onSubmit({
    amount: value,
    balance_type: balanceType,
    adjustment_type: adjustmentType,
    description: description.trim() || undefined,
  });
}

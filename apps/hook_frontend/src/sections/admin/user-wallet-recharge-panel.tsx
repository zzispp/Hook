'use client';

import { useState, useCallback } from 'react';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/locales/use-locales';
import { rechargeAdminWallet } from 'src/actions/wallet';

import { toast } from 'src/components/snackbar';

export type ManualRechargeForm = ReturnType<typeof useManualRechargeForm>;

type RechargeFormOptions = {
  walletId: string | null;
  onChanged: VoidFunction;
  refreshLedger: VoidFunction;
  refreshBalance: VoidFunction;
};

export function ManualRechargePanel({ form }: { form: ManualRechargeForm }) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={2.5} sx={{ maxWidth: 560 }}>
      <TextField label={t('userWallet.operationType')} value={t('userWallet.manualRecharge')} disabled />
      <TextField
        required
        type="number"
        label={t('userWallet.amount')}
        value={form.amount}
        helperText={t('userWallet.positiveAmount')}
        slotProps={{ htmlInput: { min: 0, step: '0.01' } }}
        onChange={(event) => form.setAmount(event.target.value)}
      />
      <TextField
        multiline
        minRows={3}
        label={t('userWallet.description')}
        value={form.description}
        helperText={t('userWallet.rechargeDescription')}
        onChange={(event) => form.setDescription(event.target.value)}
      />
      <Stack direction="row" justifyContent="flex-start">
        <Button variant="contained" loading={form.submitting} onClick={form.submit}>
          {t('common.save')}
        </Button>
      </Stack>
    </Stack>
  );
}

export function useManualRechargeForm(options: RechargeFormOptions) {
  const { t } = useTranslate('admin');
  const [amount, setAmount] = useState('');
  const [description, setDescription] = useState('');
  const [submitting, setSubmitting] = useState(false);

  const submit = useCallback(async () => {
    const value = Number(amount);
    if (!options.walletId || !Number.isFinite(value) || value <= 0) {
      toast.error(t('userWallet.invalidAmount'));
      return;
    }
    setSubmitting(true);
    try {
      await rechargeAdminWallet(options.walletId, {
        amount: value,
        description: description.trim() || undefined,
      });
      toast.success(t('userWallet.rechargeSaved'));
      setAmount('');
      setDescription('');
      options.refreshBalance();
      options.refreshLedger();
      options.onChanged();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [amount, description, options, t]);

  return { amount, description, setAmount, setDescription, submit, submitting };
}

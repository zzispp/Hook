'use client';

import type { TFunction } from 'i18next';
import type {
  RechargePackage,
  RechargePackageInput,
  RechargePackageStatus,
} from 'src/types/recharge';

import { useState, useEffect } from 'react';

import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { ManagementDialog } from 'src/sections/admin/shared';

import { formatCny, formatUsd, estimatedPayableAmount } from './recharge-display';

const MIN_RECHARGE_AMOUNT = 0.01;
const MIN_GIFT_AMOUNT = 0;
const NUMBER_STEP = 0.01;

type Form = {
  name: string;
  description: string;
  recharge_amount: number;
  gift_amount: number;
  status: RechargePackageStatus;
  sort_order: number;
};

type Props = {
  t: TFunction<'admin'>;
  open: boolean;
  item: RechargePackage | null;
  ratio: number;
  submitting: boolean;
  onClose: VoidFunction;
  onSubmit: (input: RechargePackageInput) => void;
};

export function RechargePackageDialog({ t, open, item, ratio, submitting, onClose, onSubmit }: Props) {
  const [form, setForm] = useState<Form>(() => formFromPackage(item));

  useEffect(() => {
    if (open) setForm(formFromPackage(item));
  }, [item, open]);

  const patch = (next: Partial<Form>) => setForm((current) => ({ ...current, ...next }));

  return (
    <ManagementDialog
      open={open}
      title={item ? t('adminRecharges.dialogs.editPackageTitle') : t('adminRecharges.dialogs.createPackageTitle')}
      submitting={submitting}
      submitDisabled={!form.name.trim()}
      onClose={onClose}
      onSubmit={() => onSubmit(payloadFromForm(form))}
    >
      <TextField
        required
        label={t('adminRecharges.fields.packageName')}
        value={form.name}
        onChange={(event) => patch({ name: event.target.value })}
      />
      <TextField
        multiline
        minRows={3}
        label={t('adminRecharges.fields.description')}
        value={form.description}
        onChange={(event) => patch({ description: event.target.value })}
      />
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}>
        <AmountField
          label={t('adminRecharges.fields.rechargeAmount')}
          value={form.recharge_amount}
          min={MIN_RECHARGE_AMOUNT}
          onChange={(recharge_amount) => patch({ recharge_amount })}
        />
        <AmountField
          label={t('adminRecharges.fields.giftAmount')}
          value={form.gift_amount}
          min={MIN_GIFT_AMOUNT}
          onChange={(gift_amount) => patch({ gift_amount })}
        />
      </Stack>
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}>
        <TextField
          fullWidth
          label={t('adminRecharges.fields.totalArrival')}
          value={formatUsd(form.recharge_amount + form.gift_amount)}
          disabled
        />
        <TextField
          fullWidth
          label={t('adminRecharges.fields.estimatedPayable')}
          value={formatCny(estimatedPayableAmount(form.recharge_amount, ratio))}
          disabled
        />
      </Stack>
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}>
        <TextField
          select
          fullWidth
          label={t('common.status')}
          value={form.status}
          onChange={(event) => patch({ status: event.target.value as RechargePackageStatus })}
        >
          <MenuItem value="active">{t('adminRecharges.status.package.active')}</MenuItem>
          <MenuItem value="disabled">{t('adminRecharges.status.package.disabled')}</MenuItem>
        </TextField>
        <TextField
          fullWidth
          type="number"
          label={t('common.sortOrder')}
          value={form.sort_order}
          onChange={(event) => patch({ sort_order: Number(event.target.value) })}
          slotProps={{ htmlInput: { step: 1 } }}
        />
      </Stack>
    </ManagementDialog>
  );
}

function AmountField({
  label,
  value,
  min,
  onChange,
}: {
  label: string;
  value: number;
  min: number;
  onChange: (value: number) => void;
}) {
  return (
    <TextField
      fullWidth
      type="number"
      label={label}
      value={value}
      onChange={(event) => onChange(Number(event.target.value))}
      slotProps={{ htmlInput: { min, step: NUMBER_STEP } }}
    />
  );
}

function formFromPackage(item: RechargePackage | null): Form {
  return {
    name: item?.name ?? '',
    description: item?.description ?? '',
    recharge_amount: item?.recharge_amount ?? MIN_RECHARGE_AMOUNT,
    gift_amount: item?.gift_amount ?? MIN_GIFT_AMOUNT,
    status: item?.status ?? 'active',
    sort_order: item?.sort_order ?? 0,
  };
}

function payloadFromForm(form: Form): RechargePackageInput {
  return {
    name: form.name.trim(),
    description: form.description.trim() || undefined,
    recharge_amount: form.recharge_amount,
    gift_amount: form.gift_amount,
    status: form.status,
    sort_order: Math.trunc(form.sort_order),
  };
}

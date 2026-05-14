'use client';

import type { TFunction } from 'i18next';
import type {
  CardCodeType,
  CardCodeTypeInput,
  CardCodeBalanceType,
  CardCodeGenerateInput,
} from 'src/types/card-code';

import { useState, useEffect } from 'react';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

const MIN_NUMBER_VALUE = 0;
const NUMBER_STEP = 1;
const DEFAULT_GENERATE_QUANTITY = 1;
const DEFAULT_CODE_LENGTH = 12;
const DEFAULT_GENERATE_AMOUNT = 1;

type GenerateDialogProps = {
  t: TFunction<'admin'>;
  open: boolean;
  types: CardCodeType[];
  submitting: boolean;
  onClose: VoidFunction;
  onSubmit: (input: CardCodeGenerateInput) => void;
};

type TypeDialogProps = {
  t: TFunction<'admin'>;
  open: boolean;
  item: CardCodeType | null;
  submitting: boolean;
  onClose: VoidFunction;
  onSubmit: (input: CardCodeTypeInput) => void;
};

type GenerateForm = {
  type_id: string;
  quantity: number;
  code_length: number;
  status: 'active' | 'disabled';
  amount: number;
  expires_at: string;
  remark: string;
};

type TypeForm = {
  name: string;
  balance_type: CardCodeBalanceType;
  status: 'active' | 'disabled';
  remark: string;
};

export function CardCodeGenerateDialog({ t, open, types, submitting, onClose, onSubmit }: GenerateDialogProps) {
  const [form, setForm] = useState<GenerateForm>(() => defaultGenerateForm(types));

  useEffect(() => {
    if (open) {
      setForm(defaultGenerateForm(types));
    }
  }, [open, types]);

  const selectedType = types.find((item) => item.id === form.type_id) ?? null;
  const patch = (next: Partial<GenerateForm>) => setForm((current) => ({ ...current, ...next }));
  const submit = () => onSubmit(generatePayload(form));
  const amountLabel = cardCodeAmountLabel(t, selectedType);

  return (
    <Dialog open={open} fullWidth maxWidth="sm" onClose={onClose}>
      <DialogTitle>{t('adminCardCodes.dialogs.generateTitle')}</DialogTitle>
      <DialogContent>
        <Stack spacing={2.5} sx={{ pt: 1 }}>
          <TextField select label={t('adminCardCodes.fields.type')} value={form.type_id} onChange={(event) => patch(typePatch(event.target.value))}>
            {types.length === 0 ? <MenuItem disabled value="">{t('adminCardCodes.empty.availableTypes')}</MenuItem> : null}
            {types.map((type) => (
              <MenuItem key={type.id} value={type.id}>
                {type.name}
              </MenuItem>
            ))}
          </TextField>
          <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}>
            <NumberField label={t('adminCardCodes.fields.quantity')} value={form.quantity} onChange={(quantity) => patch({ quantity })} />
            <NumberField label={t('adminCardCodes.fields.codeLength')} value={form.code_length} onChange={(code_length) => patch({ code_length })} />
          </Stack>
          <NumberField label={amountLabel} value={form.amount} onChange={(amount) => patch({ amount })} />
          <TextField select label={t('common.status')} value={form.status} onChange={(event) => patch({ status: event.target.value as GenerateForm['status'] })}>
            <MenuItem value="active">{t('adminCardCodes.status.active')}</MenuItem>
            <MenuItem value="disabled">{t('adminCardCodes.status.disabled')}</MenuItem>
          </TextField>
          <TextField label={t('adminCardCodes.fields.expiresAt')} type="datetime-local" value={form.expires_at} onChange={(event) => patch({ expires_at: event.target.value })} InputLabelProps={{ shrink: true }} />
          <TextField multiline minRows={3} label={t('adminCardCodes.fields.remark')} value={form.remark} onChange={(event) => patch({ remark: event.target.value })} />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>{t('common.cancel')}</Button>
        <Button variant="contained" loading={submitting} disabled={!selectedType} onClick={submit}>
          {t('adminCardCodes.actions.generate')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

export function CardCodeTypeDialog({ t, open, item, submitting, onClose, onSubmit }: TypeDialogProps) {
  const [form, setForm] = useState<TypeForm>(() => typeForm(item));

  useEffect(() => {
    if (open) {
      setForm(typeForm(item));
    }
  }, [item, open]);

  const patch = (next: Partial<TypeForm>) => setForm((current) => ({ ...current, ...next }));

  return (
    <Dialog open={open} fullWidth maxWidth="sm" onClose={onClose}>
      <DialogTitle>{item ? t('adminCardCodes.dialogs.editTypeTitle') : t('adminCardCodes.dialogs.createTypeTitle')}</DialogTitle>
      <DialogContent>
        <Stack spacing={2.5} sx={{ pt: 1 }}>
          <TextField label={t('adminCardCodes.fields.typeName')} value={form.name} onChange={(event) => patch({ name: event.target.value })} />
          <TextField
            select
            label={t('adminCardCodes.fields.balanceType')}
            value={form.balance_type}
            SelectProps={balanceTypeSelectProps(t)}
            onChange={(event) => patch({ balance_type: event.target.value as CardCodeBalanceType })}
          >
            <BalanceTypeOptions t={t} />
          </TextField>
          <TextField select label={t('common.status')} value={form.status} onChange={(event) => patch({ status: event.target.value as TypeForm['status'] })}>
            <MenuItem value="active">{t('adminCardCodes.status.active')}</MenuItem>
            <MenuItem value="disabled">{t('adminCardCodes.status.disabled')}</MenuItem>
          </TextField>
          <TextField multiline minRows={3} label={t('adminCardCodes.fields.remark')} value={form.remark} onChange={(event) => patch({ remark: event.target.value })} />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>{t('common.cancel')}</Button>
        <Button variant="contained" loading={submitting} onClick={() => onSubmit(typePayload(form))}>
          {t('common.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function NumberField({ label, value, onChange }: { label: string; value: number; onChange: (value: number) => void }) {
  return (
    <TextField
      fullWidth
      type="number"
      label={label}
      value={value}
      onChange={(event) => onChange(Number(event.target.value))}
      slotProps={{ htmlInput: { min: MIN_NUMBER_VALUE, step: NUMBER_STEP } }}
    />
  );
}

function defaultGenerateForm(types: CardCodeType[]): GenerateForm {
  const type = types[0];
  return {
    type_id: type?.id ?? '',
    quantity: DEFAULT_GENERATE_QUANTITY,
    code_length: DEFAULT_CODE_LENGTH,
    status: 'active',
    amount: DEFAULT_GENERATE_AMOUNT,
    expires_at: '',
    remark: '',
  };
}

function typePatch(typeId: string): Partial<GenerateForm> {
  return {
    type_id: typeId,
  };
}

function generatePayload(form: GenerateForm): CardCodeGenerateInput {
  return {
    type_id: form.type_id,
    quantity: form.quantity,
    code_length: form.code_length,
    status: form.status,
    amount: form.amount,
    expires_at: datetimeLocalToIso(form.expires_at),
    remark: form.remark.trim() || undefined,
  };
}

function typeForm(item: CardCodeType | null): TypeForm {
  return {
    name: item?.name ?? '',
    balance_type: item?.balance_type ?? 'recharge',
    status: item?.status === 'disabled' ? 'disabled' : 'active',
    remark: item?.remark ?? '',
  };
}

function typePayload(form: TypeForm): CardCodeTypeInput {
  return {
    name: form.name.trim(),
    balance_type: form.balance_type,
    status: form.status,
    remark: form.remark.trim() || undefined,
  };
}

function BalanceTypeOptions({ t }: { t: TFunction<'admin'> }) {
  return (
    <>
      <MenuItem value="recharge">{t('wallet.balanceTypeLabels.recharge')}</MenuItem>
      <MenuItem value="gift">{t('wallet.balanceTypeLabels.gift')}</MenuItem>
    </>
  );
}

function balanceTypeSelectProps(t: TFunction<'admin'>) {
  return {
    displayEmpty: true,
    renderValue: (selected: unknown) => balanceTypeLabel(t, String(selected)),
  };
}

function cardCodeAmountLabel(t: TFunction<'admin'>, type: CardCodeType | null) {
  if (type?.balance_type === 'gift') {
    return t('adminCardCodes.fields.giftAmount');
  }
  if (type?.balance_type === 'recharge') {
    return t('adminCardCodes.fields.rechargeAmount');
  }
  return t('adminCardCodes.fields.amount');
}

function balanceTypeLabel(t: TFunction<'admin'>, balanceType: string) {
  if (balanceType === 'gift' || balanceType === 'recharge') {
    return t(`wallet.balanceTypeLabels.${balanceType}`);
  }
  return '';
}

function datetimeLocalToIso(value: string) {
  return value ? new Date(value).toISOString() : undefined;
}

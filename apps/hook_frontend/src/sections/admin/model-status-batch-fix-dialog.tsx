'use client';

import type { TFunction } from 'i18next';
import type { ModelStatusBatchUpdateRequest } from 'src/types/model-status';

import { useState, useEffect } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { toast } from 'src/components/snackbar';

import { MODEL_STATUS_INTERVALS } from '../model-status/model-status-options';

type BatchFixPatch = Omit<ModelStatusBatchUpdateRequest, 'ids'>;

type Props = {
  open: boolean;
  selectedCount: number;
  tokens: { id: string; name: string; token_prefix: string }[];
  submitting: boolean;
  t: TFunction<'admin'>;
  onClose: VoidFunction;
  onSubmit: (patch: BatchFixPatch) => Promise<void>;
};

type FormState = {
  enabled: '' | 'true' | 'false';
  intervalSeconds: string;
  namePrefix: string;
  apiTokenId: string;
};

const emptyForm: FormState = {
  enabled: '',
  intervalSeconds: '',
  namePrefix: '',
  apiTokenId: '',
};

export function ModelStatusBatchFixDialog({ open, selectedCount, tokens, submitting, t, onClose, onSubmit }: Props) {
  const [form, setForm] = useState<FormState>(emptyForm);
  const set = (patch: Partial<FormState>) => setForm((current) => ({ ...current, ...patch }));
  useEffect(() => {
    if (!open) setForm(emptyForm);
  }, [open]);

  return (
    <Dialog fullWidth maxWidth="sm" open={open} onClose={onClose}>
      <Box component="form" onSubmit={(event) => void submit(event, form, t, onSubmit)}>
        <DialogTitle>{t('modelStatusChecks.batchFixTitle')}</DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ pt: 1 }}>
            <Typography variant="body2" color="text.secondary">
              {t('modelStatusChecks.batchFixCount', { count: selectedCount })}
            </Typography>
            <TextField select label={t('modelStatusChecks.enabled')} value={form.enabled} onChange={(event) => set({ enabled: event.target.value as FormState['enabled'] })}>
              <MenuItem value="">{t('modelStatusChecks.noChange')}</MenuItem>
              <MenuItem value="true">{t('modelStatusChecks.enable')}</MenuItem>
              <MenuItem value="false">{t('modelStatusChecks.disable')}</MenuItem>
            </TextField>
            <TextField select label={t('modelStatusChecks.interval')} value={form.intervalSeconds} onChange={(event) => set({ intervalSeconds: event.target.value })}>
              <MenuItem value="">{t('modelStatusChecks.noChange')}</MenuItem>
              {MODEL_STATUS_INTERVALS.map((item) => <MenuItem key={item.value} value={String(item.value)}>{item.label}</MenuItem>)}
            </TextField>
            <TextField
              label={t('modelStatusChecks.namePrefix')}
              value={form.namePrefix}
              placeholder={t('modelStatusChecks.namePrefixPlaceholder')}
              onChange={(event) => set({ namePrefix: event.target.value })}
            />
            <TextField select label={t('modelStatusChecks.apiToken')} value={form.apiTokenId} onChange={(event) => set({ apiTokenId: event.target.value })}>
              <MenuItem value="">{t('modelStatusChecks.noChange')}</MenuItem>
              {tokens.length === 0 ? <MenuItem disabled value="">{t('common.noData')}</MenuItem> : null}
              {tokens.map((token) => <MenuItem key={token.id} value={token.id}>{token.name} · {token.token_prefix}</MenuItem>)}
            </TextField>
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button color="inherit" onClick={onClose}>{t('common.cancel')}</Button>
          <Button type="submit" variant="contained" loading={submitting}>{t('modelStatusChecks.batchFix')}</Button>
        </DialogActions>
      </Box>
    </Dialog>
  );
}

async function submit(
  event: React.FormEvent<HTMLFormElement>,
  form: FormState,
  t: TFunction<'admin'>,
  onSubmit: (patch: BatchFixPatch) => Promise<void>
) {
  event.preventDefault();
  try {
    await onSubmit(batchFixPatch(form, t));
  } catch (error) {
    toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
  }
}

function batchFixPatch(form: FormState, t: TFunction<'admin'>): BatchFixPatch {
  const patch: BatchFixPatch = {};
  if (form.enabled) patch.enabled = form.enabled === 'true';
  if (form.intervalSeconds) patch.interval_seconds = Number(form.intervalSeconds);
  if (form.namePrefix.trim()) patch.name_prefix = form.namePrefix.trim();
  if (form.apiTokenId) patch.api_token_id = form.apiTokenId;
  if (Object.keys(patch).length === 0) throw new Error(t('modelStatusChecks.emptyBatchFix'));
  return patch;
}

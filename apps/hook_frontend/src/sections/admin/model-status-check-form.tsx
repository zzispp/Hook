'use client';

import type { TFunction } from 'i18next';
import type { ModelStatusCheck, ModelStatusCheckCreate, ModelStatusCheckBatchCreate } from 'src/types/model-status';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Switch from '@mui/material/Switch';
import Dialog from '@mui/material/Dialog';
import MenuItem from '@mui/material/MenuItem';
import Checkbox from '@mui/material/Checkbox';
import TextField from '@mui/material/TextField';
import DialogTitle from '@mui/material/DialogTitle';
import ListItemText from '@mui/material/ListItemText';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import FormControlLabel from '@mui/material/FormControlLabel';

import { MODEL_STATUS_INTERVALS, MODEL_STATUS_API_FORMATS } from '../model-status/model-status-options';

export type ModelStatusCheckModelOption = { id: string; name: string; display_name: string };

export type ModelStatusCheckFormState = Omit<ModelStatusCheckCreate, 'global_model_id'> & {
  id?: string;
  global_model_id: string;
  global_model_ids: string[];
  api_formats: string[];
};

export function ModelStatusCheckDialog({
  form,
  t,
  models,
  tokens,
  submitting,
  onChange,
  onClose,
  onSubmit,
}: {
  form: ModelStatusCheckFormState | null;
  t: TFunction<'admin'>;
  models: ModelStatusCheckModelOption[];
  tokens: { id: string; name: string; token_prefix: string }[];
  submitting: boolean;
  onChange: (form: ModelStatusCheckFormState | null) => void;
  onClose: VoidFunction;
  onSubmit: (event: React.FormEvent<HTMLFormElement>) => void;
}) {
  if (!form) return null;
  const set = (patch: Partial<ModelStatusCheckFormState>) => onChange({ ...form, ...patch });
  const editing = Boolean(form.id);
  return (
    <Dialog fullWidth maxWidth="sm" open onClose={onClose}>
      <Box component="form" onSubmit={onSubmit}>
        <DialogTitle>{t(form.id ? 'modelStatusChecks.edit' : 'modelStatusChecks.create')}</DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ pt: 1 }}>
            <TextField required label={t('modelStatusChecks.name')} value={form.name} onChange={(event) => set({ name: event.target.value })} />
            <ModelSelect form={form} models={models} t={t} editing={editing} onChange={set} />
            <ApiFormatSelect form={form} t={t} editing={editing} onChange={set} />
            <TextField required select label={t('modelStatusChecks.apiToken')} value={form.api_token_id} onChange={(event) => set({ api_token_id: event.target.value })}>
              {tokens.length === 0 ? <MenuItem disabled value="">{t('common.noData')}</MenuItem> : null}
              {tokens.map((token) => <MenuItem key={token.id} value={token.id}>{token.name} · {token.token_prefix}</MenuItem>)}
            </TextField>
            <TextField required select label={t('modelStatusChecks.interval')} value={form.interval_seconds} onChange={(event) => set({ interval_seconds: Number(event.target.value) })}>
              {MODEL_STATUS_INTERVALS.map((item) => <MenuItem key={item.value} value={item.value}>{item.label}</MenuItem>)}
            </TextField>
            <FormControlLabel control={<Switch checked={form.enabled ?? true} onChange={(event) => set({ enabled: event.target.checked })} />} label={t('modelStatusChecks.enabled')} />
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button color="inherit" onClick={onClose}>{t('common.cancel')}</Button>
          <Button type="submit" variant="contained" loading={submitting}>{t('common.save')}</Button>
        </DialogActions>
      </Box>
    </Dialog>
  );
}

export function emptyModelStatusCheckForm(): ModelStatusCheckFormState {
  return { name: '', global_model_id: '', global_model_ids: [], api_format: 'openai:chat', api_formats: ['openai:chat'], api_token_id: '', interval_seconds: 300, enabled: true };
}

export function modelStatusCheckFormFromRow(row: ModelStatusCheck): ModelStatusCheckFormState {
  return { id: row.id, name: row.name, global_model_id: row.global_model_id, global_model_ids: [row.global_model_id], api_format: row.api_format, api_formats: [row.api_format], api_token_id: row.api_token_id, interval_seconds: row.interval_seconds, enabled: row.enabled };
}

export function modelStatusCheckPayload(form: ModelStatusCheckFormState): ModelStatusCheckCreate {
  return {
    name: form.name,
    global_model_id: form.global_model_id,
    api_format: form.api_format,
    api_token_id: form.api_token_id,
    interval_seconds: form.interval_seconds,
    enabled: form.enabled,
  };
}

export function modelStatusCheckBatchCreatePayload(form: ModelStatusCheckFormState, t: TFunction<'admin'>): ModelStatusCheckBatchCreate {
  validateCreateSelection(form, t);
  return {
    name_prefix: form.name,
    global_model_ids: form.global_model_ids,
    api_formats: form.api_formats,
    api_token_id: form.api_token_id,
    interval_seconds: form.interval_seconds,
    enabled: form.enabled,
  };
}

function validateCreateSelection(form: ModelStatusCheckFormState, t: TFunction<'admin'>) {
  if (form.global_model_ids.length === 0) {
    throw new Error(t('modelStatusChecks.modelRequired'));
  }
  if (form.api_formats.length === 0) {
    throw new Error(t('modelStatusChecks.apiFormatRequired'));
  }
}

export function intervalLabel(value: number) {
  return MODEL_STATUS_INTERVALS.find((item) => item.value === value)?.label ?? `${value}s`;
}

function ModelSelect({
  form,
  models,
  t,
  editing,
  onChange,
}: {
  form: ModelStatusCheckFormState;
  models: ModelStatusCheckModelOption[];
  t: TFunction<'admin'>;
  editing: boolean;
  onChange: (patch: Partial<ModelStatusCheckFormState>) => void;
}) {
  if (editing) {
    return <SingleModelSelect form={form} models={models} t={t} onChange={onChange} />;
  }
  return <MultiModelSelect form={form} models={models} t={t} onChange={onChange} />;
}

function SingleModelSelect({
  form,
  models,
  t,
  onChange,
}: {
  form: ModelStatusCheckFormState;
  models: ModelStatusCheckModelOption[];
  t: TFunction<'admin'>;
  onChange: (patch: Partial<ModelStatusCheckFormState>) => void;
}) {
  return (
    <TextField required select label={t('modelStatusChecks.model')} value={form.global_model_id} onChange={(event) => onChange({ global_model_id: event.target.value, global_model_ids: [event.target.value] })}>
      {models.length === 0 ? <MenuItem disabled value="">{t('common.noData')}</MenuItem> : null}
      {models.map((model) => <MenuItem key={model.id} value={model.id}>{modelLabel(model)}</MenuItem>)}
    </TextField>
  );
}

function MultiModelSelect({
  form,
  models,
  t,
  onChange,
}: {
  form: ModelStatusCheckFormState;
  models: ModelStatusCheckModelOption[];
  t: TFunction<'admin'>;
  onChange: (patch: Partial<ModelStatusCheckFormState>) => void;
}) {
  return (
    <TextField required select label={t('modelStatusChecks.model')} value={form.global_model_ids} SelectProps={{ multiple: true, renderValue: (selected) => modelSelectionLabel(selected as string[], models) }} onChange={(event) => onChange({ global_model_ids: selectedValues(event.target.value) })}>
      {models.length === 0 ? <MenuItem disabled value="">{t('common.noData')}</MenuItem> : null}
      {models.map((model) => <MenuItem key={model.id} value={model.id}><Checkbox checked={form.global_model_ids.includes(model.id)} /><ListItemText primary={modelLabel(model)} secondary={model.name} /></MenuItem>)}
    </TextField>
  );
}

function ApiFormatSelect({
  form,
  t,
  editing,
  onChange,
}: {
  form: ModelStatusCheckFormState;
  t: TFunction<'admin'>;
  editing: boolean;
  onChange: (patch: Partial<ModelStatusCheckFormState>) => void;
}) {
  if (editing) {
    return <SingleApiFormatSelect form={form} t={t} onChange={onChange} />;
  }
  return <MultiApiFormatSelect form={form} t={t} onChange={onChange} />;
}

function SingleApiFormatSelect({
  form,
  t,
  onChange,
}: {
  form: ModelStatusCheckFormState;
  t: TFunction<'admin'>;
  onChange: (patch: Partial<ModelStatusCheckFormState>) => void;
}) {
  return (
    <TextField required select label={t('modelStatusChecks.apiFormat')} value={form.api_format} onChange={(event) => onChange({ api_format: event.target.value, api_formats: [event.target.value] })}>
      {MODEL_STATUS_API_FORMATS.map((format) => <MenuItem key={format} value={format}>{format}</MenuItem>)}
    </TextField>
  );
}

function MultiApiFormatSelect({
  form,
  t,
  onChange,
}: {
  form: ModelStatusCheckFormState;
  t: TFunction<'admin'>;
  onChange: (patch: Partial<ModelStatusCheckFormState>) => void;
}) {
  return (
    <TextField required select label={t('modelStatusChecks.apiFormat')} value={form.api_formats} SelectProps={{ multiple: true, renderValue: (selected) => (selected as string[]).join(', ') }} onChange={(event) => onChange({ api_formats: selectedValues(event.target.value) })}>
      {MODEL_STATUS_API_FORMATS.map((format) => <MenuItem key={format} value={format}><Checkbox checked={form.api_formats.includes(format)} /><ListItemText primary={format} /></MenuItem>)}
    </TextField>
  );
}

function selectedValues(value: string | string[]) {
  return Array.isArray(value) ? value : value.split(',').filter(Boolean);
}

function modelSelectionLabel(ids: string[], models: ModelStatusCheckModelOption[]) {
  return ids.map((id) => modelLabel(models.find((model) => model.id === id) ?? { id, name: id, display_name: '' })).join(', ');
}

function modelLabel(model: { name: string; display_name: string }) {
  return model.display_name || model.name;
}

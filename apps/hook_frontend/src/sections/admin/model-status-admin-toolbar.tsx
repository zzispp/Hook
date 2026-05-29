'use client';

import type { TFunction } from 'i18next';
import type { ModelStatusValue } from 'src/types/model-status';
import type { useModelStatusAdminState } from './model-status-admin-state';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import InputAdornment from '@mui/material/InputAdornment';

import { Iconify } from 'src/components/iconify';

import { MODEL_STATUS_API_FORMATS } from '../model-status/model-status-options';

const ALL_FILTER_VALUE = '';

type State = ReturnType<typeof useModelStatusAdminState>;

export function ChecksToolbar({ state, t }: { state: State; t: TFunction<'admin'> }) {
  const selectedCount = state.checkTable.selected.length;
  return (
    <Box sx={toolbarSx}>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <SearchField value={state.checkFilters.search ?? ''} placeholder={t('modelStatus.searchPlaceholder')} onChange={state.changeCheckSearch} />
        <ApiFormatField value={state.checkFilters.api_format ?? ''} t={t} onChange={state.changeCheckApiFormat} />
        <Button
          variant="outlined"
          disabled={selectedCount === 0}
          startIcon={<Iconify icon="solar:settings-bold" />}
          sx={actionButtonSx}
          onClick={() => state.setBatchFixOpen(true)}
        >
          {t('modelStatusChecks.batchFix')}
        </Button>
        <Button
          color="error"
          variant="outlined"
          disabled={selectedCount === 0}
          startIcon={<Iconify icon="solar:trash-bin-trash-bold" />}
          sx={actionButtonSx}
          onClick={() => state.setDeletingIds(state.checkTable.selected)}
        >
          {t('modelStatusChecks.deleteSelected', { count: selectedCount })}
        </Button>
      </Stack>
    </Box>
  );
}

export function RunsToolbar({ state, t }: { state: State; t: TFunction<'admin'> }) {
  return (
    <Box sx={toolbarSx}>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <SearchField value={state.runFilters.search ?? ''} placeholder={t('modelStatusChecks.runs.searchPlaceholder')} onChange={state.changeRunSearch} />
        <ApiFormatField value={state.runFilters.api_format ?? ''} t={t} onChange={state.changeRunApiFormat} />
        <StatusField value={state.runFilters.status ?? ''} t={t} onChange={state.changeRunStatus} />
      </Stack>
    </Box>
  );
}

function SearchField({ value, placeholder, onChange }: { value: string; placeholder: string; onChange: (value: string) => void }) {
  return (
    <TextField
      fullWidth
      size="small"
      value={value}
      placeholder={placeholder}
      sx={searchFieldSx}
      onChange={(event) => onChange(event.target.value)}
      slotProps={{ input: { startAdornment: <SearchIcon /> } }}
    />
  );
}

function ApiFormatField({ value, t, onChange }: { value: string; t: TFunction<'admin'>; onChange: (value: string) => void }) {
  return (
    <TextField select size="small" label={t('modelStatus.apiFormat')} value={value} sx={filterFieldSx} onChange={(event) => onChange(event.target.value)}>
      <MenuItem value={ALL_FILTER_VALUE}>{t('modelStatus.allApiFormats')}</MenuItem>
      {MODEL_STATUS_API_FORMATS.map((format) => <MenuItem key={format} value={format}>{format}</MenuItem>)}
    </TextField>
  );
}

function StatusField({ value, t, onChange }: { value: string; t: TFunction<'admin'>; onChange: (value: ModelStatusValue | '') => void }) {
  return (
    <TextField select size="small" label={t('common.status')} value={value} sx={filterFieldSx} onChange={(event) => onChange(event.target.value as ModelStatusValue | '')}>
      <MenuItem value={ALL_FILTER_VALUE}>{t('filters.allStatuses')}</MenuItem>
      {(['operational', 'degraded', 'failed', 'error'] as ModelStatusValue[]).map((status) => (
        <MenuItem key={status} value={status}>{t(`modelStatus.statusLabel.${status}`)}</MenuItem>
      ))}
    </TextField>
  );
}

function SearchIcon() {
  return (
    <InputAdornment position="start">
      <Iconify icon="eva:search-fill" sx={{ color: 'text.disabled' }} />
    </InputAdornment>
  );
}

const toolbarSx = { p: 2.5 };
const searchFieldSx = { flex: 1, minWidth: { md: 280 } };
const filterFieldSx = { minWidth: 180 };
const actionButtonSx = { flexShrink: 0, whiteSpace: 'nowrap' };

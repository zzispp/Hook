import type { TFunction } from 'i18next';
import type { ModelStatusListFilters } from 'src/types/model-status';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import InputAdornment from '@mui/material/InputAdornment';

import { Iconify } from 'src/components/iconify';

import { MODEL_STATUS_API_FORMATS } from './model-status-options';

const ALL_API_FORMATS = '';

type Props = {
  filters: ModelStatusListFilters;
  t: TFunction<'admin'>;
  onChange: (filters: ModelStatusListFilters) => void;
};

export function ModelStatusToolbar({ filters, t, onChange }: Props) {
  const patch = (patchFilters: Partial<ModelStatusListFilters>) =>
    onChange({ ...filters, ...patchFilters });

  return (
    <Box sx={{ p: 2.5 }}>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <SearchField
          value={filters.search ?? ''}
          t={t}
          onChange={(search) => patch({ search })}
        />
        <ApiFormatFilter
          value={filters.api_format ?? ALL_API_FORMATS}
          t={t}
          onChange={(api_format) => patch({ api_format })}
        />
      </Stack>
    </Box>
  );
}

function SearchField({
  value,
  t,
  onChange,
}: {
  value: string;
  t: TFunction<'admin'>;
  onChange: (value: string) => void;
}) {
  return (
    <TextField
      fullWidth
      size="small"
      value={value}
      placeholder={t('modelStatus.searchPlaceholder')}
      onChange={(event) => onChange(event.target.value)}
      slotProps={{ input: { startAdornment: <SearchIcon /> } }}
    />
  );
}

function ApiFormatFilter({
  value,
  t,
  onChange,
}: {
  value: string;
  t: TFunction<'admin'>;
  onChange: (value: string) => void;
}) {
  return (
    <TextField
      select
      size="small"
      label={t('modelStatus.apiFormat')}
      value={value}
      sx={{ minWidth: 180 }}
      onChange={(event) => onChange(event.target.value)}
    >
      <MenuItem value={ALL_API_FORMATS}>{t('modelStatus.allApiFormats')}</MenuItem>
      {MODEL_STATUS_API_FORMATS.map((format) => (
        <MenuItem key={format} value={format}>
          {format}
        </MenuItem>
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

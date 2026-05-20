'use client';

import type { GlobalModelResponse } from 'src/types/model';
import type { RequestRecordFilters } from 'src/actions/request-records';

import { useMemo, useCallback } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Switch from '@mui/material/Switch';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import Autocomplete from '@mui/material/Autocomplete';
import InputAdornment from '@mui/material/InputAdornment';
import FormControlLabel from '@mui/material/FormControlLabel';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

import { formatApiFormat, API_FORMAT_OPTIONS } from 'src/sections/admin/provider-management-utils';

const ALL_FILTER_VALUE = 'all';

export type UsageRecordFilterState = {
  search: string;
  modelId: string;
  apiFormat: string;
  type: string;
};

type FilterOption = {
  value: string;
  label: string;
};

export const DEFAULT_USAGE_RECORD_FILTERS: UsageRecordFilterState = {
  search: '',
  modelId: ALL_FILTER_VALUE,
  apiFormat: ALL_FILTER_VALUE,
  type: ALL_FILTER_VALUE,
};

export function UsageRecordsToolbar({
  filters,
  models,
  autoRefresh,
  onChange,
  onAutoRefreshChange,
}: {
  filters: UsageRecordFilterState;
  models: UsageRecordModelOption[];
  autoRefresh: boolean;
  onChange: (filters: UsageRecordFilterState) => void;
  onAutoRefreshChange: (value: boolean) => void;
}) {
  const options = useUsageRecordFilterOptions(models);
  const updateFilters = useUsageRecordFilterUpdater(filters, onChange);

  return (
    <Box sx={{ p: 2.5 }}>
      <Box sx={toolbarGridSx}>
        <SearchField value={filters.search} onChange={(search) => updateFilters({ search })} />
        <SearchSelect
          value={filters.modelId}
          label={options.labels.model}
          options={options.models}
          onChange={(modelId) => updateFilters({ modelId })}
        />
        <SelectFilter
          value={filters.apiFormat}
          label={options.labels.apiFormat}
          options={options.apiFormats}
          onChange={(apiFormat) => updateFilters({ apiFormat })}
        />
        <SelectFilter
          value={filters.type}
          label={options.labels.type}
          options={options.types}
          onChange={(type) => updateFilters({ type })}
        />
        <AutoRefreshSwitch value={autoRefresh} onChange={onAutoRefreshChange} />
      </Box>
    </Box>
  );
}

export function toUsageRecordQueryFilters(
  filters: UsageRecordFilterState
): Omit<RequestRecordFilters, 'provider_id'> {
  return {
    search: normalizedSearch(filters.search),
    model_id: optionalFilter(filters.modelId),
    api_format: optionalFilter(filters.apiFormat),
    type: optionalFilter(filters.type),
  };
}

function SearchField({ value, onChange }: { value: string; onChange: (value: string) => void }) {
  const { t } = useTranslate('admin');

  return (
    <TextField
      size="small"
      value={value}
      placeholder={t('filters.searchUsageRecords')}
      onChange={(event) => onChange(event.target.value)}
      slotProps={{
        input: {
          startAdornment: (
            <InputAdornment position="start">
              <Iconify icon="eva:search-fill" sx={{ color: 'text.disabled' }} />
            </InputAdornment>
          ),
        },
      }}
    />
  );
}

function SearchSelect({
  value,
  label,
  options,
  onChange,
}: {
  value: string;
  label: string;
  options: FilterOption[];
  onChange: (value: string) => void;
}) {
  const selected = options.find((option) => option.value === value) ?? options[0] ?? null;

  return (
    <Autocomplete
      size="small"
      options={options}
      value={selected}
      getOptionLabel={(option) => option.label}
      isOptionEqualToValue={(option, current) => option.value === current.value}
      onChange={(_event, option) => onChange(option?.value ?? ALL_FILTER_VALUE)}
      renderInput={(params) => <TextField {...params} label={label} />}
    />
  );
}

function SelectFilter({
  value,
  label,
  options,
  onChange,
}: {
  value: string;
  label: string;
  options: FilterOption[];
  onChange: (value: string) => void;
}) {
  return (
    <TextField
      select
      size="small"
      label={label}
      value={value}
      onChange={(event) => onChange(event.target.value)}
    >
      {options.map((option) => (
        <MenuItem key={option.value || 'all'} value={option.value}>
          {option.label}
        </MenuItem>
      ))}
    </TextField>
  );
}

function AutoRefreshSwitch({
  value,
  onChange,
}: {
  value: boolean;
  onChange: (value: boolean) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction="row" alignItems="center">
      <FormControlLabel
        control={<Switch checked={value} onChange={(event) => onChange(event.target.checked)} />}
        label={<Typography variant="body2">{t('requestRecords.autoRefresh')}</Typography>}
        sx={{ whiteSpace: 'nowrap' }}
      />
    </Stack>
  );
}

function useUsageRecordFilterOptions(models: UsageRecordModelOption[]) {
  const { t } = useTranslate('admin');

  return useMemo(
    () => ({
      labels: {
        model: t('requestRecords.model'),
        apiFormat: t('requestRecords.apiFormat'),
        type: t('requestRecords.type'),
      },
      models: [
        { value: ALL_FILTER_VALUE, label: t('requestRecords.allModels') },
        ...models,
      ],
      apiFormats: [
        { value: ALL_FILTER_VALUE, label: t('requestRecords.allFormats') },
        ...API_FORMAT_OPTIONS.map((value) => ({ value, label: formatApiFormat(value) })),
      ],
      types: [
        { value: ALL_FILTER_VALUE, label: t('requestRecords.allTypes') },
        { value: 'stream', label: t('requestRecords.stream') },
        { value: 'non_stream', label: t('requestRecords.nonStream') },
      ],
    }),
    [models, t]
  );
}

function useUsageRecordFilterUpdater(
  filters: UsageRecordFilterState,
  onChange: (filters: UsageRecordFilterState) => void
) {
  return useCallback(
    (patch: Partial<UsageRecordFilterState>) => {
      onChange({ ...filters, ...patch });
    },
    [filters, onChange]
  );
}

function normalizedSearch(search: string) {
  const trimmed = search.trim();
  return trimmed || undefined;
}

function optionalFilter(value: string) {
  const normalized = value.trim();
  if (!normalized || normalized === ALL_FILTER_VALUE) return undefined;
  return normalized;
}

const toolbarGridSx = {
  gap: 1.5,
  display: 'grid',
  alignItems: 'center',
  gridTemplateColumns: {
    xs: '1fr',
    md: 'minmax(220px, 1.4fr) repeat(2, minmax(180px, 1fr))',
    xl: 'minmax(240px, 1.4fr) repeat(3, minmax(150px, 1fr)) auto',
  },
};

export type UsageRecordModelOption = {
  value: string;
  label: string;
};

export function catalogUsageRecordOptions(models: GlobalModelResponse[]) {
  return models.map((model) => ({
    value: model.id,
    label: model.display_name || model.name,
  }));
}

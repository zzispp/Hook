'use client';

import type { Theme } from '@mui/material/styles';
import type { GlobalModelResponse } from 'src/types/model';
import type { ProviderFilters } from 'src/actions/providers';

import { useMemo, useCallback } from 'react';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import InputAdornment from '@mui/material/InputAdornment';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

import { formatApiFormat, API_FORMAT_OPTIONS } from './provider-management-utils';

type ProviderStatusFilter = 'all' | 'enabled' | 'disabled';

export type ProviderFilterState = {
  search: string;
  status: ProviderStatusFilter;
  apiFormat: string;
  modelId: string;
};

type Props = {
  filters: ProviderFilterState;
  models: GlobalModelResponse[];
  schedulingLabel: string;
  onChange: (filters: ProviderFilterState) => void;
  onOpenPriority: () => void;
};

type FilterOption = {
  label: string;
  value: string;
};

export const DEFAULT_PROVIDER_FILTERS: ProviderFilterState = {
  search: '',
  status: 'all',
  apiFormat: 'all',
  modelId: 'all',
};

export function ProviderFiltersToolbar(props: Props) {
  const options = useProviderFilterOptions(props.models);
  const hasActiveFilters = providerFiltersAreActive(props.filters);

  return (
    <Box sx={{ px: 2.5, py: 2, borderBottom: (theme: Theme) => `1px solid ${theme.palette.divider}` }}>
      <FilterControls {...props} {...options} hasActiveFilters={hasActiveFilters} />
    </Box>
  );
}

function FilterControls({
  filters,
  onChange,
  apiFormats,
  statusOptions,
  modelOptions,
  schedulingLabel,
  onOpenPriority,
  hasActiveFilters,
}: Props & ReturnType<typeof useProviderFilterOptions> & { hasActiveFilters: boolean }) {
  const { t } = useTranslate('admin');
  const updateFilters = useProviderFilterUpdater(filters, onChange);

  return (
    <Box
      sx={{
        gap: 1.5,
        display: 'grid',
        alignItems: 'center',
        gridTemplateColumns: {
          xs: '1fr',
          lg: 'minmax(220px, 1fr) 132px 148px 180px auto auto',
        },
      }}
    >
      <SearchField value={filters.search} onChange={(search) => updateFilters({ search })} />
      <SelectFilter
        value={filters.status}
        label={t('common.status')}
        onChange={(status) => updateFilters({ status: status as ProviderStatusFilter })}
      >
        {statusOptions.map((option: FilterOption) => (
          <MenuItem key={option.value} value={option.value}>{option.label}</MenuItem>
        ))}
      </SelectFilter>
      <SelectFilter
        value={filters.apiFormat}
        label={t('providers.apiFormat')}
        onChange={(apiFormat) => updateFilters({ apiFormat })}
      >
        {apiFormats.map((option: FilterOption) => (
          <MenuItem key={option.value} value={option.value}>{option.label}</MenuItem>
        ))}
      </SelectFilter>
      <SelectFilter value={filters.modelId} label={t('providers.model')} onChange={(modelId) => updateFilters({ modelId })}>
        {modelOptions.map((option: FilterOption) => (
          <MenuItem key={option.value} value={option.value}>{option.label}</MenuItem>
        ))}
      </SelectFilter>
      <SchedulingButton label={schedulingLabel} onClick={onOpenPriority} />
      {hasActiveFilters && <ResetFiltersButton onClick={() => onChange(DEFAULT_PROVIDER_FILTERS)} />}
    </Box>
  );
}

function SearchField({ value, onChange }: { value: string; onChange: (value: string) => void }) {
  const { t } = useTranslate('admin');

  return (
    <TextField
      size="small"
      value={value}
      placeholder={t('filters.searchProviders')}
      onChange={(event) => onChange(event.target.value)}
      sx={{ width: 1 }}
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

function SelectFilter({
  value,
  label,
  children,
  onChange,
}: {
  value: string;
  label: string;
  children: React.ReactNode;
  onChange: (value: string) => void;
}) {
  return (
    <TextField
      select
      size="small"
      label={label}
      value={value}
      onChange={(event) => onChange(event.target.value)}
      sx={{ width: 1 }}
    >
      {children}
    </TextField>
  );
}

function SchedulingButton({ label, onClick }: { label: string; onClick: () => void }) {
  const { t } = useTranslate('admin');

  return (
    <Button color="inherit" variant="outlined" endIcon={<Iconify icon="eva:chevron-down-fill" />} onClick={onClick}>
      <Box component="span" sx={{ color: 'text.secondary', mr: 0.75 }}>{t('providers.scheduling')}:</Box>
      {label}
    </Button>
  );
}

function ResetFiltersButton({ onClick }: { onClick: () => void }) {
  const { t } = useTranslate('admin');

  return (
      <Tooltip title={t('wallet.actions.reset')}>
      <IconButton onClick={onClick}>
        <Iconify icon="solar:close-circle-bold" />
      </IconButton>
    </Tooltip>
  );
}

export function toProviderFilters(filters: ProviderFilterState): ProviderFilters {
  return {
    search: normalizedSearch(filters.search),
    is_active: statusValue(filters.status),
    api_format: allValue(filters.apiFormat),
    model_id: allValue(filters.modelId),
  };
}

function useProviderFilterOptions(models: GlobalModelResponse[]) {
  const { t } = useTranslate('admin');

  return useMemo(() => {
    const modelOptions = [...models]
      .sort((left, right) => left.display_name.localeCompare(right.display_name))
      .map((model) => ({ value: model.id, label: model.display_name || model.name }));
    return {
      modelOptions: [{ value: 'all', label: t('providers.allModels') }, ...modelOptions],
      apiFormats: [{ value: 'all', label: t('providers.allFormats') }, ...apiFormatOptions()],
      statusOptions: [
        { value: 'all', label: t('filters.allStatuses') },
        { value: 'enabled', label: t('common.enabled') },
        { value: 'disabled', label: t('common.disabled') },
      ],
    };
  }, [models, t]);
}

function useProviderFilterUpdater(
  filters: ProviderFilterState,
  onChange: (filters: ProviderFilterState) => void
) {
  return useCallback((patch: Partial<ProviderFilterState>) => {
    onChange({ ...filters, ...patch });
  }, [filters, onChange]);
}

function apiFormatOptions() {
  return API_FORMAT_OPTIONS.map((value) => ({ value, label: formatApiFormat(value) }));
}

function providerFiltersAreActive(filters: ProviderFilterState) {
  return filters.search.trim() !== '' || filters.status !== 'all' || filters.apiFormat !== 'all' || filters.modelId !== 'all';
}

function normalizedSearch(search: string) {
  const trimmed = search.trim();
  return trimmed || undefined;
}

function statusValue(status: ProviderStatusFilter) {
  if (status === 'all') return undefined;
  return status === 'enabled';
}

function allValue(value: string) {
  return value === 'all' ? undefined : value;
}

'use client';

import type { RbacListFilters } from 'src/actions/rbac';
import type { GlobalModelFilters } from 'src/actions/models';
import type { ProviderFilters } from 'src/actions/providers';

import { useMemo, useCallback } from 'react';

import Box from '@mui/material/Box';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import InputAdornment from '@mui/material/InputAdornment';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

type StatusFilter = 'all' | 'enabled' | 'disabled';

export type AdminFilterState = {
  role?: string;
  search: string;
  status: StatusFilter;
};

type FilterRoleOption = {
  label: string;
  value: string;
};

type Props = {
  filters: AdminFilterState;
  roleOptions?: FilterRoleOption[];
  searchPlaceholder: string;
  onChange: (filters: AdminFilterState) => void;
};

export const DEFAULT_ADMIN_FILTERS: AdminFilterState = {
  role: '',
  search: '',
  status: 'all',
};

export function AdminFiltersToolbar({ filters, roleOptions = [], searchPlaceholder, onChange }: Props) {
  const { t } = useTranslate('admin');
  const statusOptions = useStatusOptions();

  const updateFilters = useCallback(
    (patch: Partial<AdminFilterState>) => {
      onChange({ ...filters, ...patch });
    },
    [filters, onChange]
  );

  return (
    <Box
      sx={{
        p: 2.5,
        gap: 2,
        display: 'grid',
        alignItems: 'center',
        gridTemplateColumns: {
          xs: '1fr',
          md: roleOptions.length ? 'minmax(260px, 1fr) 180px 180px' : 'minmax(260px, 1fr) 180px',
        },
      }}
    >
      <TextField
        fullWidth
        value={filters.search}
        placeholder={searchPlaceholder}
        onChange={(event) => updateFilters({ search: event.target.value })}
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
      <TextField
        select
        label={t('common.status')}
        value={filters.status}
        onChange={(event) => updateFilters({ status: event.target.value as StatusFilter })}
      >
        {statusOptions.map((option) => (
          <MenuItem key={option.value} value={option.value}>
            {option.label}
          </MenuItem>
        ))}
      </TextField>
      {!!roleOptions.length && (
        <TextField
          select
          label={t('common.role')}
          value={filters.role ?? ''}
          onChange={(event) => updateFilters({ role: event.target.value })}
        >
          <MenuItem value="">{t('filters.allRoles')}</MenuItem>
          {roleOptions.map((option) => (
            <MenuItem key={option.value} value={option.value}>
              {option.label}
            </MenuItem>
          ))}
        </TextField>
      )}
    </Box>
  );
}

export function toEnabledFilters(filters: AdminFilterState): Pick<RbacListFilters, 'enabled' | 'search'> {
  return {
    search: normalizedSearch(filters.search),
    enabled: statusValue(filters.status),
  };
}

export function toUserFilters(filters: AdminFilterState): RbacListFilters {
  return {
    search: normalizedSearch(filters.search),
    role: filters.role || undefined,
    is_active: statusValue(filters.status),
  };
}

export function toModelFilters(filters: AdminFilterState): GlobalModelFilters {
  return {
    search: normalizedSearch(filters.search),
    is_active: statusValue(filters.status),
  };
}

export function toProviderFilters(filters: AdminFilterState): ProviderFilters {
  return {
    search: normalizedSearch(filters.search),
    is_active: statusValue(filters.status),
  };
}

function useStatusOptions() {
  const { t } = useTranslate('admin');

  return useMemo(
    () => [
      { value: 'all', label: t('filters.allStatuses') },
      { value: 'enabled', label: t('common.enabled') },
      { value: 'disabled', label: t('common.disabled') },
    ],
    [t]
  );
}

function normalizedSearch(search: string) {
  const trimmed = search.trim();
  return trimmed || undefined;
}

function statusValue(status: StatusFilter) {
  if (status === 'all') return undefined;
  return status === 'enabled';
}

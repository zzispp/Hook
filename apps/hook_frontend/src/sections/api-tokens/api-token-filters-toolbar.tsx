'use client';

import type { ApiTokenType } from 'src/types/api-token';
import type { ApiTokenFilters } from 'src/actions/api-tokens';
import type { AdminFilterState } from '../admin/admin-filters-toolbar';

import { useMemo, useCallback } from 'react';

import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/locales/use-locales';

import { AdminFiltersToolbar } from '../admin/admin-filters-toolbar';

type TokenTypeFilter = 'all' | ApiTokenType;

export type TokenFilterState = {
  search: string;
  status: AdminFilterState['status'];
  tokenType: TokenTypeFilter;
};

type Props = {
  filters: TokenFilterState;
  showTokenType?: boolean;
  onChange: (filters: TokenFilterState) => void;
};

export const DEFAULT_TOKEN_FILTERS: TokenFilterState = {
  search: '',
  status: 'all',
  tokenType: 'all',
};

export function ApiTokenFiltersToolbar({ filters, showTokenType = false, onChange }: Props) {
  const { t } = useTranslate('admin');
  const tokenTypeOptions = useTokenTypeOptions();
  const baseFilters = useMemo(
    () => ({ search: filters.search, status: filters.status }),
    [filters.search, filters.status]
  );
  const handleBaseFiltersChange = useCallback(
    (base: AdminFilterState) => {
      onChange({ ...filters, search: base.search, status: base.status });
    },
    [filters, onChange]
  );

  return (
    <AdminFiltersToolbar
      filters={baseFilters}
      searchPlaceholder={t('filters.searchTokens')}
      onChange={handleBaseFiltersChange}
    >
      {showTokenType ? (
        <TextField
          select
          label={t('fields.tokenType')}
          value={filters.tokenType}
          onChange={(event) =>
            onChange({ ...filters, tokenType: event.target.value as TokenTypeFilter })
          }
        >
          {tokenTypeOptions.map((option) => (
            <MenuItem key={option.value} value={option.value}>
              {option.label}
            </MenuItem>
          ))}
        </TextField>
      ) : null}
    </AdminFiltersToolbar>
  );
}

export function toApiTokenFilters(
  filters: TokenFilterState,
  fixedUserId?: string
): ApiTokenFilters {
  return {
    user_id: fixedUserId,
    search: normalizedSearch(filters.search),
    is_active: statusValue(filters.status),
    token_type: filters.tokenType === 'all' ? undefined : filters.tokenType,
  };
}

function useTokenTypeOptions() {
  const { t } = useTranslate('admin');

  return useMemo(
    () => [
      { value: 'all', label: t('filters.allTokenTypes') },
      { value: 'independent', label: t('tokens.independentToken') },
      { value: 'user', label: t('tokens.userToken') },
    ],
    [t]
  );
}

function normalizedSearch(search: string) {
  const trimmed = search.trim();
  return trimmed || undefined;
}

function statusValue(status: AdminFilterState['status']) {
  if (status === 'all') return undefined;
  return status === 'enabled';
}

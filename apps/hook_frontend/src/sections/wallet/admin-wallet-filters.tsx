'use client';

import type { TFunction } from 'i18next';
import type { AdminWalletFilters } from 'src/actions/wallet';

import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import InputAdornment from '@mui/material/InputAdornment';

import { Iconify } from 'src/components/iconify';

import { ALL_FILTER_VALUE } from './wallet-constants';

export type AdminWalletFilterState = {
  search: string;
  status: string;
};

export const DEFAULT_ADMIN_WALLET_FILTERS: AdminWalletFilterState = {
  search: '',
  status: ALL_FILTER_VALUE,
};

type Props = {
  t: TFunction<'admin'>;
  filters: AdminWalletFilterState;
  onChange: (filters: AdminWalletFilterState) => void;
};

export function AdminWalletFiltersToolbar({ t, filters, onChange }: Props) {
  const patchFilters = (patch: Partial<AdminWalletFilterState>) => {
    onChange({ ...filters, ...patch });
  };

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2.5 }}>
      <TextField
        fullWidth
        value={filters.search}
        placeholder={t('adminWallets.filters.searchPlaceholder')}
        onChange={(event) => patchFilters({ search: event.target.value })}
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
        sx={{ minWidth: 180 }}
        onChange={(event) => patchFilters({ status: event.target.value })}
      >
        <MenuItem value={ALL_FILTER_VALUE}>{t('filters.allStatuses')}</MenuItem>
        <MenuItem value="active">{t('wallet.statusLabels.active')}</MenuItem>
        <MenuItem value="disabled">{t('wallet.statusLabels.disabled')}</MenuItem>
      </TextField>
    </Stack>
  );
}

export function toAdminWalletFilters(filters: AdminWalletFilterState): AdminWalletFilters {
  return {
    search: filters.search.trim() || undefined,
    status: filters.status === ALL_FILTER_VALUE ? undefined : filters.status,
  };
}

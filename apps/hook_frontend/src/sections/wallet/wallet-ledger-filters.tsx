'use client';

import type { TFunction } from 'i18next';
import type { WalletFilterOption, WalletLedgerFilterState } from './wallet-filters';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { Iconify } from 'src/components/iconify';

import { FILTER_CONTROL_WIDTH, SEARCH_CONTROL_WIDTH } from './wallet-constants';
import {
  DEFAULT_WALLET_FILTERS,
  walletStaticFilterOptions,
} from './wallet-filters';

type WalletLedgerFiltersProps = {
  filters: WalletLedgerFilterState;
  t: TFunction<'admin'>;
  reasonOptions: WalletFilterOption[];
  linkTypeOptions: WalletFilterOption[];
  hasFilters: boolean;
  onChange: (filters: WalletLedgerFilterState) => void;
};

export function WalletLedgerFilters({
  filters,
  t,
  onChange,
  hasFilters,
  reasonOptions,
  linkTypeOptions,
}: WalletLedgerFiltersProps) {
  const setFilter = (name: keyof WalletLedgerFilterState, value: string) => {
    onChange({ ...filters, [name]: value });
  };

  return (
    <Stack spacing={2} sx={{ p: 2.5, pt: 0 }}>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={1.5} useFlexGap flexWrap="wrap">
        <SearchFilter t={t} value={filters.search} onChange={(value) => setFilter('search', value)} />
        {filterSelectConfigs(t, filters, reasonOptions, linkTypeOptions).map((config) => (
          <FilterSelect key={config.name} {...config} onChange={(value) => setFilter(config.name, value)} />
        ))}
        <ResetFiltersButton t={t} disabled={!hasFilters} onReset={() => onChange(DEFAULT_WALLET_FILTERS)} />
      </Stack>
    </Stack>
  );
}

function SearchFilter({
  t,
  value,
  onChange,
}: {
  t: TFunction<'admin'>;
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <TextField
      value={value}
      label={t('wallet.filters.searchLabel')}
      placeholder={t('wallet.filters.searchPlaceholder')}
      sx={{ width: { xs: 1, md: SEARCH_CONTROL_WIDTH } }}
      onChange={(event) => onChange(event.target.value)}
    />
  );
}

function filterSelectConfigs(
  t: TFunction<'admin'>,
  filters: WalletLedgerFilterState,
  reasonOptions: WalletFilterOption[],
  linkTypeOptions: WalletFilterOption[]
) {
  const staticOptions = walletStaticFilterOptions(t);

  return [
    { name: 'category' as const, label: t('wallet.filters.category'), value: filters.category, options: staticOptions.categories },
    { name: 'reason' as const, label: t('wallet.filters.reason'), value: filters.reason, options: [allReasonOption(t), ...reasonOptions] },
    { name: 'direction' as const, label: t('wallet.filters.direction'), value: filters.direction, options: staticOptions.directions },
    { name: 'balanceType' as const, label: t('wallet.filters.balanceType'), value: filters.balanceType, options: staticOptions.balanceTypes },
    { name: 'linkType' as const, label: t('wallet.filters.linkType'), value: filters.linkType, options: [allLinkTypeOption(t), ...linkTypeOptions] },
  ];
}

function allReasonOption(t: TFunction<'admin'>): WalletFilterOption {
  return { value: 'all', label: t('wallet.filters.allReasons') };
}

function allLinkTypeOption(t: TFunction<'admin'>): WalletFilterOption {
  return { value: 'all', label: t('wallet.filters.allLinkTypes') };
}

function FilterSelect({
  label,
  value,
  options,
  onChange,
}: {
  name: keyof WalletLedgerFilterState;
  label: string;
  value: string;
  options: WalletFilterOption[];
  onChange: (value: string) => void;
}) {
  return (
    <TextField
      select
      label={label}
      value={value}
      sx={{ minWidth: FILTER_CONTROL_WIDTH }}
      onChange={(event) => onChange(event.target.value)}
    >
      {options.map((option) => (
        <MenuItem key={option.value} value={option.value}>
          {option.label}
        </MenuItem>
      ))}
    </TextField>
  );
}

function ResetFiltersButton({
  t,
  disabled,
  onReset,
}: {
  t: TFunction<'admin'>;
  disabled: boolean;
  onReset: VoidFunction;
}) {
  return (
    <Button
      color="inherit"
      variant="outlined"
      disabled={disabled}
      startIcon={<Iconify icon="solar:restart-bold" />}
      onClick={onReset}
    >
      {t('wallet.actions.reset')}
    </Button>
  );
}

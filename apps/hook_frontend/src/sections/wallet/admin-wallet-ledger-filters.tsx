'use client';

import type { TFunction } from 'i18next';
import type { WalletLedgerEntryFilters } from 'src/actions/wallet';

import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { RefreshButton } from '../admin/shared';
import { walletStaticFilterOptions } from './wallet-filters';
import { walletTransactionReasonLabel } from './wallet-display';
import { ALL_FILTER_VALUE, FILTER_CONTROL_WIDTH } from './wallet-constants';

type AdminWalletLedgerFilterState = {
  category: string;
  reason: string;
  ownerType: string;
};

export const DEFAULT_ADMIN_LEDGER_FILTERS: AdminWalletLedgerFilterState = {
  category: ALL_FILTER_VALUE,
  reason: ALL_FILTER_VALUE,
  ownerType: ALL_FILTER_VALUE,
};

type Props = {
  t: TFunction<'admin'>;
  loading: boolean;
  filters: AdminWalletLedgerFilterState;
  onChange: (filters: AdminWalletLedgerFilterState) => void;
  onRefresh: VoidFunction;
};

export function AdminWalletLedgerFiltersToolbar({ t, loading, filters, onChange, onRefresh }: Props) {
  const patchFilters = (patch: Partial<AdminWalletLedgerFilterState>) => {
    onChange({ ...filters, ...patch });
  };

  return (
    <Stack
      direction={{ xs: 'column', md: 'row' }}
      justifyContent="space-between"
      spacing={1.5}
      sx={{ p: 2.5 }}
    >
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={1.5}>
        <FilterSelect
          label={t('wallet.filters.category')}
          value={filters.category}
          options={walletStaticFilterOptions(t).categories}
          onChange={(category) => patchFilters({ category, reason: ALL_FILTER_VALUE })}
        />
        <FilterSelect
          label={t('wallet.filters.reason')}
          value={filters.reason}
          options={reasonOptions(t, filters.category)}
          onChange={(reason) => patchFilters({ reason })}
        />
        <FilterSelect
          label={t('wallet.filters.ownerType')}
          value={filters.ownerType}
          options={ownerTypeOptions(t)}
          onChange={(ownerType) => patchFilters({ ownerType })}
        />
      </Stack>
      <RefreshButton loading={loading} onClick={onRefresh} />
    </Stack>
  );
}

export function toAdminLedgerFilters(filters: AdminWalletLedgerFilterState): WalletLedgerEntryFilters {
  return {
    category: filters.category === ALL_FILTER_VALUE ? undefined : filters.category,
    reason_code: filters.reason === ALL_FILTER_VALUE ? undefined : filters.reason,
    owner_type: filters.ownerType === ALL_FILTER_VALUE ? undefined : filters.ownerType,
  };
}

function reasonOptions(t: TFunction<'admin'>, category: string) {
  return [
    { value: ALL_FILTER_VALUE, label: t('wallet.filters.allReasons') },
    ...reasonValues(category).map((value) => ({ value, label: walletTransactionReasonLabel(t, value) })),
  ];
}

function reasonValues(category: string) {
  const values = [
    'topup_admin_manual',
    'topup_gateway',
    'topup_redeem_code',
    'gift_initial',
    'gift_campaign',
    'gift_referral_commission',
    'adjust_admin',
    'adjust_system',
    'llm_model_usage',
    'refund_out',
    'refund_revert',
  ];

  if (category === ALL_FILTER_VALUE) {
    return values;
  }

  return values.filter((value) => value.startsWith(reasonPrefix(category)));
}

function reasonPrefix(category: string) {
  return category === 'recharge' ? 'topup_' : `${category}_`;
}

function ownerTypeOptions(t: TFunction<'admin'>) {
  return [
    { value: ALL_FILTER_VALUE, label: t('wallet.filters.allOwnerTypes') },
    { value: 'user', label: t('wallet.ownerTypes.user') },
  ];
}

function FilterSelect({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: Array<{ value: string; label: string }>;
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

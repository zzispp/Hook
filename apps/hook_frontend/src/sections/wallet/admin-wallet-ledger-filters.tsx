'use client';

import type { TFunction } from 'i18next';
import type { WalletLedgerRangePreset, WalletLedgerEntryFilters } from 'src/actions/wallet';

import dayjs from 'dayjs';

import Stack from '@mui/material/Stack';
import Switch from '@mui/material/Switch';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import { DatePicker } from '@mui/x-date-pickers/DatePicker';
import FormControlLabel from '@mui/material/FormControlLabel';

import { RefreshButton } from '../admin/shared';
import { walletStaticFilterOptions } from './wallet-filters';
import { walletTransactionReasonLabel } from './wallet-display';
import { ALL_FILTER_VALUE, FILTER_CONTROL_WIDTH } from './wallet-constants';

export type AdminWalletLedgerFilterState = {
  category: string;
  reason: string;
  ownerType: string;
  rangePreset: WalletLedgerRangePreset;
  startDate?: string;
  endDate?: string;
  aggregateConsumption: boolean;
};

export const DEFAULT_ADMIN_LEDGER_FILTERS: AdminWalletLedgerFilterState = {
  category: ALL_FILTER_VALUE,
  reason: ALL_FILTER_VALUE,
  ownerType: ALL_FILTER_VALUE,
  rangePreset: 'all',
  aggregateConsumption: false,
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
  const updateRangePreset = (rangePreset: WalletLedgerRangePreset) => {
    onChange({ ...filters, ...rangePresetPatch(rangePreset, filters) });
  };

  return (
    <Stack
      direction={{ xs: 'column', md: 'row' }}
      justifyContent="space-between"
      spacing={1.5}
      sx={{ p: 2.5 }}
    >
      <LedgerFilterControls t={t} filters={filters} onPatch={patchFilters} onRangePreset={updateRangePreset} />
      <LedgerToolbarActions t={t} loading={loading} filters={filters} onPatch={patchFilters} onRefresh={onRefresh} />
    </Stack>
  );
}

function LedgerFilterControls({
  t,
  filters,
  onPatch,
  onRangePreset,
}: {
  t: TFunction<'admin'>;
  filters: AdminWalletLedgerFilterState;
  onPatch: (filters: Partial<AdminWalletLedgerFilterState>) => void;
  onRangePreset: (rangePreset: WalletLedgerRangePreset) => void;
}) {
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={1.5}>
      {!filters.aggregateConsumption ? <CategoryReasonFilters t={t} filters={filters} onPatch={onPatch} /> : null}
      <FilterSelect
        label={t('wallet.filters.ownerType')}
        value={filters.ownerType}
        options={ownerTypeOptions(t)}
        onChange={(ownerType) => onPatch({ ownerType })}
      />
      <FilterSelect
        label={t('wallet.filters.dateRange')}
        value={filters.rangePreset}
        options={rangeOptions(t)}
        onChange={(value) => onRangePreset(value as WalletLedgerRangePreset)}
      />
      {filters.rangePreset === 'custom' ? <CustomRangeFields t={t} filters={filters} onChange={onPatch} /> : null}
    </Stack>
  );
}

function CategoryReasonFilters({
  t,
  filters,
  onPatch,
}: {
  t: TFunction<'admin'>;
  filters: AdminWalletLedgerFilterState;
  onPatch: (filters: Partial<AdminWalletLedgerFilterState>) => void;
}) {
  return (
    <>
      <FilterSelect
        label={t('wallet.filters.category')}
        value={filters.category}
        options={walletStaticFilterOptions(t).categories}
        onChange={(category) => onPatch({ category, reason: ALL_FILTER_VALUE })}
      />
      <FilterSelect
        label={t('wallet.filters.reason')}
        value={filters.reason}
        options={reasonOptions(t, filters.category)}
        onChange={(reason) => onPatch({ reason })}
      />
    </>
  );
}

function LedgerToolbarActions({
  t,
  loading,
  filters,
  onPatch,
  onRefresh,
}: {
  t: TFunction<'admin'>;
  loading: boolean;
  filters: AdminWalletLedgerFilterState;
  onPatch: (filters: Partial<AdminWalletLedgerFilterState>) => void;
  onRefresh: VoidFunction;
}) {
  return (
    <Stack direction="row" alignItems="center" spacing={1.5}>
      <FormControlLabel
        control={
          <Switch
            checked={filters.aggregateConsumption}
            onChange={(event) => onPatch({ aggregateConsumption: event.target.checked })}
          />
        }
        label={t('wallet.filters.aggregateConsumption')}
        sx={{ m: 0, whiteSpace: 'nowrap' }}
      />
      <RefreshButton loading={loading} onClick={onRefresh} />
    </Stack>
  );
}

export function toAdminLedgerFilters(filters: AdminWalletLedgerFilterState): WalletLedgerEntryFilters {
  return {
    category: filters.category === ALL_FILTER_VALUE ? undefined : filters.category,
    reason_code: filters.reason === ALL_FILTER_VALUE ? undefined : filters.reason,
    owner_type: filters.ownerType === ALL_FILTER_VALUE ? undefined : filters.ownerType,
    ...rangeFilters(filters),
  };
}

export function toAdminConsumptionSummaryFilters(filters: AdminWalletLedgerFilterState): WalletLedgerEntryFilters {
  return {
    owner_type: filters.ownerType === ALL_FILTER_VALUE ? undefined : filters.ownerType,
    ...rangeFilters(filters),
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

function rangeOptions(t: TFunction<'admin'>) {
  const values: WalletLedgerRangePreset[] = ['all', 'today', 'last7days', 'last30days', 'custom'];
  return values.map((value) => ({ value, label: t(`wallet.filters.rangePresets.${value}`) }));
}

function rangeFilters(filters: AdminWalletLedgerFilterState) {
  return {
    range_preset: filters.rangePreset,
    start_date: filters.rangePreset === 'custom' ? filters.startDate : undefined,
    end_date: filters.rangePreset === 'custom' ? filters.endDate : undefined,
  };
}

function rangePresetPatch(
  rangePreset: WalletLedgerRangePreset,
  current: AdminWalletLedgerFilterState
): Partial<AdminWalletLedgerFilterState> {
  if (rangePreset !== 'custom') {
    return { rangePreset };
  }
  return {
    rangePreset,
    startDate: current.startDate ?? dayjs().subtract(29, 'day').format('YYYY-MM-DD'),
    endDate: current.endDate ?? dayjs().format('YYYY-MM-DD'),
  };
}

function CustomRangeFields({
  t,
  filters,
  onChange,
}: {
  t: TFunction<'admin'>;
  filters: AdminWalletLedgerFilterState;
  onChange: (filters: Partial<AdminWalletLedgerFilterState>) => void;
}) {
  return (
    <>
      <DatePicker
        label={t('wallet.filters.startDate')}
        value={filters.startDate ? dayjs(filters.startDate) : null}
        slotProps={{ textField: { sx: { minWidth: FILTER_CONTROL_WIDTH } } }}
        onChange={(value) => onChange({ startDate: value?.format('YYYY-MM-DD') })}
      />
      <DatePicker
        label={t('wallet.filters.endDate')}
        value={filters.endDate ? dayjs(filters.endDate) : null}
        slotProps={{ textField: { sx: { minWidth: FILTER_CONTROL_WIDTH } } }}
        onChange={(value) => onChange({ endDate: value?.format('YYYY-MM-DD') })}
      />
    </>
  );
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

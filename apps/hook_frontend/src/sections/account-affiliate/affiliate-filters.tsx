'use client';

import type { TFunction } from 'i18next';
import type {
  AffiliateReferralFilters,
  AffiliateCommissionFilters,
} from 'src/actions/account-affiliate';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';
import InputAdornment from '@mui/material/InputAdornment';

import { Iconify } from 'src/components/iconify';

const EMPTY = '';
const BUTTON_SX = { minWidth: 112, whiteSpace: 'nowrap', width: { xs: '100%', md: 'auto' } };

export type ReferralFilterState = {
  search: string;
  referredStart: string;
  referredEnd: string;
};

export type CommissionFilterState = {
  referredSearch: string;
  rechargeOrderNo: string;
  startAt: string;
  endAt: string;
  minCommissionAmount: string;
  maxCommissionAmount: string;
};

export const DEFAULT_REFERRAL_FILTERS: ReferralFilterState = {
  search: EMPTY,
  referredStart: EMPTY,
  referredEnd: EMPTY,
};

export const DEFAULT_COMMISSION_FILTERS: CommissionFilterState = {
  referredSearch: EMPTY,
  rechargeOrderNo: EMPTY,
  startAt: EMPTY,
  endAt: EMPTY,
  minCommissionAmount: EMPTY,
  maxCommissionAmount: EMPTY,
};

export function ReferralFiltersToolbar({
  t,
  filters,
  onChange,
}: {
  t: TFunction<'admin'>;
  filters: ReferralFilterState;
  onChange: (filters: ReferralFilterState) => void;
}) {
  const patch = (next: Partial<ReferralFilterState>) => onChange({ ...filters, ...next });
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2.5 }}>
      <SearchField
        label={t('affiliateCenter.filters.search')}
        value={filters.search}
        placeholder={t('affiliateCenter.filters.referralSearchPlaceholder')}
        onChange={(search) => patch({ search })}
      />
      <DateField
        label={t('affiliateCenter.filters.start')}
        value={filters.referredStart}
        onChange={(referredStart) => patch({ referredStart })}
      />
      <DateField
        label={t('affiliateCenter.filters.end')}
        value={filters.referredEnd}
        onChange={(referredEnd) => patch({ referredEnd })}
      />
    </Stack>
  );
}

export function CommissionFiltersToolbar({
  t,
  filters,
  onChange,
  onExport,
}: {
  t: TFunction<'admin'>;
  filters: CommissionFilterState;
  onChange: (filters: CommissionFilterState) => void;
  onExport: VoidFunction;
}) {
  const patch = (next: Partial<CommissionFilterState>) => onChange({ ...filters, ...next });
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2.5, flexWrap: 'wrap' }}>
      <SearchField
        label={t('affiliateCenter.filters.referredSearch')}
        value={filters.referredSearch}
        placeholder={t('affiliateCenter.filters.referredSearchPlaceholder')}
        onChange={(referredSearch) => patch({ referredSearch })}
      />
      <TextField
        fullWidth
        label={t('affiliateCenter.filters.orderNo')}
        value={filters.rechargeOrderNo}
        onChange={(event) => patch({ rechargeOrderNo: event.target.value })}
      />
      <DateField label={t('affiliateCenter.filters.start')} value={filters.startAt} onChange={(startAt) => patch({ startAt })} />
      <DateField label={t('affiliateCenter.filters.end')} value={filters.endAt} onChange={(endAt) => patch({ endAt })} />
      <NumberField
        label={t('affiliateCenter.filters.minCommission')}
        value={filters.minCommissionAmount}
        onChange={(minCommissionAmount) => patch({ minCommissionAmount })}
      />
      <NumberField
        label={t('affiliateCenter.filters.maxCommission')}
        value={filters.maxCommissionAmount}
        onChange={(maxCommissionAmount) => patch({ maxCommissionAmount })}
      />
      <Button
        variant="outlined"
        startIcon={<Iconify icon="solar:download-bold" />}
        sx={BUTTON_SX}
        onClick={onExport}
      >
        {t('affiliateCenter.actions.exportCsv')}
      </Button>
    </Stack>
  );
}

export function toReferralFilters(filters: ReferralFilterState): AffiliateReferralFilters {
  return {
    search: optionalText(filters.search),
    referred_start: optionalText(filters.referredStart),
    referred_end: optionalText(filters.referredEnd),
  };
}

export function toCommissionFilters(filters: CommissionFilterState): AffiliateCommissionFilters {
  return {
    referred_search: optionalText(filters.referredSearch),
    recharge_order_no: optionalText(filters.rechargeOrderNo),
    start_at: optionalText(filters.startAt),
    end_at: optionalText(filters.endAt),
    min_commission_amount: optionalNumber(filters.minCommissionAmount),
    max_commission_amount: optionalNumber(filters.maxCommissionAmount),
  };
}

function SearchField({ label, value, placeholder, onChange }: TextProps) {
  return (
    <TextField
      fullWidth
      label={label}
      value={value}
      placeholder={placeholder}
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

function DateField({ label, value, onChange }: TextProps) {
  return (
    <TextField
      label={label}
      type="datetime-local"
      value={value}
      sx={{ minWidth: 220 }}
      InputLabelProps={{ shrink: true }}
      onChange={(event) => onChange(event.target.value)}
    />
  );
}

function NumberField({ label, value, onChange }: TextProps) {
  return (
    <TextField
      label={label}
      type="number"
      value={value}
      sx={{ minWidth: 180 }}
      onChange={(event) => onChange(event.target.value)}
    />
  );
}

function optionalText(value: string) {
  const trimmed = value.trim();
  return trimmed ? trimmed : undefined;
}

function optionalNumber(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return undefined;
  const number = Number(trimmed);
  return Number.isFinite(number) ? number : undefined;
}

type TextProps = {
  label: string;
  value: string;
  placeholder?: string;
  onChange: (value: string) => void;
};

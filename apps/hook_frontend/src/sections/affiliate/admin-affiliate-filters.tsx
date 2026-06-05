'use client';

import type { TFunction } from 'i18next';
import type {
  AdminAffiliateReportFilters,
  AdminAffiliateRelationFilters,
  AdminAffiliateCommissionFilters,
} from 'src/actions/affiliate';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import InputAdornment from '@mui/material/InputAdornment';

import { Iconify } from 'src/components/iconify';

const EMPTY = '';
const BUTTON_SX = { minWidth: 112, whiteSpace: 'nowrap', width: { xs: '100%', md: 'auto' } };

export type RelationFilterState = {
  userSearch: string;
  referrerSearch: string;
  hasReferrer: string;
  referredStart: string;
  referredEnd: string;
};

export type CommissionFilterState = {
  referrerSearch: string;
  referredSearch: string;
  rechargeOrderId: string;
  startAt: string;
  endAt: string;
  minCommissionAmount: string;
  maxCommissionAmount: string;
};

export type ReportFilterState = {
  startDate: string;
  endDate: string;
  referrerSearch: string;
  referredSearch: string;
};

export const DEFAULT_RELATION_FILTERS: RelationFilterState = {
  userSearch: EMPTY,
  referrerSearch: EMPTY,
  hasReferrer: EMPTY,
  referredStart: EMPTY,
  referredEnd: EMPTY,
};

export const DEFAULT_COMMISSION_FILTERS: CommissionFilterState = {
  referrerSearch: EMPTY,
  referredSearch: EMPTY,
  rechargeOrderId: EMPTY,
  startAt: EMPTY,
  endAt: EMPTY,
  minCommissionAmount: EMPTY,
  maxCommissionAmount: EMPTY,
};

export const DEFAULT_REPORT_FILTERS: ReportFilterState = {
  startDate: EMPTY,
  endDate: EMPTY,
  referrerSearch: EMPTY,
  referredSearch: EMPTY,
};

export function RelationFiltersToolbar({
  t,
  filters,
  onChange,
}: {
  t: TFunction<'admin'>;
  filters: RelationFilterState;
  onChange: (filters: RelationFilterState) => void;
}) {
  const patch = (next: Partial<RelationFilterState>) => onChange({ ...filters, ...next });
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2.5 }}>
      <SearchField
        label={t('adminAffiliates.filters.userSearch')}
        value={filters.userSearch}
        placeholder={t('adminAffiliates.filters.userSearchPlaceholder')}
        onChange={(userSearch) => patch({ userSearch })}
      />
      <SearchField
        label={t('adminAffiliates.filters.referrerSearch')}
        value={filters.referrerSearch}
        placeholder={t('adminAffiliates.filters.referrerSearchPlaceholder')}
        onChange={(referrerSearch) => patch({ referrerSearch })}
      />
      <HasReferrerSelect t={t} value={filters.hasReferrer} onChange={(hasReferrer) => patch({ hasReferrer })} />
      <DateField label={t('adminAffiliates.filters.start')} value={filters.referredStart} onChange={(referredStart) => patch({ referredStart })} />
      <DateField label={t('adminAffiliates.filters.end')} value={filters.referredEnd} onChange={(referredEnd) => patch({ referredEnd })} />
    </Stack>
  );
}

export function CommissionFiltersToolbar({
  t,
  filters,
  onChange,
}: {
  t: TFunction<'admin'>;
  filters: CommissionFilterState;
  onChange: (filters: CommissionFilterState) => void;
}) {
  const patch = (next: Partial<CommissionFilterState>) => onChange({ ...filters, ...next });
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2.5, flexWrap: 'wrap' }}>
      <SearchField label={t('adminAffiliates.filters.referrerSearch')} value={filters.referrerSearch} onChange={(referrerSearch) => patch({ referrerSearch })} />
      <SearchField label={t('adminAffiliates.filters.referredSearch')} value={filters.referredSearch} onChange={(referredSearch) => patch({ referredSearch })} />
      <TextField fullWidth label={t('adminAffiliates.filters.orderId')} value={filters.rechargeOrderId} onChange={(event) => patch({ rechargeOrderId: event.target.value })} />
      <DateField label={t('adminAffiliates.filters.start')} value={filters.startAt} onChange={(startAt) => patch({ startAt })} />
      <DateField label={t('adminAffiliates.filters.end')} value={filters.endAt} onChange={(endAt) => patch({ endAt })} />
      <NumberField label={t('adminAffiliates.filters.minCommission')} value={filters.minCommissionAmount} onChange={(minCommissionAmount) => patch({ minCommissionAmount })} />
      <NumberField label={t('adminAffiliates.filters.maxCommission')} value={filters.maxCommissionAmount} onChange={(maxCommissionAmount) => patch({ maxCommissionAmount })} />
    </Stack>
  );
}

export function ReportFiltersToolbar({
  t,
  filters,
  onChange,
  onExportDetails,
  onExportDaily,
  onExportReferrers,
}: {
  t: TFunction<'admin'>;
  filters: ReportFilterState;
  onChange: (filters: ReportFilterState) => void;
  onExportDetails: VoidFunction;
  onExportDaily: VoidFunction;
  onExportReferrers: VoidFunction;
}) {
  const patch = (next: Partial<ReportFilterState>) => onChange({ ...filters, ...next });
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2.5, flexWrap: 'wrap' }}>
      <DateField label={t('adminAffiliates.filters.start')} value={filters.startDate} onChange={(startDate) => patch({ startDate })} />
      <DateField label={t('adminAffiliates.filters.end')} value={filters.endDate} onChange={(endDate) => patch({ endDate })} />
      <SearchField label={t('adminAffiliates.filters.referrerSearch')} value={filters.referrerSearch} onChange={(referrerSearch) => patch({ referrerSearch })} />
      <SearchField label={t('adminAffiliates.filters.referredSearch')} value={filters.referredSearch} onChange={(referredSearch) => patch({ referredSearch })} />
      <ExportButton label={t('adminAffiliates.actions.exportDetails')} onClick={onExportDetails} />
      <ExportButton label={t('adminAffiliates.actions.exportDaily')} onClick={onExportDaily} />
      <ExportButton label={t('adminAffiliates.actions.exportReferrers')} onClick={onExportReferrers} />
    </Stack>
  );
}

export function toRelationFilters(filters: RelationFilterState): AdminAffiliateRelationFilters {
  return {
    user_search: optionalText(filters.userSearch),
    referrer_search: optionalText(filters.referrerSearch),
    has_referrer: optionalBool(filters.hasReferrer),
    referred_start: optionalText(filters.referredStart),
    referred_end: optionalText(filters.referredEnd),
  };
}

export function toCommissionFilters(filters: CommissionFilterState): AdminAffiliateCommissionFilters {
  return {
    referrer_search: optionalText(filters.referrerSearch),
    referred_search: optionalText(filters.referredSearch),
    recharge_order_id: optionalText(filters.rechargeOrderId),
    start_at: optionalText(filters.startAt),
    end_at: optionalText(filters.endAt),
    min_commission_amount: optionalNumber(filters.minCommissionAmount),
    max_commission_amount: optionalNumber(filters.maxCommissionAmount),
  };
}

export function toReportFilters(filters: ReportFilterState): AdminAffiliateReportFilters {
  return {
    start_date: optionalText(filters.startDate),
    end_date: optionalText(filters.endDate),
    referrer_search: optionalText(filters.referrerSearch),
    referred_search: optionalText(filters.referredSearch),
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

function HasReferrerSelect({ t, value, onChange }: { t: TFunction<'admin'>; value: string; onChange: (value: string) => void }) {
  return (
    <TextField select label={t('adminAffiliates.filters.hasReferrer')} value={value} sx={{ minWidth: 160 }} onChange={(event) => onChange(event.target.value)}>
      <MenuItem value="">{t('adminAffiliates.filters.all')}</MenuItem>
      <MenuItem value="true">{t('adminAffiliates.filters.withReferrer')}</MenuItem>
      <MenuItem value="false">{t('adminAffiliates.filters.withoutReferrer')}</MenuItem>
    </TextField>
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

function ExportButton({ label, onClick }: { label: string; onClick: VoidFunction }) {
  return (
    <Button variant="outlined" startIcon={<Iconify icon="solar:download-bold" />} sx={BUTTON_SX} onClick={onClick}>
      {label}
    </Button>
  );
}

function optionalText(value: string) {
  return value.trim() || undefined;
}

function optionalBool(value: string) {
  if (value === 'true') return true;
  if (value === 'false') return false;
  return undefined;
}

function optionalNumber(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return undefined;
  return Number(trimmed);
}

type TextProps = {
  label: string;
  value: string;
  placeholder?: string;
  onChange: (value: string) => void;
};

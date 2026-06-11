'use client';

import type { TFunction } from 'i18next';
import type { TextFieldProps } from '@mui/material/TextField';
import type { RechargeOrderDatePreset } from 'src/types/recharge';
import type {
  RechargeOrderFilters,
  PaymentCallbackFilters,
  RechargePackageFilters,
} from 'src/actions/recharge';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import InputAdornment from '@mui/material/InputAdornment';

import { Iconify } from 'src/components/iconify';

const EMPTY_STATUS_FILTER = '';
const TOOLBAR_BUTTON_SX = {
  flexShrink: 0,
  minWidth: 112,
  whiteSpace: 'nowrap',
  width: { xs: '100%', md: 'auto' },
};

export type RechargePackageFilterState = {
  search: string;
  status: string;
};

export type RechargeOrderFilterState = {
  search: string;
  status: string;
  datePreset: RechargeOrderDatePreset;
  startDate: string;
  endDate: string;
  summaryEnabled: boolean;
};

export type PaymentCallbackFilterState = {
  search: string;
  status: string;
};

export const DEFAULT_RECHARGE_PACKAGE_FILTERS: RechargePackageFilterState = {
  search: '',
  status: EMPTY_STATUS_FILTER,
};

export const DEFAULT_RECHARGE_ORDER_FILTERS: RechargeOrderFilterState = {
  search: '',
  status: EMPTY_STATUS_FILTER,
  datePreset: 'all',
  startDate: '',
  endDate: '',
  summaryEnabled: false,
};

export const DEFAULT_PAYMENT_CALLBACK_FILTERS: PaymentCallbackFilterState = {
  search: '',
  status: EMPTY_STATUS_FILTER,
};

export function RechargePackageToolbar({
  t,
  filters,
  busy,
  onChange,
  onCreate,
}: {
  t: TFunction<'admin'>;
  filters: RechargePackageFilterState;
  busy: boolean;
  onChange: (filters: RechargePackageFilterState) => void;
  onCreate: VoidFunction;
}) {
  const patchFilters = (patch: Partial<RechargePackageFilterState>) =>
    onChange({ ...filters, ...patch });

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2.5 }}>
      <SearchField
        value={filters.search}
        placeholder={t('adminRecharges.filters.searchPackages')}
        onChange={(search) => patchFilters({ search })}
      />
      <TextField
        select
        label={t('common.status')}
        value={filters.status}
        sx={{ minWidth: 160 }}
        InputLabelProps={{ shrink: true }}
        SelectProps={packageStatusSelectProps(t)}
        onChange={(event) => patchFilters({ status: event.target.value })}
      >
        {packageStatusOptions(t)}
      </TextField>
      <Button
        variant="contained"
        disabled={busy}
        startIcon={<Iconify icon="solar:add-circle-bold" />}
        sx={TOOLBAR_BUTTON_SX}
        onClick={onCreate}
      >
        {t('adminRecharges.actions.createPackage')}
      </Button>
    </Stack>
  );
}

export function PaymentCallbackToolbar({
  t,
  filters,
  onChange,
}: {
  t: TFunction<'admin'>;
  filters: PaymentCallbackFilterState;
  onChange: (filters: PaymentCallbackFilterState) => void;
}) {
  const patchFilters = (patch: Partial<PaymentCallbackFilterState>) =>
    onChange({ ...filters, ...patch });

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2.5 }}>
      <SearchField
        value={filters.search}
        placeholder={t('adminRecharges.filters.searchCallbacks')}
        onChange={(search) => patchFilters({ search })}
      />
      <TextField
        select
        label={t('common.status')}
        value={filters.status}
        sx={{ minWidth: 180 }}
        InputLabelProps={{ shrink: true }}
        SelectProps={callbackStatusSelectProps(t)}
        onChange={(event) => patchFilters({ status: event.target.value })}
      >
        {callbackStatusOptions(t)}
      </TextField>
    </Stack>
  );
}

export function toRechargePackageFilters(
  filters: RechargePackageFilterState
): RechargePackageFilters {
  return {
    search: filters.search.trim() || undefined,
    status: filters.status || undefined,
  };
}

export function toRechargeOrderFilters(filters: RechargeOrderFilterState): RechargeOrderFilters {
  return {
    search: filters.search.trim() || undefined,
    status: filters.status || undefined,
    date_preset: filters.datePreset,
    start_date: filters.datePreset === 'custom' ? filters.startDate || undefined : undefined,
    end_date: filters.datePreset === 'custom' ? filters.endDate || undefined : undefined,
    tz_offset_minutes: -new Date().getTimezoneOffset(),
  };
}

export function toPaymentCallbackFilters(
  filters: PaymentCallbackFilterState
): PaymentCallbackFilters {
  return {
    search: filters.search.trim() || undefined,
    status: filters.status || undefined,
  };
}

function SearchField({
  value,
  placeholder,
  onChange,
}: {
  value: string;
  placeholder: string;
  onChange: (value: string) => void;
}) {
  return (
    <TextField
      fullWidth
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

function packageStatusOptions(t: TFunction<'admin'>) {
  return [
    <MenuItem key="all" value={EMPTY_STATUS_FILTER}>
      {t('filters.allStatuses')}
    </MenuItem>,
    <MenuItem key="active" value="active">
      {t('adminRecharges.status.package.active')}
    </MenuItem>,
    <MenuItem key="disabled" value="disabled">
      {t('adminRecharges.status.package.disabled')}
    </MenuItem>,
  ];
}

function callbackStatusOptions(t: TFunction<'admin'>) {
  return [
    <MenuItem key="all" value={EMPTY_STATUS_FILTER}>
      {t('filters.allStatuses')}
    </MenuItem>,
    <MenuItem key="received" value="received">
      {t('adminRecharges.status.callback.received')}
    </MenuItem>,
    <MenuItem key="processed" value="processed">
      {t('adminRecharges.status.callback.processed')}
    </MenuItem>,
    <MenuItem key="ignored" value="ignored">
      {t('adminRecharges.status.callback.ignored')}
    </MenuItem>,
    <MenuItem key="failed" value="failed">
      {t('adminRecharges.status.callback.failed')}
    </MenuItem>,
  ];
}

function packageStatusSelectProps(t: TFunction<'admin'>): TextFieldProps['SelectProps'] {
  return {
    displayEmpty: true,
    renderValue: (selected) =>
      selected
        ? t(`adminRecharges.status.package.${String(selected)}`)
        : t('filters.allStatuses'),
  };
}

function callbackStatusSelectProps(t: TFunction<'admin'>): TextFieldProps['SelectProps'] {
  return {
    displayEmpty: true,
    renderValue: (selected) =>
      selected
        ? t(`adminRecharges.status.callback.${String(selected)}`)
        : t('filters.allStatuses'),
  };
}

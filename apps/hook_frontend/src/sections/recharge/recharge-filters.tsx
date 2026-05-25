'use client';

import type { TFunction } from 'i18next';
import type { TextFieldProps } from '@mui/material/TextField';
import type { RechargeOrderFilters, RechargePackageFilters } from 'src/actions/recharge';

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
};

export const DEFAULT_RECHARGE_PACKAGE_FILTERS: RechargePackageFilterState = {
  search: '',
  status: EMPTY_STATUS_FILTER,
};

export const DEFAULT_RECHARGE_ORDER_FILTERS: RechargeOrderFilterState = {
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
        <PackageStatusOptions t={t} />
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

export function RechargeOrderToolbar({
  t,
  filters,
  onChange,
}: {
  t: TFunction<'admin'>;
  filters: RechargeOrderFilterState;
  onChange: (filters: RechargeOrderFilterState) => void;
}) {
  const patchFilters = (patch: Partial<RechargeOrderFilterState>) =>
    onChange({ ...filters, ...patch });

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2.5 }}>
      <SearchField
        value={filters.search}
        placeholder={t('adminRecharges.filters.searchOrders')}
        onChange={(search) => patchFilters({ search })}
      />
      <TextField
        select
        label={t('common.status')}
        value={filters.status}
        sx={{ minWidth: 180 }}
        InputLabelProps={{ shrink: true }}
        SelectProps={orderStatusSelectProps(t)}
        onChange={(event) => patchFilters({ status: event.target.value })}
      >
        <OrderStatusOptions t={t} />
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

function PackageStatusOptions({ t }: { t: TFunction<'admin'> }) {
  return (
    <>
      <MenuItem value={EMPTY_STATUS_FILTER}>{t('filters.allStatuses')}</MenuItem>
      <MenuItem value="active">{t('adminRecharges.status.package.active')}</MenuItem>
      <MenuItem value="disabled">{t('adminRecharges.status.package.disabled')}</MenuItem>
    </>
  );
}

function OrderStatusOptions({ t }: { t: TFunction<'admin'> }) {
  return (
    <>
      <MenuItem value={EMPTY_STATUS_FILTER}>{t('filters.allStatuses')}</MenuItem>
      <MenuItem value="pending">{t('adminRecharges.status.order.pending')}</MenuItem>
      <MenuItem value="expired">{t('adminRecharges.status.order.expired')}</MenuItem>
      <MenuItem value="paid">{t('adminRecharges.status.order.paid')}</MenuItem>
      <MenuItem value="cancelled">{t('adminRecharges.status.order.cancelled')}</MenuItem>
      <MenuItem value="failed">{t('adminRecharges.status.order.failed')}</MenuItem>
    </>
  );
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

function orderStatusSelectProps(t: TFunction<'admin'>): TextFieldProps['SelectProps'] {
  return {
    displayEmpty: true,
    renderValue: (selected) =>
      selected ? t(`adminRecharges.status.order.${String(selected)}`) : t('filters.allStatuses'),
  };
}


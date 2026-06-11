'use client';

import type { TFunction } from 'i18next';
import type { RechargeOrderDatePreset } from 'src/types/recharge';
import type { RechargeOrderFilterState } from './recharge-filters';

import dayjs from 'dayjs';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Switch from '@mui/material/Switch';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import InputAdornment from '@mui/material/InputAdornment';
import { DatePicker } from '@mui/x-date-pickers/DatePicker';
import FormControlLabel from '@mui/material/FormControlLabel';

import { Iconify } from 'src/components/iconify';

const EMPTY_STATUS_FILTER = '';
const DATE_PRESETS: RechargeOrderDatePreset[] = [
  'all',
  'today',
  'last7days',
  'last30days',
  'custom',
];

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
    <Box sx={{ p: 2.5 }}>
      <Stack spacing={2}>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
          <SearchField
            value={filters.search}
            placeholder={t('adminRecharges.filters.searchOrders')}
            onChange={(search) => patchFilters({ search })}
          />
          <OrderStatusSelect
            t={t}
            value={filters.status}
            onChange={(status) => patchFilters({ status })}
          />
          <SummarySwitch
            t={t}
            checked={filters.summaryEnabled}
            onChange={(summaryEnabled) => patchFilters({ summaryEnabled })}
          />
        </Stack>
        <Stack direction={{ xs: 'column', lg: 'row' }} spacing={2} alignItems={{ lg: 'center' }}>
          <DatePresetSelect
            t={t}
            value={filters.datePreset}
            onChange={(datePreset) => onChange(nextDatePresetFilters(filters, datePreset))}
          />
          {filters.datePreset === 'custom' ? (
            <CustomDateRange t={t} filters={filters} onChange={patchFilters} />
          ) : null}
        </Stack>
      </Stack>
    </Box>
  );
}

function OrderStatusSelect({
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
      select
      size="small"
      label={t('common.status')}
      value={value}
      sx={{ minWidth: 180, width: { xs: '100%', md: 'auto' } }}
      InputLabelProps={{ shrink: true }}
      SelectProps={{
        displayEmpty: true,
        renderValue: (selected) =>
          selected
            ? t(`adminRecharges.status.order.${String(selected)}`)
            : t('filters.allStatuses'),
      }}
      onChange={(event) => onChange(event.target.value)}
    >
      {orderStatusOptions(t)}
    </TextField>
  );
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
      size="small"
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

function SummarySwitch({
  t,
  checked,
  onChange,
}: {
  t: TFunction<'admin'>;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <FormControlLabel
      control={<Switch checked={checked} onChange={(event) => onChange(event.target.checked)} />}
      label={<Typography variant="body2">{t('adminRecharges.filters.userSummary')}</Typography>}
      sx={{ flexShrink: 0, minWidth: { xs: 1, md: 140 }, m: 0 }}
    />
  );
}

function DatePresetSelect({
  t,
  value,
  onChange,
}: {
  t: TFunction<'admin'>;
  value: RechargeOrderDatePreset;
  onChange: (value: RechargeOrderDatePreset) => void;
}) {
  return (
    <TextField
      select
      size="small"
      label={t('adminRecharges.filters.dateRange')}
      value={value}
      sx={{ width: { xs: '100%', sm: 220 }, flexShrink: 0 }}
      InputLabelProps={{ shrink: true }}
      SelectProps={{
        displayEmpty: true,
        renderValue: (selected) =>
          t(`adminRecharges.filters.datePresets.${String(selected)}`),
      }}
      onChange={(event) => onChange(event.target.value as RechargeOrderDatePreset)}
    >
      {DATE_PRESETS.map((preset) => (
        <MenuItem key={preset} value={preset}>
          {t(`adminRecharges.filters.datePresets.${preset}`)}
        </MenuItem>
      ))}
    </TextField>
  );
}

function CustomDateRange({
  t,
  filters,
  onChange,
}: {
  t: TFunction<'admin'>;
  filters: RechargeOrderFilterState;
  onChange: (patch: Partial<RechargeOrderFilterState>) => void;
}) {
  return (
    <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2} sx={{ minWidth: { lg: 340 } }}>
      <DatePicker
        label={t('adminRecharges.filters.startDate')}
        value={filters.startDate ? dayjs(filters.startDate) : null}
        slotProps={{ textField: { size: 'small' } }}
        onChange={(value) => onChange({ startDate: value?.format('YYYY-MM-DD') ?? '' })}
      />
      <DatePicker
        label={t('adminRecharges.filters.endDate')}
        value={filters.endDate ? dayjs(filters.endDate) : null}
        slotProps={{ textField: { size: 'small' } }}
        onChange={(value) => onChange({ endDate: value?.format('YYYY-MM-DD') ?? '' })}
      />
    </Stack>
  );
}

function orderStatusOptions(t: TFunction<'admin'>) {
  return [
    <MenuItem key="all" value={EMPTY_STATUS_FILTER}>
      {t('filters.allStatuses')}
    </MenuItem>,
    <MenuItem key="pending" value="pending">
      {t('adminRecharges.status.order.pending')}
    </MenuItem>,
    <MenuItem key="expired" value="expired">
      {t('adminRecharges.status.order.expired')}
    </MenuItem>,
    <MenuItem key="paid" value="paid">
      {t('adminRecharges.status.order.paid')}
    </MenuItem>,
    <MenuItem key="cancelled" value="cancelled">
      {t('adminRecharges.status.order.cancelled')}
    </MenuItem>,
    <MenuItem key="failed" value="failed">
      {t('adminRecharges.status.order.failed')}
    </MenuItem>,
  ];
}

function nextDatePresetFilters(
  filters: RechargeOrderFilterState,
  datePreset: RechargeOrderDatePreset
): RechargeOrderFilterState {
  if (datePreset !== 'custom') {
    return { ...filters, datePreset, startDate: '', endDate: '' };
  }
  return {
    ...filters,
    datePreset,
    startDate: filters.startDate || dayjs().format('YYYY-MM-DD'),
    endDate: filters.endDate || dayjs().format('YYYY-MM-DD'),
  };
}

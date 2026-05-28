'use client';

import type { TFunction } from 'i18next';
import type { DashboardCostAnalysisPreset } from 'src/types/dashboard';

import dayjs from 'dayjs';

import Stack from '@mui/material/Stack';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import InputLabel from '@mui/material/InputLabel';
import FormControl from '@mui/material/FormControl';
import { DatePicker } from '@mui/x-date-pickers/DatePicker';

export type AdminDashboardRangeFilters = {
  preset: DashboardCostAnalysisPreset;
  start_date?: string;
  end_date?: string;
};

const RANGE_PRESETS: DashboardCostAnalysisPreset[] = [
  'today',
  'yesterday',
  'last7days',
  'last30days',
  'last90days',
  'custom',
];

export function DashboardDateRangePicker({
  t,
  filters,
  onChange,
  inline = false,
}: {
  t: TFunction<'admin'>;
  filters: AdminDashboardRangeFilters;
  onChange: (filters: AdminDashboardRangeFilters) => void;
  inline?: boolean;
}) {
  const isCustom = filters.preset === 'custom';
  return (
    <Stack
      spacing={1}
      sx={{
        width: { xs: 1, md: 'auto' },
        display: 'grid',
        gridTemplateColumns: { xs: '1fr', sm: isCustom ? '180px 160px 160px' : '180px' },
        ...(inline ? { display: 'contents' } : {}),
      }}
    >
      <FormControl size="small">
        <InputLabel>{t('dashboard.stats.costAnalysis.range')}</InputLabel>
        <Select
          label={t('dashboard.stats.costAnalysis.range')}
          value={filters.preset}
          onChange={(event) => onChange(rangeForPreset(event.target.value as DashboardCostAnalysisPreset, filters))}
        >
          {RANGE_PRESETS.map((preset) => (
            <MenuItem key={preset} value={preset}>
              {t(`dashboard.stats.costAnalysis.presets.${preset}`)}
            </MenuItem>
          ))}
        </Select>
      </FormControl>
      {isCustom ? <CustomRangeFields t={t} filters={filters} onChange={onChange} /> : null}
    </Stack>
  );
}

function CustomRangeFields({
  t,
  filters,
  onChange,
}: {
  t: TFunction<'admin'>;
  filters: AdminDashboardRangeFilters;
  onChange: (filters: AdminDashboardRangeFilters) => void;
}) {
  return (
    <>
      <DatePicker
        label={t('dashboard.stats.costAnalysis.startDate')}
        value={filters.start_date ? dayjs(filters.start_date) : null}
        slotProps={{ textField: { size: 'small' } }}
        onChange={(value) => onChange({ ...filters, start_date: value?.format('YYYY-MM-DD') })}
      />
      <DatePicker
        label={t('dashboard.stats.costAnalysis.endDate')}
        value={filters.end_date ? dayjs(filters.end_date) : null}
        slotProps={{ textField: { size: 'small' } }}
        onChange={(value) => onChange({ ...filters, end_date: value?.format('YYYY-MM-DD') })}
      />
    </>
  );
}

function rangeForPreset(
  preset: DashboardCostAnalysisPreset,
  current: AdminDashboardRangeFilters
): AdminDashboardRangeFilters {
  if (preset === 'custom') {
    return {
      preset,
      start_date: current.start_date ?? dayjs().subtract(29, 'day').format('YYYY-MM-DD'),
      end_date: current.end_date ?? dayjs().format('YYYY-MM-DD'),
    };
  }
  return { preset };
}

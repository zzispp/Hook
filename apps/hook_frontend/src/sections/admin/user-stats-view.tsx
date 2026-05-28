'use client';

import type { TFunction } from 'i18next';
import type { SystemUser } from 'src/types/rbac';
import type { DashboardUserStatsFilters } from 'src/actions/dashboard';
import type { DashboardPreset, DashboardCostAnalysisPreset } from 'src/types/dashboard';

import dayjs from 'dayjs';
import { useState } from 'react';

import Stack from '@mui/material/Stack';
import TextField from '@mui/material/TextField';
import Autocomplete from '@mui/material/Autocomplete';

import { useUsers } from 'src/actions/rbac';
import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { AdminBreadcrumbs } from './shared';
import { DashboardUserStats } from '../overview/analytics/view/dashboard-user-stats';
import {
  DashboardDateRangePicker,
  type AdminDashboardRangeFilters,
} from './dashboard-date-range-picker';

const USER_PAGE_SIZE = 100;

export function UserStatsView() {
  const { t, currentLang } = useTranslate('admin');
  const users = useUsers(0, USER_PAGE_SIZE);
  const [range, setRange] = useState<AdminDashboardRangeFilters>(defaultRange());
  const [filters, setFilters] = useState<DashboardUserStatsFilters>({
    preset: 'today',
    metric: 'requests',
    leaderboardPage: 0,
    leaderboardPageSize: 5,
  });

  function updateRange(next: AdminDashboardRangeFilters) {
    setRange(next);
    setFilters((value) => ({ ...value, ...userStatsRangeParams(next), leaderboardPage: 0 }));
  }

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.userStats}
        action={
          <UserStatsFilters
            t={t}
            users={users.items}
            range={range}
            filters={filters}
            loading={users.isLoading}
            onRangeChange={updateRange}
            onChange={(next) => setFilters({ ...filters, ...next })}
          />
        }
      />
      <Stack spacing={3}>
        <DashboardUserStats
          t={t}
          locale={currentLang.numberFormat.code}
          filters={filters}
          onChange={setFilters}
        />
      </Stack>
    </DashboardContent>
  );
}

function UserStatsFilters({
  t,
  users,
  range,
  filters,
  loading,
  onChange,
  onRangeChange,
}: {
  t: TFunction<'admin'>;
  users: SystemUser[];
  range: AdminDashboardRangeFilters;
  filters: DashboardUserStatsFilters;
  loading: boolean;
  onChange: (filters: Partial<DashboardUserStatsFilters>) => void;
  onRangeChange: (range: AdminDashboardRangeFilters) => void;
}) {
  const isCustom = range.preset === 'custom';
  return (
    <Stack
      spacing={1}
      sx={{
        width: { xs: 1, md: 'auto' },
        display: 'grid',
        gridTemplateColumns: {
          xs: '1fr',
          sm: isCustom ? 'repeat(2, minmax(0, 1fr))' : 'repeat(3, minmax(0, 1fr))',
          lg: isCustom ? '180px 160px 160px 220px 220px' : '180px 220px 220px',
        },
      }}
    >
      <DashboardDateRangePicker inline t={t} filters={range} onChange={onRangeChange} />
      <UserSelect
        label={t('dashboard.stats.userStats.user')}
        users={users}
        value={filters.userId}
        loading={loading}
        onChange={(userId) => onChange({ userId })}
      />
      <UserSelect
        label={t('dashboard.stats.userStats.compareUser')}
        users={users}
        value={filters.compareUserId}
        loading={loading}
        onChange={(compareUserId) => onChange({ compareUserId })}
      />
    </Stack>
  );
}

function defaultRange(): AdminDashboardRangeFilters {
  return { preset: 'today' as DashboardCostAnalysisPreset };
}

function userStatsRangeParams(range: AdminDashboardRangeFilters) {
  if (range.preset === 'custom') {
    return { start_date: range.start_date, end_date: range.end_date, preset: undefined };
  }
  if (range.preset === 'yesterday') {
    const date = dayjs().subtract(1, 'day').format('YYYY-MM-DD');
    return { start_date: date, end_date: date, preset: undefined };
  }
  return { preset: userStatsPreset(range.preset), start_date: undefined, end_date: undefined };
}

function userStatsPreset(preset: DashboardCostAnalysisPreset): DashboardPreset {
  if (preset === 'last7days') return '7d';
  if (preset === 'last30days') return '30d';
  if (preset === 'last90days') return '90d';
  return 'today';
}

function UserSelect({
  label,
  users,
  value,
  loading,
  onChange,
}: {
  label: string;
  users: SystemUser[];
  value?: string;
  loading: boolean;
  onChange: (userId?: string) => void;
}) {
  const selected = users.find((user) => user.id === value) ?? null;
  return (
    <Autocomplete
      size="small"
      loading={loading}
      options={users}
      value={selected}
      getOptionLabel={(option) => option.username}
      isOptionEqualToValue={(option, selectedValue) => option.id === selectedValue.id}
      onChange={(_, option) => onChange(option?.id)}
      sx={{ minWidth: 0 }}
      renderInput={(params) => <TextField {...params} label={label} />}
    />
  );
}

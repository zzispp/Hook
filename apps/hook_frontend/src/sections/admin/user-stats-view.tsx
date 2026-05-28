'use client';

import type { TFunction } from 'i18next';
import type { SystemUser } from 'src/types/rbac';
import type { DashboardPreset } from 'src/types/dashboard';
import type { DashboardUserStatsFilters } from 'src/actions/dashboard';

import { useState } from 'react';

import Stack from '@mui/material/Stack';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import InputLabel from '@mui/material/InputLabel';
import FormControl from '@mui/material/FormControl';
import Autocomplete from '@mui/material/Autocomplete';

import { useUsers } from 'src/actions/rbac';
import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { AdminBreadcrumbs } from './shared';
import { DashboardUserStats } from '../overview/analytics/view/dashboard-user-stats';

const PRESETS: DashboardPreset[] = ['today', '7d', '30d', '90d'];
const USER_PAGE_SIZE = 100;

export function UserStatsView() {
  const { t, currentLang } = useTranslate('admin');
  const users = useUsers(0, USER_PAGE_SIZE);
  const [preset, setPreset] = useState<DashboardPreset>('today');
  const [filters, setFilters] = useState<DashboardUserStatsFilters>({
    preset: 'today',
    metric: 'requests',
    leaderboardPage: 0,
    leaderboardPageSize: 5,
  });

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.userStats}
        action={
          <UserStatsFilters
            t={t}
            users={users.items}
            preset={preset}
            filters={filters}
            loading={users.isLoading}
            onPresetChange={setPreset}
            onChange={(next) => setFilters({ ...filters, ...next })}
          />
        }
      />
      <Stack spacing={3}>
        <DashboardUserStats
          t={t}
          locale={currentLang.numberFormat.code}
          preset={preset}
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
  preset,
  filters,
  loading,
  onChange,
  onPresetChange,
}: {
  t: TFunction<'admin'>;
  users: SystemUser[];
  preset: DashboardPreset;
  filters: DashboardUserStatsFilters;
  loading: boolean;
  onChange: (filters: Partial<DashboardUserStatsFilters>) => void;
  onPresetChange: (preset: DashboardPreset) => void;
}) {
  return (
    <Stack
      spacing={1}
      sx={{
        width: { xs: 1, md: 'auto' },
        display: 'grid',
        gridTemplateColumns: { xs: '1fr', sm: 'repeat(3, minmax(0, 1fr))', lg: '150px 220px 220px' },
      }}
    >
      <FormControl size="small" sx={{ minWidth: 0 }}>
        <InputLabel>{t('dashboard.stats.userStats.interval')}</InputLabel>
        <Select
          label={t('dashboard.stats.userStats.interval')}
          value={preset}
          onChange={(event) => onPresetChange(event.target.value as DashboardPreset)}
        >
          {PRESETS.map((item) => (
            <MenuItem key={item} value={item}>
              {t(`dashboard.stats.presets.${item}`)}
            </MenuItem>
          ))}
        </Select>
      </FormControl>
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

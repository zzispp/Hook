'use client';

import type { DashboardPreset } from 'src/types/dashboard';
import type { DashboardActivityFilters } from 'src/actions/dashboard';

import { useState, useEffect } from 'react';

import Alert from '@mui/material/Alert';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import {
  useDashboardActivity,
  useDashboardOverview,
  useDashboardFilterOptions,
} from 'src/actions/dashboard';

import { useTable } from 'src/components/table';

import { useAuthContext } from 'src/auth/hooks';

import { KpiGrid } from './dashboard-kpi';
import { errorMessage } from './dashboard-format';
import { DashboardHeader } from './dashboard-header';
import { DashboardMainGrid } from './dashboard-main-grid';
import { PeriodSummaryGrid } from './dashboard-period-summary';

export function OverviewAnalyticsView() {
  const { t, currentLang } = useTranslate('admin');
  const { user } = useAuthContext();
  const [preset, setPreset] = useState<DashboardPreset>('today');
  const isAdmin = user?.role === 'admin';
  const [activityFilters, setActivityFilters] = useState<DashboardActivityFilters>(
    isAdmin ? { scope: 'global' } : { scope: 'me' }
  );
  const dailyTable = useTable({ defaultRowsPerPage: 10 });
  const overview = useDashboardOverview(preset, activityFilters, dailyTable.page, dailyTable.rowsPerPage);
  const activity = useDashboardActivity(activityFilters);
  const filterOptions = useDashboardFilterOptions(isAdmin);
  const locale = currentLang.numberFormat.code;
  const error = overview.error ?? activity.error ?? filterOptions.error;
  const statsLoading = overview.isLoading;
  const resetDailyTablePage = dailyTable.onResetPage;

  useEffect(() => {
    if (user === undefined) return;
    if (!isAdmin && activityFilters.scope !== 'me') {
      resetDailyTablePage();
      setActivityFilters({ scope: 'me' });
    }
    if (isAdmin && activityFilters.scope === 'me') {
      resetDailyTablePage();
      setActivityFilters({ scope: 'global' });
    }
  }, [activityFilters.scope, isAdmin, resetDailyTablePage, user]);

  function handlePresetChange(value: DashboardPreset) {
    resetDailyTablePage();
    setPreset(value);
  }

  function handleFiltersChange(value: DashboardActivityFilters) {
    resetDailyTablePage();
    setActivityFilters(value);
  }

  return (
    <DashboardContent maxWidth="xl">
      <DashboardHeader
        t={t}
        isAdmin={isAdmin}
        preset={preset}
        loading={overview.isValidating || activity.isValidating}
        filters={activityFilters}
        filterOptions={filterOptions.data}
        onPresetChange={handlePresetChange}
        onFiltersChange={handleFiltersChange}
        onRefresh={() => {
          void overview.refresh();
          void activity.refresh();
          void filterOptions.refresh();
        }}
      />

      {error ? <DashboardError message={errorMessage(error)} /> : null}
      <KpiGrid
        t={t}
        locale={locale}
        isAdmin={isAdmin}
        loading={statsLoading}
        data={overview.data}
      />
      <PeriodSummaryGrid
        t={t}
        locale={locale}
        isAdmin={isAdmin}
        loading={statsLoading}
        preset={preset}
        summary={overview.data?.summary}
      />
      <DashboardMainGrid
        t={t}
        locale={locale}
        isAdmin={isAdmin}
        statsLoading={statsLoading}
        preset={preset}
        dailyTable={dailyTable}
        overview={overview.data}
        trendLoading={overview.isLoading}
        activity={activity.data}
        activityLoading={activity.isLoading}
      />
    </DashboardContent>
  );
}

function DashboardError({ message }: { message: string }) {
  return (
    <Alert severity="error" sx={{ mb: 3 }}>
      {message}
    </Alert>
  );
}

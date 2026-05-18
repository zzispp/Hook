'use client';

import type { DashboardActivityFilters } from 'src/actions/dashboard';
import type { DashboardPreset, DashboardOverviewResponse } from 'src/types/dashboard';

import { useState } from 'react';

import Grid from '@mui/material/Grid';
import Alert from '@mui/material/Alert';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import {
  useDashboardActivity,
  useDashboardOverview,
  useDashboardFilterOptions,
} from 'src/actions/dashboard';

import { useAuthContext } from 'src/auth/hooks';

import { KpiGrid } from './dashboard-kpi';
import { TrendCard } from './dashboard-trend';
import { errorMessage } from './dashboard-format';
import { DashboardHeader } from './dashboard-header';
import { BreakdownCard } from './dashboard-breakdown';
import { ActivityGridCard } from './dashboard-activity';

export function OverviewAnalyticsView() {
  const { t, currentLang } = useTranslate('admin');
  const { user } = useAuthContext();
  const [preset, setPreset] = useState<DashboardPreset>('today');
  const overview = useDashboardOverview(preset);
  const isAdmin = user?.role === 'admin';
  const [activityFilters, setActivityFilters] = useState<DashboardActivityFilters>(
    isAdmin ? { scope: 'global' } : { scope: 'me' }
  );
  const activity = useDashboardActivity(activityFilters);
  const filterOptions = useDashboardFilterOptions(isAdmin);
  const locale = currentLang.numberFormat.code;
  const error = overview.error ?? activity.error ?? filterOptions.error;
  const statsLoading = overview.isLoading;

  return (
    <DashboardContent maxWidth="xl">
      <DashboardHeader
        t={t}
        isAdmin={isAdmin}
        preset={preset}
        loading={overview.isValidating || activity.isValidating}
        onPresetChange={setPreset}
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
        loading={statsLoading}
        data={overview.data}
      />

      <Grid container spacing={3}>
        <Grid size={{ xs: 12, lg: 8 }}>
          <TrendCard t={t} loading={overview.isLoading} data={overview.data} />
        </Grid>
        <Grid size={{ xs: 12, lg: 4 }}>
          <BreakdownCard
            t={t}
            locale={locale}
            title={t('dashboard.stats.breakdowns.models')}
            items={overview.data?.breakdowns.models}
            loading={statsLoading}
          />
        </Grid>
        <Grid size={{ xs: 12 }}>
          <ActivityGridCard
            t={t}
            isAdmin={isAdmin}
            filters={activityFilters}
            activity={activity.data}
            filterOptions={filterOptions.data}
            loading={activity.isLoading}
            onFiltersChange={setActivityFilters}
          />
        </Grid>
        <SharedBreakdowns
          t={t}
          locale={locale}
          loading={statsLoading}
          overview={overview.data}
          isAdmin={isAdmin}
        />
      </Grid>
    </DashboardContent>
  );
}

function SharedBreakdowns({
  t,
  locale,
  loading,
  overview,
  isAdmin,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  locale: string;
  loading: boolean;
  overview?: DashboardOverviewResponse;
  isAdmin: boolean;
}) {
  return (
    <>
      <Grid size={{ xs: 12, md: 6, lg: isAdmin ? 3 : 4 }}>
        <BreakdownCard
          t={t}
          locale={locale}
          title={t('dashboard.stats.breakdowns.apiFormats')}
          items={overview?.breakdowns.api_formats}
          loading={loading}
          variant="distribution"
        />
      </Grid>
      <Grid size={{ xs: 12, md: 6, lg: isAdmin ? 3 : 4 }}>
        <BreakdownCard
          t={t}
          locale={locale}
          title={t('dashboard.stats.breakdowns.tokens')}
          items={overview?.breakdowns.tokens}
          loading={loading}
        />
      </Grid>
      {isAdmin ? (
        <AdminBreakdowns
          t={t}
          locale={locale}
          loading={loading}
          overview={overview}
        />
      ) : null}
    </>
  );
}

function AdminBreakdowns({
  t,
  locale,
  loading,
  overview,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  locale: string;
  loading: boolean;
  overview?: DashboardOverviewResponse;
}) {
  return (
    <>
      <Grid size={{ xs: 12, md: 6, lg: 3 }}>
        <BreakdownCard
          t={t}
          locale={locale}
          title={t('dashboard.stats.breakdowns.providers')}
          items={overview?.breakdowns.providers}
          loading={loading}
          variant="distribution"
        />
      </Grid>
      <Grid size={{ xs: 12, md: 6, lg: 3 }}>
        <BreakdownCard
          t={t}
          locale={locale}
          title={t('dashboard.stats.breakdowns.users')}
          items={overview?.breakdowns.users}
          loading={loading}
        />
      </Grid>
    </>
  );
}

function DashboardError({ message }: { message: string }) {
  return (
    <Alert severity="error" sx={{ mb: 3 }}>
      {message}
    </Alert>
  );
}

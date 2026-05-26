'use client';

import type { DashboardActivityFilters } from 'src/actions/dashboard';
import type {
  DashboardPreset,
  DashboardOverviewResponse,
  DashboardActivityResponse,
} from 'src/types/dashboard';

import { useState, useEffect } from 'react';

import Box from '@mui/material/Box';
import Grid from '@mui/material/Grid';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';

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
  const isAdmin = user?.role === 'admin';
  const [activityFilters, setActivityFilters] = useState<DashboardActivityFilters>(
    isAdmin ? { scope: 'global' } : { scope: 'me' }
  );
  const overview = useDashboardOverview(preset, activityFilters);
  const activity = useDashboardActivity(activityFilters);
  const filterOptions = useDashboardFilterOptions(isAdmin);
  const locale = currentLang.numberFormat.code;
  const error = overview.error ?? activity.error ?? filterOptions.error;
  const statsLoading = overview.isLoading;

  useEffect(() => {
    if (user === undefined) return;
    if (!isAdmin && activityFilters.scope !== 'me') setActivityFilters({ scope: 'me' });
    if (isAdmin && activityFilters.scope === 'me') setActivityFilters({ scope: 'global' });
  }, [activityFilters.scope, isAdmin, user]);

  return (
    <DashboardContent maxWidth="xl">
      <DashboardHeader
        t={t}
        isAdmin={isAdmin}
        preset={preset}
        loading={overview.isValidating || activity.isValidating}
        filters={activityFilters}
        filterOptions={filterOptions.data}
        onPresetChange={setPreset}
        onFiltersChange={setActivityFilters}
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

      <DashboardMainGrid
        t={t}
        locale={locale}
        isAdmin={isAdmin}
        statsLoading={statsLoading}
        overview={overview.data}
        trendLoading={overview.isLoading}
        activity={activity.data}
        activityLoading={activity.isLoading}
      />
    </DashboardContent>
  );
}

type DashboardMainGridProps = {
  t: ReturnType<typeof useTranslate>['t'];
  locale: string;
  isAdmin: boolean;
  statsLoading: boolean;
  trendLoading: boolean;
  activityLoading: boolean;
  overview?: DashboardOverviewResponse;
  activity?: DashboardActivityResponse;
};

function DashboardMainGrid({
  t,
  locale,
  isAdmin,
  overview,
  activity,
  statsLoading,
  trendLoading,
  activityLoading,
}: DashboardMainGridProps) {
  return (
    <Stack spacing={3}>
      <Box sx={DASHBOARD_TOP_GRID_SX}>
        <Box sx={DASHBOARD_TREND_ACTIVITY_SX}>
          <Box sx={DASHBOARD_TREND_AREA_SX}>
            <TrendCard t={t} isAdmin={isAdmin} loading={trendLoading} data={overview} />
          </Box>
          <Box sx={DASHBOARD_ACTIVITY_AREA_SX}>
            <ActivityGridCard
              t={t}
              isAdmin={isAdmin}
              activity={activity}
              loading={activityLoading}
            />
          </Box>
        </Box>
        <Box sx={DASHBOARD_MODELS_AREA_SX}>
          <BreakdownCard
            t={t}
            locale={locale}
            isAdmin={isAdmin}
            title={t('dashboard.stats.breakdowns.models')}
            items={overview?.breakdowns.models}
            loading={statsLoading}
          />
        </Box>
      </Box>
      <Grid container spacing={3}>
        <SharedBreakdowns
          t={t}
          locale={locale}
          loading={statsLoading}
          overview={overview}
          isAdmin={isAdmin}
        />
      </Grid>
    </Stack>
  );
}

const DASHBOARD_TOP_GRID_SX = {
  gap: 3,
  display: 'grid',
  alignItems: 'start',
  gridTemplateColumns: { xs: '1fr', lg: 'minmax(0, 8fr) minmax(0, 4fr)' },
  gridTemplateAreas: { xs: '"trend" "models" "activity"', lg: '"main models"' },
} as const;

const DASHBOARD_TREND_ACTIVITY_SX = {
  gap: 3,
  gridArea: { lg: 'main' },
  display: { xs: 'contents', lg: 'flex' },
  flexDirection: 'column',
} as const;

const DASHBOARD_TREND_AREA_SX = {
  gridArea: { xs: 'trend', lg: 'auto' },
} as const;

const DASHBOARD_ACTIVITY_AREA_SX = {
  gridArea: { xs: 'activity', lg: 'auto' },
} as const;

const DASHBOARD_MODELS_AREA_SX = {
  gridArea: 'models',
} as const;

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
      <Grid size={{ xs: 12, md: 6, lg: isAdmin ? 4 : 6 }}>
        <BreakdownCard
          t={t}
          locale={locale}
          isAdmin={isAdmin}
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
      <Grid size={{ xs: 12, md: 6, lg: 4 }}>
        <BreakdownCard
          t={t}
          locale={locale}
          isAdmin
          title={t('dashboard.stats.breakdowns.providers')}
          items={overview?.breakdowns.providers}
          loading={loading}
          variant="distribution"
        />
      </Grid>
      <Grid size={{ xs: 12, md: 6, lg: 4 }}>
        <BreakdownCard
          t={t}
          locale={locale}
          isAdmin
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

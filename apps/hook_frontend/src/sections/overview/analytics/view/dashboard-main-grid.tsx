import type { TFunction } from 'i18next';
import type { UseTableReturn } from 'src/components/table';
import type {
  DashboardPreset,
  DashboardOverviewResponse,
  DashboardActivityResponse,
} from 'src/types/dashboard';

import Box from '@mui/material/Box';
import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';

import { TrendCard } from './dashboard-trend';
import { BreakdownCard } from './dashboard-breakdown';
import { ActivityGridCard } from './dashboard-activity';
import { DailyStatsTable } from './dashboard-daily-table';
import { DailyInsights } from './dashboard-daily-insights';

type DashboardMainGridProps = {
  t: TFunction<'admin'>;
  locale: string;
  isAdmin: boolean;
  preset: DashboardPreset;
  dailyTable: UseTableReturn;
  statsLoading: boolean;
  trendLoading: boolean;
  activityLoading: boolean;
  overview?: DashboardOverviewResponse;
  activity?: DashboardActivityResponse;
};

export function DashboardMainGrid({
  t,
  locale,
  isAdmin,
  preset,
  dailyTable,
  overview,
  activity,
  statsLoading,
  trendLoading,
  activityLoading,
}: DashboardMainGridProps) {
  return (
    <Stack spacing={3}>
      <DailyInsights
        t={t}
        locale={locale}
        isAdmin={isAdmin}
        loading={statsLoading}
        preset={preset}
        data={overview?.daily}
      />
      <DailyStatsTable
        t={t}
        locale={locale}
        isAdmin={isAdmin}
        loading={statsLoading}
        table={dailyTable}
        data={overview?.daily}
      />
      <DashboardTopGrid
        t={t}
        locale={locale}
        isAdmin={isAdmin}
        overview={overview}
        activity={activity}
        statsLoading={statsLoading}
        trendLoading={trendLoading}
        activityLoading={activityLoading}
      />
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

function DashboardTopGrid({
  t,
  locale,
  isAdmin,
  overview,
  activity,
  statsLoading,
  trendLoading,
  activityLoading,
}: Omit<DashboardMainGridProps, 'preset' | 'dailyTable'>) {
  return (
    <Box sx={DASHBOARD_TOP_GRID_SX}>
      <Box sx={DASHBOARD_TREND_ACTIVITY_SX}>
        <Box sx={DASHBOARD_TREND_AREA_SX}>
          <TrendCard t={t} isAdmin={isAdmin} loading={trendLoading} data={overview} />
        </Box>
        <Box sx={DASHBOARD_ACTIVITY_AREA_SX}>
          <ActivityGridCard t={t} isAdmin={isAdmin} activity={activity} loading={activityLoading} />
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
  t: TFunction<'admin'>;
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
  t: TFunction<'admin'>;
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

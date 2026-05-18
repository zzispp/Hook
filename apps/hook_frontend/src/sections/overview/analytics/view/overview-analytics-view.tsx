'use client';

import type { DashboardActivityFilters } from 'src/actions/dashboard';
import type { DashboardPreset, DashboardOverviewResponse } from 'src/types/dashboard';

import { useMemo, useState } from 'react';

import Grid from '@mui/material/Grid';
import Alert from '@mui/material/Alert';

import { currencyDisplayFromResponse } from 'src/utils/currency-format';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useCurrencyDisplay } from 'src/actions/system-settings';
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
  const currency = useCurrencyDisplay();
  const currencyDisplay = useMemo(
    () => currencyDisplayFromResponse(currency.data, t('requestRecords.exchangeRateUnavailable')),
    [currency.data, t]
  );
  const locale = currentLang.numberFormat.code;
  const error = overview.error ?? activity.error ?? filterOptions.error ?? currency.error;
  const statsLoading = overview.isLoading || currency.isLoading;

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
          void currency.refresh();
        }}
      />

      {error ? <DashboardError message={errorMessage(error)} /> : null}
      <KpiGrid
        t={t}
        locale={locale}
        loading={statsLoading}
        data={overview.data}
        currencyDisplay={currencyDisplay}
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
            currencyDisplay={currencyDisplay}
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
          currencyDisplay={currencyDisplay}
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
  currencyDisplay,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  locale: string;
  loading: boolean;
  overview?: DashboardOverviewResponse;
  isAdmin: boolean;
  currencyDisplay?: ReturnType<typeof currencyDisplayFromResponse>;
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
          currencyDisplay={currencyDisplay}
        />
      </Grid>
      <Grid size={{ xs: 12, md: 6, lg: isAdmin ? 3 : 4 }}>
        <BreakdownCard
          t={t}
          locale={locale}
          title={t('dashboard.stats.breakdowns.tokens')}
          items={overview?.breakdowns.tokens}
          loading={loading}
          currencyDisplay={currencyDisplay}
        />
      </Grid>
      {isAdmin ? (
        <AdminBreakdowns
          t={t}
          locale={locale}
          loading={loading}
          overview={overview}
          currencyDisplay={currencyDisplay}
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
  currencyDisplay,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  locale: string;
  loading: boolean;
  overview?: DashboardOverviewResponse;
  currencyDisplay?: ReturnType<typeof currencyDisplayFromResponse>;
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
          currencyDisplay={currencyDisplay}
        />
      </Grid>
      <Grid size={{ xs: 12, md: 6, lg: 3 }}>
        <BreakdownCard
          t={t}
          locale={locale}
          title={t('dashboard.stats.breakdowns.users')}
          items={overview?.breakdowns.users}
          loading={loading}
          currencyDisplay={currencyDisplay}
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

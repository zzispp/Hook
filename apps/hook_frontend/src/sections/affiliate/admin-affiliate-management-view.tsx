'use client';

import type { TFunction } from 'i18next';

import { useState } from 'react';

import Tab from '@mui/material/Tab';
import Card from '@mui/material/Card';
import Grid from '@mui/material/Grid';
import Tabs from '@mui/material/Tabs';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { RefreshButton, AdminBreadcrumbs } from 'src/sections/admin/shared';

import { AffiliateRelationDialog } from './admin-affiliate-relation-dialog';
import { RelationChangesTable } from './admin-affiliate-relation-change-table';
import { formatMoney, formatCount, formatPercent } from './admin-affiliate-format';
import { type AffiliateTab, useAdminAffiliateManagementState } from './admin-affiliate-state';
import {
  ReportFiltersToolbar,
  RelationFiltersToolbar,
  CommissionFiltersToolbar,
} from './admin-affiliate-filters';
import {
  RelationsTable,
  CommissionsTable,
  DailyReportTable,
  ReferrerReportTable,
} from './admin-affiliate-tables';

export function AdminAffiliateManagementView() {
  const { t, currentLang } = useTranslate('admin');
  const [tab, setTab] = useState<AffiliateTab>('relations');
  const state = useAdminAffiliateManagementState(t);
  const locale = currentLang.numberFormat.code;
  const loading = tabLoading(tab, state);

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.affiliateManagement}
        action={<RefreshButton loading={loading} onClick={() => state.refresh(tab)} />}
      />
      {state.errorMessage ? <ErrorAlert message={state.errorMessage} /> : null}
      <OverviewCards t={t} locale={locale} data={state.overview.data} />
      <Tabs value={tab} onChange={(_event, next: AffiliateTab) => setTab(next)} sx={{ mb: 3 }}>
        <Tab value="relations" label={t('adminAffiliates.tabs.relations')} />
        <Tab value="relationChanges" label={t('adminAffiliates.tabs.relationChanges')} />
        <Tab value="commissions" label={t('adminAffiliates.tabs.commissions')} />
        <Tab value="reports" label={t('adminAffiliates.tabs.reports')} />
      </Tabs>
      {tab === 'relations' ? <RelationsPanel t={t} locale={locale} state={state} /> : null}
      {tab === 'relationChanges' ? (
        <RelationChangesPanel t={t} locale={locale} state={state} />
      ) : null}
      {tab === 'commissions' ? <CommissionsPanel t={t} locale={locale} state={state} /> : null}
      {tab === 'reports' ? <ReportsPanel t={t} locale={locale} state={state} /> : null}
      <AffiliateRelationDialog
        t={t}
        state={state.dialog}
        submitting={state.submitting}
        onChange={state.changeDialog}
        onClose={state.closeDialog}
        onSubmit={state.submitDialog}
      />
    </DashboardContent>
  );
}

function OverviewCards({
  t,
  locale,
  data,
}: {
  t: TFunction<'admin'>;
  locale: string;
  data?: ReturnType<typeof useAdminAffiliateManagementState>['overview']['data'];
}) {
  const items = [
    {
      label: t('adminAffiliates.overview.totalCommission'),
      value: formatMoney(data?.total_commission_amount ?? 0, locale),
    },
    {
      label: t('adminAffiliates.overview.totalReferredUsers'),
      value: formatCount(data?.total_referred_users ?? 0, locale),
    },
    {
      label: t('adminAffiliates.overview.todayCommission'),
      value: formatMoney(data?.today_commission_amount ?? 0, locale),
    },
    {
      label: t('adminAffiliates.overview.monthCommission'),
      value: formatMoney(data?.month_commission_amount ?? 0, locale),
    },
    {
      label: t('adminAffiliates.overview.commissionPercent'),
      value: formatPercent(data?.affiliate_commission_percent ?? 0, locale),
    },
    {
      label: t('adminAffiliates.overview.activeReferrers'),
      value: formatCount(data?.active_referrer_count ?? 0, locale),
    },
  ];

  return (
    <Grid container spacing={2} sx={{ mb: 3 }}>
      {items.map((item) => (
        <Grid key={item.label} size={{ xs: 12, sm: 6, md: 4, lg: 2 }}>
          <Card sx={{ p: 2 }}>
            <Typography variant="caption" color="text.secondary">
              {item.label}
            </Typography>
            <Typography variant="h6" sx={{ mt: 0.5 }}>
              {item.value}
            </Typography>
          </Card>
        </Grid>
      ))}
    </Grid>
  );
}

function RelationsPanel({ t, locale, state }: PanelProps) {
  return (
    <Card>
      <RelationFiltersToolbar
        t={t}
        filters={state.relationFilters}
        onChange={state.changeRelationFilters}
      />
      <RelationsTable
        t={t}
        locale={locale}
        rows={state.relations.data?.items ?? []}
        total={state.relations.data?.total ?? 0}
        loading={state.relations.isLoading}
        page={state.relationTable.page}
        rowsPerPage={state.relationTable.rowsPerPage}
        onRebind={state.openRebind}
        onClear={state.openClear}
        onPageChange={state.relationTable.onChangePage}
        onRowsPerPageChange={state.relationTable.onChangeRowsPerPage}
      />
    </Card>
  );
}

function RelationChangesPanel({ t, locale, state }: PanelProps) {
  return (
    <Card>
      <RelationChangesTable
        t={t}
        locale={locale}
        rows={state.relationChanges.data?.items ?? []}
        total={state.relationChanges.data?.total ?? 0}
        loading={state.relationChanges.isLoading}
        page={state.relationChangeTable.page}
        rowsPerPage={state.relationChangeTable.rowsPerPage}
        onPageChange={state.relationChangeTable.onChangePage}
        onRowsPerPageChange={state.relationChangeTable.onChangeRowsPerPage}
      />
    </Card>
  );
}

function CommissionsPanel({ t, locale, state }: PanelProps) {
  return (
    <Card>
      <CommissionFiltersToolbar
        t={t}
        filters={state.commissionFilters}
        onChange={state.changeCommissionFilters}
      />
      <CommissionsTable
        t={t}
        locale={locale}
        rows={state.commissions.data?.items ?? []}
        total={state.commissions.data?.total ?? 0}
        loading={state.commissions.isLoading}
        page={state.commissionTable.page}
        rowsPerPage={state.commissionTable.rowsPerPage}
        onPageChange={state.commissionTable.onChangePage}
        onRowsPerPageChange={state.commissionTable.onChangeRowsPerPage}
      />
    </Card>
  );
}

function ReportsPanel({ t, locale, state }: PanelProps) {
  return (
    <Stack spacing={3}>
      <Card>
        <ReportFiltersToolbar
          t={t}
          filters={state.reportFilters}
          onChange={state.changeReportFilters}
          onExportDetails={state.exportDetails}
          onExportDaily={state.exportDaily}
          onExportReferrers={state.exportReferrers}
        />
      </Card>
      <Card>
        <Typography variant="subtitle1" sx={{ p: 2.5, pb: 0 }}>
          {t('adminAffiliates.actions.exportDaily')}
        </Typography>
        <DailyReportTable
          t={t}
          locale={locale}
          rows={state.reports.data?.daily_items ?? []}
          loading={state.reports.isLoading}
          rowsPerPage={5}
        />
      </Card>
      <Card>
        <Typography variant="subtitle1" sx={{ p: 2.5, pb: 0 }}>
          {t('adminAffiliates.actions.exportReferrers')}
        </Typography>
        <ReferrerReportTable
          t={t}
          locale={locale}
          rows={state.reports.data?.referrer_items ?? []}
          total={state.reports.data?.referrer_total ?? 0}
          loading={state.reports.isLoading}
          page={state.reportTable.page}
          rowsPerPage={state.reportTable.rowsPerPage}
          onPageChange={state.reportTable.onChangePage}
          onRowsPerPageChange={state.reportTable.onChangeRowsPerPage}
        />
      </Card>
    </Stack>
  );
}

function tabLoading(tab: AffiliateTab, state: ReturnType<typeof useAdminAffiliateManagementState>) {
  if (tab === 'relations') return state.relations.isLoading || state.overview.isLoading;
  if (tab === 'relationChanges') return state.relationChanges.isLoading || state.overview.isLoading;
  if (tab === 'commissions') return state.commissions.isLoading || state.overview.isLoading;
  return state.reports.isLoading || state.overview.isLoading;
}

function ErrorAlert({ message }: { message: string }) {
  return (
    <Alert severity="error" sx={{ mb: 3 }}>
      {message}
    </Alert>
  );
}

type PanelProps = {
  t: TFunction<'admin'>;
  locale: string;
  state: ReturnType<typeof useAdminAffiliateManagementState>;
};

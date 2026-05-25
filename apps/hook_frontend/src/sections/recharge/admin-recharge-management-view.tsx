'use client';

import { useState } from 'react';

import Tab from '@mui/material/Tab';
import Card from '@mui/material/Card';
import Tabs from '@mui/material/Tabs';
import Alert from '@mui/material/Alert';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useSystemSettings } from 'src/actions/system-settings';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { EmptyContent } from 'src/components/empty-content';

import { RefreshButton, AdminBreadcrumbs } from 'src/sections/admin/shared';

import { RechargeOrderTable } from './recharge-order-table';
import { RechargePackageTable } from './recharge-package-table';
import { RechargePackageDialog } from './recharge-package-dialog';
import { RechargeOrderToolbar, RechargePackageToolbar } from './recharge-filters';
import { type RechargeTab, useRechargeManagementState } from './admin-recharge-state';

export function AdminRechargeManagementView() {
  const { t, currentLang } = useTranslate('admin');
  const [tab, setTab] = useState<RechargeTab>('orders');
  const state = useRechargeManagementState(t);
  const settings = useSystemSettings();
  const locale = currentLang.numberFormat.code;
  const loading = tab === 'orders' ? state.orders.isLoading : state.packages.isLoading;
  const ratio = settings.data?.recharge_arrival_ratio ?? 1;

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.rechargeManagement}
        action={<RefreshButton loading={loading} onClick={() => state.refresh(tab)} />}
      />
      {state.errorMessage ? <ErrorAlert message={state.errorMessage} /> : null}
      {settings.error ? <ErrorAlert message={settings.error.message} /> : null}
      <Tabs value={tab} onChange={(_event, next: RechargeTab) => setTab(next)} sx={{ mb: 3 }}>
        <Tab value="orders" label={t('adminRecharges.tabs.orders')} />
        <Tab value="packages" label={t('adminRecharges.tabs.packages')} />
        <Tab value="callbacks" label={t('adminRecharges.tabs.callbacks')} />
      </Tabs>
      {tab === 'orders' ? <OrdersPanel t={t} locale={locale} state={state} /> : null}
      {tab === 'packages' ? (
        <PackagesPanel t={t} locale={locale} ratio={ratio} state={state} />
      ) : null}
      {tab === 'callbacks' ? <CallbacksPanel t={t} /> : null}
      <RechargePackageDialog
        t={t}
        open={state.dialogOpen}
        item={state.editingPackage}
        ratio={ratio}
        submitting={state.submitting}
        onClose={state.closeDialog}
        onSubmit={state.submitPackage}
      />
    </DashboardContent>
  );
}

function OrdersPanel({
  t,
  locale,
  state,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  locale: string;
  state: ReturnType<typeof useRechargeManagementState>;
}) {
  return (
    <Card>
      <RechargeOrderToolbar
        t={t}
        filters={state.orderFilters}
        onChange={state.changeOrderFilters}
      />
      <RechargeOrderTable
        t={t}
        locale={locale}
        rows={state.orders.data?.items ?? []}
        total={state.orders.data?.total ?? 0}
        loading={state.orders.isLoading}
        page={state.orderTable.page}
        rowsPerPage={state.orderTable.rowsPerPage}
        onPageChange={state.orderTable.onChangePage}
        onRowsPerPageChange={state.orderTable.onChangeRowsPerPage}
      />
    </Card>
  );
}

function PackagesPanel({
  t,
  locale,
  ratio,
  state,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  locale: string;
  ratio: number;
  state: ReturnType<typeof useRechargeManagementState>;
}) {
  return (
    <Card>
      <RechargePackageToolbar
        t={t}
        filters={state.packageFilters}
        busy={state.submitting}
        onChange={state.changePackageFilters}
        onCreate={state.openCreatePackage}
      />
      <RechargePackageTable
        t={t}
        locale={locale}
        ratio={ratio}
        rows={state.packages.data?.items ?? []}
        total={state.packages.data?.total ?? 0}
        loading={state.packages.isLoading}
        busy={state.submitting}
        page={state.packageTable.page}
        rowsPerPage={state.packageTable.rowsPerPage}
        onEdit={state.openEditPackage}
        onToggleStatus={state.togglePackageStatus}
        onPageChange={state.packageTable.onChangePage}
        onRowsPerPageChange={state.packageTable.onChangeRowsPerPage}
      />
    </Card>
  );
}

function CallbacksPanel({ t }: { t: ReturnType<typeof useTranslate>['t'] }) {
  return (
    <Card sx={{ py: 6 }}>
      <EmptyContent
        filled
        title={t('adminRecharges.empty.callbacks')}
        description={t('adminRecharges.empty.callbacksDescription')}
      />
    </Card>
  );
}

function ErrorAlert({ message }: { message: string }) {
  return (
    <Alert severity="error" sx={{ mb: 3 }}>
      {message}
    </Alert>
  );
}

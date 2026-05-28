'use client';

import type { PaymentCallbackRecord } from 'src/types/recharge';

import { useMemo, useState, useCallback } from 'react';

import Tab from '@mui/material/Tab';
import Card from '@mui/material/Card';
import Tabs from '@mui/material/Tabs';
import Alert from '@mui/material/Alert';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useSystemSettings } from 'src/actions/system-settings';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { RefreshButton, AdminBreadcrumbs } from 'src/sections/admin/shared';

import { RechargeOrderTable } from './recharge-order-table';
import { RechargePackageTable } from './recharge-package-table';
import { RechargeCallbackTable } from './recharge-callback-table';
import { RechargePackageDialog } from './recharge-package-dialog';
import { RechargeCallbackDetailDrawer } from './recharge-callback-detail-drawer';
import { type RechargeTab, useRechargeManagementState } from './admin-recharge-state';
import {
  RechargeOrderToolbar,
  RechargePackageToolbar,
  PaymentCallbackToolbar,
} from './recharge-filters';

export function AdminRechargeManagementView() {
  const { t, currentLang } = useTranslate('admin');
  const [tab, setTab] = useState<RechargeTab>('orders');
  const state = useRechargeManagementState(t);
  const settings = useSystemSettings();
  const locale = currentLang.numberFormat.code;
  const loading = tabLoading(tab, state);
  const ratio = settings.data?.recharge_arrival_ratio ?? 1;
  const callbackSelection = useCallbackSelection(state.callbacks.data?.items ?? []);
  const handleTabChange = (_event: unknown, next: RechargeTab) => {
    if (next !== 'callbacks') callbackSelection.close();
    setTab(next);
  };

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.rechargeManagement}
        action={<RefreshButton loading={loading} onClick={() => state.refresh(tab)} />}
      />
      {state.errorMessage ? <ErrorAlert message={state.errorMessage} /> : null}
      {settings.error ? <ErrorAlert message={settings.error.message} /> : null}
      <Tabs value={tab} onChange={handleTabChange} sx={{ mb: 3 }}>
        <Tab value="orders" label={t('adminRecharges.tabs.orders')} />
        <Tab value="packages" label={t('adminRecharges.tabs.packages')} />
        <Tab value="callbacks" label={t('adminRecharges.tabs.callbacks')} />
      </Tabs>
      {tab === 'orders' ? <OrdersPanel t={t} locale={locale} state={state} /> : null}
      {tab === 'packages' ? (
        <PackagesPanel t={t} locale={locale} ratio={ratio} state={state} />
      ) : null}
      {tab === 'callbacks' ? (
        <CallbacksPanel t={t} locale={locale} state={state} onOpen={callbackSelection.open} />
      ) : null}
      <RechargePackageDialog
        t={t}
        open={state.dialogOpen}
        item={state.editingPackage}
        ratio={ratio}
        submitting={state.submitting}
        onClose={state.closeDialog}
        onSubmit={state.submitPackage}
      />
      <RechargeCallbackDetailDrawer
        t={t}
        open={tab === 'callbacks' && Boolean(callbackSelection.record)}
        record={callbackSelection.record}
        locale={locale}
        onClose={callbackSelection.close}
      />
    </DashboardContent>
  );
}

function useCallbackSelection(items: PaymentCallbackRecord[]) {
  const [selectedCallback, setSelectedCallback] = useState<PaymentCallbackRecord | null>(null);
  const record = useMemo(
    () => latestSelectedCallback(selectedCallback, items),
    [items, selectedCallback]
  );
  const open = useCallback((callback: PaymentCallbackRecord) => {
    setSelectedCallback(callback);
  }, []);
  const close = useCallback(() => {
    setSelectedCallback(null);
  }, []);

  return { record, open, close };
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

function CallbacksPanel({
  t,
  locale,
  state,
  onOpen,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  locale: string;
  state: ReturnType<typeof useRechargeManagementState>;
  onOpen: (record: PaymentCallbackRecord) => void;
}) {
  return (
    <Card>
      <PaymentCallbackToolbar
        t={t}
        filters={state.callbackFilters}
        onChange={state.changeCallbackFilters}
      />
      <RechargeCallbackTable
        t={t}
        locale={locale}
        rows={state.callbacks.data?.items ?? []}
        total={state.callbacks.data?.total ?? 0}
        loading={state.callbacks.isLoading}
        page={state.callbackTable.page}
        rowsPerPage={state.callbackTable.rowsPerPage}
        onOpen={onOpen}
        onPageChange={state.callbackTable.onChangePage}
        onRowsPerPageChange={state.callbackTable.onChangeRowsPerPage}
      />
    </Card>
  );
}

function tabLoading(tab: RechargeTab, state: ReturnType<typeof useRechargeManagementState>) {
  if (tab === 'orders') return state.orders.isLoading;
  if (tab === 'packages') return state.packages.isLoading;
  return state.callbacks.isLoading;
}

function latestSelectedCallback(
  selectedCallback: PaymentCallbackRecord | null,
  items: PaymentCallbackRecord[]
) {
  if (!selectedCallback) return null;
  return items.find((record) => record.id === selectedCallback.id) ?? selectedCallback;
}

function ErrorAlert({ message }: { message: string }) {
  return (
    <Alert severity="error" sx={{ mb: 3 }}>
      {message}
    </Alert>
  );
}

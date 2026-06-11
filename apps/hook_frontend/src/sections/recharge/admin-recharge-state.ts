'use client';

import type { useTranslate } from 'src/locales/use-locales';
import type { RechargePackage, RechargePackageInput } from 'src/types/recharge';

import { useMemo, useState, useCallback } from 'react';

import {
  useRechargeOrders,
  usePaymentCallbacks,
  useRechargePackages,
  createRechargePackage,
  updateRechargePackage,
  useRechargeOrderSummary,
} from 'src/actions/recharge';

import { toast } from 'src/components/snackbar';
import { useTable } from 'src/components/table';

import {
  toRechargeOrderFilters,
  toPaymentCallbackFilters,
  toRechargePackageFilters,
  DEFAULT_RECHARGE_ORDER_FILTERS,
  DEFAULT_PAYMENT_CALLBACK_FILTERS,
  DEFAULT_RECHARGE_PACKAGE_FILTERS,
} from './recharge-filters';

export type RechargeTab = 'orders' | 'packages' | 'callbacks';

export function useRechargeManagementState(t: ReturnType<typeof useTranslate>['t']) {
  const orderTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'created_at' });
  const orderSummaryTable = useTable({
    defaultRowsPerPage: 10,
    defaultOrderBy: 'total_payable_amount',
  });
  const packageTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'sort_order' });
  const callbackTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'received_at' });
  const [orderFilters, setOrderFilters] = useState(DEFAULT_RECHARGE_ORDER_FILTERS);
  const [packageFilters, setPackageFilters] = useState(DEFAULT_RECHARGE_PACKAGE_FILTERS);
  const [callbackFilters, setCallbackFilters] = useState(DEFAULT_PAYMENT_CALLBACK_FILTERS);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingPackage, setEditingPackage] = useState<RechargePackage | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const orderQueryFilters = useMemo(() => toRechargeOrderFilters(orderFilters), [orderFilters]);
  const packageQueryFilters = useMemo(
    () => toRechargePackageFilters(packageFilters),
    [packageFilters]
  );
  const callbackQueryFilters = useMemo(
    () => toPaymentCallbackFilters(callbackFilters),
    [callbackFilters]
  );
  const orders = useRechargeOrders(orderTable.page, orderTable.rowsPerPage, orderQueryFilters);
  const orderSummary = useRechargeOrderSummary(
    orderSummaryTable.page,
    orderSummaryTable.rowsPerPage,
    orderQueryFilters,
    orderFilters.summaryEnabled
  );
  const packages = useRechargePackages(
    packageTable.page,
    packageTable.rowsPerPage,
    packageQueryFilters
  );
  const callbacks = usePaymentCallbacks(
    callbackTable.page,
    callbackTable.rowsPerPage,
    callbackQueryFilters
  );

  const refresh = useCallback(
    (tab: RechargeTab) => {
      if (tab === 'orders') {
        void orders.refresh();
        if (orderFilters.summaryEnabled) void orderSummary.refresh();
      }
      if (tab === 'packages') void packages.refresh();
      if (tab === 'callbacks') void callbacks.refresh();
    },
    [callbacks, orderFilters.summaryEnabled, orderSummary, orders, packages]
  );

  return {
    orders,
    orderSummary,
    packages,
    callbacks,
    orderTable,
    orderSummaryTable,
    packageTable,
    callbackTable,
    orderFilters,
    packageFilters,
    callbackFilters,
    submitting,
    dialogOpen,
    editingPackage,
    errorMessage:
      orders.error?.message ??
      orderSummary.error?.message ??
      packages.error?.message ??
      callbacks.error?.message,
    refresh,
    changeOrderFilters: orderFilterHandler(orderTable, orderSummaryTable, setOrderFilters),
    changePackageFilters: filterHandler(packageTable, setPackageFilters),
    changeCallbackFilters: filterHandler(callbackTable, setCallbackFilters),
    openCreatePackage: () => {
      setEditingPackage(null);
      setDialogOpen(true);
    },
    openEditPackage: (item: RechargePackage) => {
      setEditingPackage(item);
      setDialogOpen(true);
    },
    closeDialog: () => setDialogOpen(false),
    submitPackage: submitPackageHandler(
      t,
      editingPackage,
      setSubmitting,
      setDialogOpen,
      packages.refresh
    ),
    togglePackageStatus: togglePackageStatusHandler(t, setSubmitting, packages.refresh),
  };
}

function orderFilterHandler(
  orderTable: ReturnType<typeof useTable>,
  summaryTable: ReturnType<typeof useTable>,
  setFilters: React.Dispatch<React.SetStateAction<typeof DEFAULT_RECHARGE_ORDER_FILTERS>>
) {
  return (next: typeof DEFAULT_RECHARGE_ORDER_FILTERS) => {
    orderTable.onResetPage();
    summaryTable.onResetPage();
    setFilters(next);
  };
}

function filterHandler<T>(
  table: ReturnType<typeof useTable>,
  setFilters: React.Dispatch<React.SetStateAction<T>>
) {
  return (next: T) => {
    table.onResetPage();
    setFilters(next);
  };
}

function submitPackageHandler(
  t: ReturnType<typeof useTranslate>['t'],
  editingPackage: RechargePackage | null,
  setSubmitting: (value: boolean) => void,
  setOpen: (value: boolean) => void,
  refresh: VoidFunction
) {
  return async (input: RechargePackageInput) => {
    setSubmitting(true);
    try {
      if (editingPackage) {
        await updateRechargePackage(editingPackage.id, input);
      } else {
        await createRechargePackage(input);
      }
      toast.success(t('adminRecharges.messages.packageSaved'));
      refresh();
      setOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  };
}

function togglePackageStatusHandler(
  t: ReturnType<typeof useTranslate>['t'],
  setSubmitting: (value: boolean) => void,
  refresh: VoidFunction
) {
  return async (item: RechargePackage) => {
    const status = item.status === 'active' ? 'disabled' : 'active';
    setSubmitting(true);
    try {
      await updateRechargePackage(item.id, {
        name: item.name,
        description: item.description ?? undefined,
        recharge_amount: item.recharge_amount,
        gift_amount: item.gift_amount,
        sort_order: item.sort_order,
        status,
      });
      toast.success(t('adminRecharges.messages.packageStatusUpdated'));
      refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  };
}

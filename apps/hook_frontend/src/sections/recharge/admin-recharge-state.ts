'use client';

import type { useTranslate } from 'src/locales/use-locales';
import type { RechargePackage, RechargePackageInput } from 'src/types/recharge';

import { useMemo, useState, useCallback } from 'react';

import {
  useRechargeOrders,
  useRechargePackages,
  createRechargePackage,
  updateRechargePackage,
} from 'src/actions/recharge';

import { toast } from 'src/components/snackbar';
import { useTable } from 'src/components/table';

import {
  toRechargeOrderFilters,
  toRechargePackageFilters,
  DEFAULT_RECHARGE_ORDER_FILTERS,
  DEFAULT_RECHARGE_PACKAGE_FILTERS,
} from './recharge-filters';

export type RechargeTab = 'orders' | 'packages' | 'callbacks';

export function useRechargeManagementState(t: ReturnType<typeof useTranslate>['t']) {
  const orderTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'created_at' });
  const packageTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'sort_order' });
  const [orderFilters, setOrderFilters] = useState(DEFAULT_RECHARGE_ORDER_FILTERS);
  const [packageFilters, setPackageFilters] = useState(DEFAULT_RECHARGE_PACKAGE_FILTERS);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingPackage, setEditingPackage] = useState<RechargePackage | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const orderQueryFilters = useMemo(() => toRechargeOrderFilters(orderFilters), [orderFilters]);
  const packageQueryFilters = useMemo(
    () => toRechargePackageFilters(packageFilters),
    [packageFilters]
  );
  const orders = useRechargeOrders(orderTable.page, orderTable.rowsPerPage, orderQueryFilters);
  const packages = useRechargePackages(
    packageTable.page,
    packageTable.rowsPerPage,
    packageQueryFilters
  );

  const refresh = useCallback(
    (tab: RechargeTab) => {
      if (tab === 'orders') void orders.refresh();
      if (tab === 'packages') void packages.refresh();
    },
    [orders, packages]
  );

  return {
    orders,
    packages,
    orderTable,
    packageTable,
    orderFilters,
    packageFilters,
    submitting,
    dialogOpen,
    editingPackage,
    errorMessage: orders.error?.message ?? packages.error?.message,
    refresh,
    changeOrderFilters: filterHandler(orderTable, setOrderFilters),
    changePackageFilters: filterHandler(packageTable, setPackageFilters),
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

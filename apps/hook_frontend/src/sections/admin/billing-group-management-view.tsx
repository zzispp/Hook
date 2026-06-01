'use client';

import type { BillingGroup } from 'src/types/group';

import { useState, useCallback } from 'react';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { useGlobalModels } from 'src/actions/models';
import { useBillingGroups } from 'src/actions/groups';
import { useTranslate } from 'src/locales/use-locales';
import { useUserGroups } from 'src/actions/user-groups';
import { DashboardContent } from 'src/layouts/dashboard';
import { useProviders, useProviderKeysByProvider } from 'src/actions/providers';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { useTable } from 'src/components/table';
import { ConfirmDialog } from 'src/components/custom-dialog';

import { BillingGroupTable } from './billing-group-table';
import { BillingGroupDialog } from './billing-group-dialog';
import { AddButton, RefreshButton, AdminBreadcrumbs } from './shared';
import { BillingGroupDetailDialog } from './billing-group-detail-dialog';
import { enabledUserGroupOptions, USER_GROUP_MAX_PAGE_SIZE } from './user-group-utils';
import { useGroupDialog, useDeleteGroupDialog } from './billing-group-management-state';
import { toModelFilters, AdminFiltersToolbar, DEFAULT_ADMIN_FILTERS } from './admin-filters-toolbar';

export function BillingGroupManagementView() {
  const state = useBillingGroupManagementPage();

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.billingGroups}
        action={
          <BillingGroupPageActions
            loading={state.groups.isLoading}
            onRefresh={() => void state.groups.refresh()}
            onCreate={state.dialog.openCreate}
          />
        }
      />
      <Card>
        <AdminFiltersToolbar
          filters={state.filters}
          searchPlaceholder={state.t('filters.searchGroups')}
          onChange={state.handleFiltersChange}
        />
        <BillingGroupTable
          rows={state.groups.items}
          userGroups={state.userGroups.items}
          total={state.groups.total}
          loading={state.groups.isLoading}
          table={state.table}
          onView={state.setDetailTarget}
          onEdit={state.dialog.openEdit}
          onDelete={state.deleteDialog.setDeleteTarget}
        />
      </Card>
      <BillingGroupDialogs state={state} />
    </DashboardContent>
  );
}

function useBillingGroupManagementPage() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'code' });
  const [filters, setFilters] = useState(DEFAULT_ADMIN_FILTERS);
  const [detailTarget, setDetailTarget] = useState<BillingGroup | null>(null);
  const groups = useBillingGroups(table.page, table.rowsPerPage, toModelFilters(filters));
  const models = useGlobalModels(0, 1000);
  const providers = useProviders(0, 1000);
  const providerKeys = useProviderKeysByProvider(providers.items);
  const userGroups = useUserGroups(0, USER_GROUP_MAX_PAGE_SIZE);
  const dialog = useGroupDialog(t);
  const deleteDialog = useDeleteGroupDialog(t);
  const userGroupOptions = enabledUserGroupOptions(userGroups.items);
  const handleFiltersChange = useCallback(
    (nextFilters: typeof DEFAULT_ADMIN_FILTERS) => {
      table.onResetPage();
      setFilters(nextFilters);
    },
    [table]
  );

  return {
    t,
    table,
    filters,
    groups,
    models,
    providers,
    providerKeys,
    userGroups,
    userGroupOptions,
    dialog,
    deleteDialog,
    detailTarget,
    setDetailTarget,
    handleFiltersChange,
  };
}

function BillingGroupDialogs({ state }: { state: ReturnType<typeof useBillingGroupManagementPage> }) {
  return (
    <>
      <BillingGroupDetailDialog
        group={state.detailTarget}
        models={state.models.items}
        providers={state.providers.items}
        providerKeysByProvider={state.providerKeys.itemsByProvider}
        userGroups={state.userGroups.items}
        open={!!state.detailTarget}
        onClose={() => state.setDetailTarget(null)}
      />
      <BillingGroupDialog
        dialog={state.dialog}
        models={state.models.items}
        providers={state.providers.items}
        providerKeysByProvider={state.providerKeys.itemsByProvider}
        userGroups={state.userGroupOptions}
      />
      <DeleteBillingGroupDialog
        open={!!state.deleteDialog.deleteTarget}
        targetName={state.deleteDialog.deleteTarget?.name ?? ''}
        onClose={() => state.deleteDialog.setDeleteTarget(null)}
        onConfirm={state.deleteDialog.confirmDelete}
      />
    </>
  );
}

function BillingGroupPageActions({
  loading,
  onRefresh,
  onCreate,
}: {
  loading: boolean;
  onRefresh: () => void;
  onCreate: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction="row" spacing={1}>
      <RefreshButton loading={loading} onClick={onRefresh} />
      <AddButton onClick={onCreate}>{t('actions.addBillingGroup')}</AddButton>
    </Stack>
  );
}

function DeleteBillingGroupDialog({
  open,
  targetName,
  onClose,
  onConfirm,
}: {
  open: boolean;
  targetName: string;
  onClose: () => void;
  onConfirm: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <ConfirmDialog
      open={open}
      onClose={onClose}
      title={t('dialogs.deleteBillingGroup')}
      content={t('billingGroups.deleteConfirm', { name: targetName })}
      cancelText={t('common.cancel')}
      action={
        <Button variant="contained" color="error" onClick={onConfirm}>
          {t('common.delete')}
        </Button>
      }
    />
  );
}

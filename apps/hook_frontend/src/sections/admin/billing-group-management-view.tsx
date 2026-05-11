'use client';

import { useState, useCallback } from 'react';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { useGlobalModels } from 'src/actions/models';
import { useProviders } from 'src/actions/providers';
import { useBillingGroups } from 'src/actions/groups';
import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { useTable } from 'src/components/table';
import { ConfirmDialog } from 'src/components/custom-dialog';

import { BillingGroupTable } from './billing-group-table';
import { BillingGroupDialog } from './billing-group-dialog';
import { AddButton, RefreshButton, AdminBreadcrumbs } from './shared';
import { useGroupDialog, useDeleteGroupDialog } from './billing-group-management-state';
import { toModelFilters, AdminFiltersToolbar, DEFAULT_ADMIN_FILTERS } from './admin-filters-toolbar';

export function BillingGroupManagementView() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'code' });
  const [filters, setFilters] = useState(DEFAULT_ADMIN_FILTERS);
  const groups = useBillingGroups(table.page, table.rowsPerPage, toModelFilters(filters));
  const models = useGlobalModels(0, 1000);
  const providers = useProviders(0, 1000);
  const dialog = useGroupDialog(t);
  const deleteDialog = useDeleteGroupDialog(t);

  const handleFiltersChange = useCallback(
    (nextFilters: typeof DEFAULT_ADMIN_FILTERS) => {
      table.onResetPage();
      setFilters(nextFilters);
    },
    [table]
  );

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.billingGroups}
        action={
          <Stack direction="row" spacing={1}>
            <RefreshButton loading={groups.isLoading} onClick={() => void groups.refresh()} />
            <AddButton onClick={dialog.openCreate}>{t('actions.addBillingGroup')}</AddButton>
          </Stack>
        }
      />
      <Card>
        <AdminFiltersToolbar
          filters={filters}
          searchPlaceholder={t('filters.searchGroups')}
          onChange={handleFiltersChange}
        />
        <BillingGroupTable
          rows={groups.items}
          total={groups.total}
          loading={groups.isLoading}
          table={table}
          onEdit={dialog.openEdit}
          onDelete={deleteDialog.setDeleteTarget}
        />
      </Card>
      <BillingGroupDialog dialog={dialog} models={models.items} providers={providers.items} />
      <ConfirmDialog
        open={!!deleteDialog.deleteTarget}
        onClose={() => deleteDialog.setDeleteTarget(null)}
        title={t('dialogs.deleteBillingGroup')}
        content={t('billingGroups.deleteConfirm', { name: deleteDialog.deleteTarget?.name ?? '' })}
        cancelText={t('common.cancel')}
        action={
          <Button variant="contained" color="error" onClick={deleteDialog.confirmDelete}>
            {t('common.delete')}
          </Button>
        }
      />
    </DashboardContent>
  );
}

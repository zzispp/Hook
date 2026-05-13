'use client';

import { useMemo, useState, useCallback } from 'react';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useRoles, useMenuItems, useUnboundApis } from 'src/actions/rbac';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { useTable } from 'src/components/table';

import { RoleTable } from './role-table';
import { RoleFormDialog } from './role-form-dialog';
import { RoleDeleteDialog } from './role-delete-dialog';
import { RolePermissionDialog } from './role-permission-dialog';
import { useRoleManagementActions } from './role-management-actions';
import { AddButton, RefreshButton, AdminBreadcrumbs } from './shared';
import {
  filterRoleAssignableApis,
  filterRoleAssignableApiIds,
} from './role-permission-utils';
import { toEnabledFilters, AdminFiltersToolbar, DEFAULT_ADMIN_FILTERS } from './admin-filters-toolbar';
import {
  useRoleFormState,
  useRoleDeleteState,
  useRolePermissionState,
} from './role-management-state';

export function RoleManagementView() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'sort_order' });
  const [filters, setFilters] = useState(DEFAULT_ADMIN_FILTERS);
  const roles = useRoles(table.page, table.rowsPerPage, toEnabledFilters(filters));
  const menuItems = useMenuItems(0, 100);
  const apis = useUnboundApis(0, 100);
  const formState = useRoleFormState();
  const deleteState = useRoleDeleteState();
  const permissionState = useRolePermissionState();
  const roleAssignableApis = useMemo(
    () => filterRoleAssignableApis(apis.items, permissionState.readOnlyApis),
    [apis.items, permissionState.readOnlyApis]
  );
  const roleAssignableSelectedApis = useMemo(
    () => filterRoleAssignableApiIds(permissionState.selectedApis, permissionState.readOnlyApis),
    [permissionState.readOnlyApis, permissionState.selectedApis]
  );
  const actionState = useRoleManagementActions({ deleteState, formState, permissionState });
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
        headingCode={DASHBOARD_MENU_CODES.roleManagement}
        action={
          <Stack direction="row" spacing={1}>
            <RefreshButton loading={roles.isLoading} onClick={() => void roles.refresh()} />
            <AddButton onClick={formState.openCreate}>{t('actions.addRole')}</AddButton>
          </Stack>
        }
      />

      <Card>
        <AdminFiltersToolbar
          filters={filters}
          searchPlaceholder={t('filters.searchRoles')}
          onChange={handleFiltersChange}
        />
        <RoleTable
          loading={roles.isLoading}
          rows={roles.items}
          table={table}
          total={roles.total}
          onDelete={deleteState.setTarget}
          onEdit={formState.openEdit}
          onPermissions={permissionState.open}
        />
      </Card>

      <RoleFormDialog
        form={formState.form}
        editing={formState.editing}
        open={formState.open}
        title={formState.editing ? t('dialogs.editRole') : t('dialogs.createRole')}
        submitting={actionState.submitting}
        onClose={formState.close}
        onFormChange={formState.setForm}
        onSubmit={actionState.submitRole}
      />
      <RolePermissionDialog
        role={permissionState.target}
        loading={permissionState.loading}
        submitting={actionState.submitting}
        apis={roleAssignableApis}
        readOnlyApis={permissionState.readOnlyApis}
        menus={menuItems.items}
        selectedApis={roleAssignableSelectedApis}
        selectedMenus={permissionState.selectedMenus}
        onSelectedApisChange={permissionState.setSelectedApis}
        onSelectedMenusChange={permissionState.setSelectedMenus}
        onClose={permissionState.close}
        onSubmit={actionState.savePermissions}
      />
      <RoleDeleteDialog
        target={deleteState.target}
        onClose={() => deleteState.setTarget(null)}
        onConfirm={actionState.confirmDelete}
      />
    </DashboardContent>
  );
}

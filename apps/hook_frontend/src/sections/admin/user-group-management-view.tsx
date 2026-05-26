'use client';

import type { UserGroup } from 'src/types/user-group';

import { useState, useCallback } from 'react';

import Card from '@mui/material/Card';
import Button from '@mui/material/Button';

import { useTranslate } from 'src/locales/use-locales';
import { useUserGroups } from 'src/actions/user-groups';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { useTable } from 'src/components/table';
import { ConfirmDialog } from 'src/components/custom-dialog';

import { AdminBreadcrumbs } from './shared';
import { UserGroupTable } from './user-group-table';
import { UserGroupDialog } from './user-group-dialog';
import { RefreshAddActions } from './admin-page-actions';
import { isDefaultUserGroup } from './user-group-management-utils';
import { UserGroupMembersDialog } from './user-group-members-dialog';
import { UserGroupAssignmentDialog } from './user-group-assignment-dialog';
import { enabledUserGroupOptions, USER_GROUP_MAX_PAGE_SIZE } from './user-group-utils';
import { useUserGroupDialog, useUserGroupDeleteDialog } from './user-group-management-state';
import {
  toUserGroupFilters,
  AdminFiltersToolbar,
  DEFAULT_ADMIN_FILTERS,
} from './admin-filters-toolbar';

export function UserGroupManagementView() {
  const state = useUserGroupManagementState();

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.userGroups}
        action={
          <RefreshAddActions
            loading={state.userGroups.isLoading}
            addLabel={state.t('actions.addUserGroup')}
            onAdd={state.dialog.openCreate}
            onRefresh={() => void state.userGroups.refresh()}
          />
        }
      />
      <Card>
        <AdminFiltersToolbar
          filters={state.filters}
          searchPlaceholder={state.t('filters.searchUserGroups')}
          onChange={state.handleFiltersChange}
        />
        <UserGroupTable
          loading={state.userGroups.isLoading}
          rows={state.userGroups.items}
          table={state.table}
          total={state.userGroups.total}
          onAssign={state.setAssignTarget}
          onDelete={state.deleteDialog.setTarget}
          onEdit={state.dialog.openEdit}
          onMembers={state.setMembersTarget}
        />
      </Card>
      <UserGroupDialogs state={state} />
    </DashboardContent>
  );
}

function useUserGroupManagementState() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'sort_order' });
  const [filters, setFilters] = useState(DEFAULT_ADMIN_FILTERS);
  const [membersTarget, setMembersTarget] = useState<UserGroup | null>(null);
  const [assignTarget, setAssignTarget] = useState<UserGroup | null>(null);
  const userGroups = useUserGroups(table.page, table.rowsPerPage, toUserGroupFilters(filters));
  const allUserGroups = useUserGroups(0, USER_GROUP_MAX_PAGE_SIZE);
  const assignableGroups = useUserGroups(0, USER_GROUP_MAX_PAGE_SIZE, { is_active: true });
  const dialog = useUserGroupDialog(t);
  const deleteDialog = useUserGroupDeleteDialog(t);
  const assignmentOptions = enabledUserGroupOptions(assignableGroups.items);
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
    userGroups,
    allUserGroups,
    assignableGroups,
    assignmentOptions,
    dialog,
    deleteDialog,
    membersTarget,
    assignTarget,
    setMembersTarget,
    setAssignTarget,
    handleFiltersChange,
  };
}

function UserGroupDialogs({ state }: { state: ReturnType<typeof useUserGroupManagementState> }) {
  const { t } = useTranslate('admin');
  const deleteTarget = state.deleteDialog.target;

  return (
    <>
      <UserGroupDialog dialog={state.dialog} />
      <UserGroupAssignmentDialog
        initialGroup={state.assignTarget}
        displayGroups={state.allUserGroups.items}
        groups={state.assignmentOptions}
        onClose={() => state.setAssignTarget(null)}
        onAssigned={() => {
          void state.userGroups.refresh();
          void state.allUserGroups.refresh();
          void state.assignableGroups.refresh();
        }}
      />
      <UserGroupMembersDialog
        group={state.membersTarget}
        onClose={() => state.setMembersTarget(null)}
      />
      <ConfirmDialog
        open={!!deleteTarget && !isDefaultUserGroup(deleteTarget)}
        onClose={() => state.deleteDialog.setTarget(null)}
        title={t('dialogs.deleteUserGroup')}
        content={t('dialogs.deleteContent', { name: deleteTarget?.name ?? '' })}
        cancelText={t('common.cancel')}
        action={
          <Button variant="contained" color="error" onClick={state.deleteDialog.confirmDelete}>
            {t('common.delete')}
          </Button>
        }
      />
    </>
  );
}

'use client';

import type { SystemUser } from 'src/types/rbac';

import { useState, useCallback } from 'react';

import Card from '@mui/material/Card';
import Button from '@mui/material/Button';

import { useGlobalModels } from 'src/actions/models';
import { useProviders } from 'src/actions/providers';
import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';
import { useRoles, useUsers, createUser, deleteUser, updateUser } from 'src/actions/rbac';

import { toast } from 'src/components/snackbar';
import { useTable } from 'src/components/table';
import { ConfirmDialog } from 'src/components/custom-dialog';

import { UserTable } from './user-table';
import { AdminBreadcrumbs } from './shared';
import { UserFormDialog } from './user-form-dialog';
import { UserTokenDialog } from './user-token-dialog';
import { UserWalletDialog } from './user-wallet-dialog';
import { RefreshAddActions } from './admin-page-actions';
import { toUserFilters, AdminFiltersToolbar, DEFAULT_ADMIN_FILTERS } from './admin-filters-toolbar';
import {
  formFromUser,
  formToPayload,
  roleFilterOptions,
  DEFAULT_USER_FORM,
  enabledRoleOptions,
} from './user-management-utils';

export function UserManagementView() {
  const state = useUserManagementState();

  return (
    <DashboardContent maxWidth="xl">
      <UserManagementHeader state={state} />
      <UserManagementTableCard state={state} />
      <UserManagementDialogs state={state} />
    </DashboardContent>
  );
}

function useUserManagementState() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'username' });
  const [filters, setFilters] = useState(DEFAULT_ADMIN_FILTERS);
  const users = useUsers(table.page, table.rowsPerPage, toUserFilters(filters));
  const roles = useRoles(0, 100);
  const models = useGlobalModels(0, 1000);
  const providers = useProviders(0, 1000);
  const dialog = useUserDialog(t, () => void users.refresh());
  const [deleteTarget, setDeleteTarget] = useState<SystemUser | null>(null);
  const [walletUser, setWalletUser] = useState<SystemUser | null>(null);
  const [tokenUser, setTokenUser] = useState<SystemUser | null>(null);
  const roleOptions = enabledRoleOptions(roles.items);

  const handleFiltersChange = useCallback(
    (nextFilters: typeof DEFAULT_ADMIN_FILTERS) => {
      table.onResetPage();
      setFilters(nextFilters);
    },
    [table]
  );

  const confirmDelete = useCallback(async () => {
    if (!deleteTarget) return;
    try {
      await deleteUser(deleteTarget.id);
      toast.success(t('messages.userDeleted'));
      setDeleteTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteTarget, t]);

  return {
    t,
    table,
    users,
    roles,
    models,
    dialog,
    filters,
    providers,
    tokenUser,
    walletUser,
    roleOptions,
    deleteTarget,
    confirmDelete,
    setTokenUser,
    setWalletUser,
    setDeleteTarget,
    handleFiltersChange,
  };
}

function UserManagementHeader({ state }: { state: ReturnType<typeof useUserManagementState> }) {
  return (
    <AdminBreadcrumbs
      headingCode={DASHBOARD_MENU_CODES.userManagement}
      action={
        <RefreshAddActions
          loading={state.users.isLoading}
          addLabel={state.t('actions.addUser')}
          onAdd={() => state.dialog.openCreate(state.roleOptions[0]?.code)}
          onRefresh={() => void state.users.refresh()}
        />
      }
    />
  );
}

function UserManagementTableCard({ state }: { state: ReturnType<typeof useUserManagementState> }) {
  const { t } = useTranslate('admin');

  return (
    <Card>
      <AdminFiltersToolbar
        filters={state.filters}
        roleOptions={roleFilterOptions(state.roleOptions)}
        searchPlaceholder={t('filters.searchUsers')}
        onChange={state.handleFiltersChange}
      />
      <UserTable
        rows={state.users.items}
        roles={state.roles.items}
        total={state.users.total}
        loading={state.users.isLoading}
        table={state.table}
        onEdit={state.dialog.openEdit}
        onWallet={state.setWalletUser}
        onTokens={state.setTokenUser}
        onDelete={state.setDeleteTarget}
      />
    </Card>
  );
}

function UserManagementDialogs({ state }: { state: ReturnType<typeof useUserManagementState> }) {
  const { t } = useTranslate('admin');

  return (
    <>
      <UserFormDialog
        dialog={state.dialog}
        roles={state.roleOptions}
        models={state.models.items}
        providers={state.providers.items}
      />
      <UserWalletDialog
        user={state.walletUser}
        onClose={() => state.setWalletUser(null)}
        onChanged={() => state.users.refresh()}
      />
      <UserTokenDialog user={state.tokenUser} onClose={() => state.setTokenUser(null)} />
      <ConfirmDialog
        open={!!state.deleteTarget}
        onClose={() => state.setDeleteTarget(null)}
        title={t('dialogs.deleteUser')}
        content={t('dialogs.deleteContent', { name: state.deleteTarget?.username ?? '' })}
        cancelText={t('common.cancel')}
        action={
          <Button variant="contained" color="error" onClick={state.confirmDelete}>
            {t('common.delete')}
          </Button>
        }
      />
    </>
  );
}

function useUserDialog(t: ReturnType<typeof useTranslate>['t'], refresh: VoidFunction) {
  const [form, setForm] = useState(DEFAULT_USER_FORM);
  const [editing, setEditing] = useState<SystemUser | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  const close = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_USER_FORM);
  }, []);

  const openCreate = useCallback((defaultRole = '') => {
    setEditing(null);
    setCreating(true);
    setForm({ ...DEFAULT_USER_FORM, role: defaultRole });
  }, []);

  const openEdit = useCallback((user: SystemUser) => {
    setEditing(user);
    setCreating(false);
    setForm(formFromUser(user));
  }, []);

  const submit = useCallback(async () => {
    setSubmitting(true);
    try {
      if (editing) {
        await updateUser(editing.id, formToPayload(form));
        toast.success(t('messages.userUpdated'));
      } else {
        await createUser(formToPayload(form));
        toast.success(t('messages.userCreated'));
      }
      refresh();
      close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [close, editing, form, refresh, t]);

  return {
    close,
    creating,
    editing,
    form,
    open: creating || !!editing,
    openCreate,
    openEdit,
    setForm,
    submit,
    submitting,
  };
}

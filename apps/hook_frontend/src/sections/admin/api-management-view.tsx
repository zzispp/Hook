'use client';

import type { ApiPermission, ApiPermissionInput } from 'src/types/rbac';

import { useMemo, useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import CircularProgress from '@mui/material/CircularProgress';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';
import { useApis, createApi, deleteApi, updateApi, getApiMenus, useMenuItems } from 'src/actions/rbac';

import { toast } from 'src/components/snackbar';
import { useTable } from 'src/components/table';
import { ConfirmDialog } from 'src/components/custom-dialog';

import { apiTableHead } from './api-table-head';
import { ApiMenuSelect } from './api-menu-select';
import { RefreshAddActions } from './admin-page-actions';
import { ApiManagementTable } from './api-management-table';
import {
  toEnabledFilters,
  AdminFiltersToolbar,
  DEFAULT_ADMIN_FILTERS,
} from './admin-filters-toolbar';
import {
  SwitchRow,
  TextFieldRow,
  METHOD_OPTIONS,
  AdminBreadcrumbs,
  ManagementDialog,
} from './shared';

const DEFAULT_FORM: ApiPermissionInput = {
  code: '',
  method: 'GET',
  path_pattern: '',
  name: '',
  enabled: true,
  menu_item_ids: [],
};

export function ApiManagementView() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'name' });
  const [filters, setFilters] = useState(DEFAULT_ADMIN_FILTERS);
  const apis = useApis(table.page, table.rowsPerPage, toEnabledFilters(filters));
  const { items, total, isLoading } = apis;
  const menuItems = useMenuItems(0, 100);
  const tableHead = useMemo(() => apiTableHead(t), [t]);

  const [form, setForm] = useState<ApiPermissionInput>(DEFAULT_FORM);
  const [editing, setEditing] = useState<ApiPermission | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [loadingBindings, setLoadingBindings] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<ApiPermission | null>(null);

  const dialogOpen = creating || !!editing;
  const handleFiltersChange = useCallback(
    (nextFilters: typeof DEFAULT_ADMIN_FILTERS) => {
      table.onResetPage();
      setFilters(nextFilters);
    },
    [table]
  );

  const handleOpenCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm({ ...DEFAULT_FORM });
  }, []);

  const handleOpenEdit = useCallback(async (api: ApiPermission) => {
    setEditing(api);
    setForm({
      code: api.code,
      method: api.method,
      path_pattern: api.path_pattern,
      name: api.name,
      enabled: api.enabled,
      menu_item_ids: [],
    });
    setLoadingBindings(true);
    try {
      const binding = await getApiMenus(api.id);
      setForm((current) => ({ ...current, menu_item_ids: binding.menu_item_ids }));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.loadBindingsFailed'));
    } finally {
      setLoadingBindings(false);
    }
  }, [t]);

  const handleCloseDialog = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_FORM);
  }, []);

  const handleSubmit = useCallback(async () => {
    setSubmitting(true);
    try {
      if (editing) {
        await updateApi(editing.id, form);
        toast.success(t('messages.apiUpdated'));
      } else {
        await createApi(form);
        toast.success(t('messages.apiCreated'));
      }
      handleCloseDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [editing, form, handleCloseDialog, t]);

  const handleDelete = useCallback(async () => {
    if (!deleteTarget) return;

    try {
      await deleteApi(deleteTarget.id);
      toast.success(t('messages.apiDeleted'));
      setDeleteTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteTarget, t]);

  return (
    <DashboardContent maxWidth="xl">
    <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.apiManagement}
        action={
          <RefreshAddActions
            loading={isLoading}
            addLabel={t('actions.addApi')}
            onAdd={handleOpenCreate}
            onRefresh={() => void apis.refresh()}
          />
        }
      />

      <Card>
        <AdminFiltersToolbar
          filters={filters}
          searchPlaceholder={t('filters.searchApis')}
          onChange={handleFiltersChange}
        />
        <ApiManagementTable
          apis={items}
          loading={isLoading}
          menus={menuItems.items}
          table={table}
          tableHead={tableHead}
          total={total}
          onDelete={setDeleteTarget}
          onEdit={handleOpenEdit}
        />
      </Card>

      <ManagementDialog
        open={dialogOpen}
        title={editing ? t('dialogs.editApi') : t('dialogs.createApi')}
        submitting={submitting}
        onClose={handleCloseDialog}
        onSubmit={handleSubmit}
      >
        <TextFieldRow
          required
          label={t('common.name')}
          value={form.name}
          onChange={(value) => setForm((current) => ({ ...current, name: value }))}
        />
        <TextFieldRow
          required
          label={t('common.code')}
          value={form.code}
          onChange={(value) => setForm((current) => ({ ...current, code: value }))}
        />
        <TextFieldRow
          required
          select
          label={t('common.method')}
          value={form.method}
          onChange={(value) => setForm((current) => ({ ...current, method: value }))}
        >
          {METHOD_OPTIONS.map((method) => (
            <MenuItem key={method} value={method}>
              {method}
            </MenuItem>
          ))}
        </TextFieldRow>
        <TextFieldRow
          required
          label={t('fields.pathPattern')}
          value={form.path_pattern}
          helperText={t('helper.pathPatternExample')}
          onChange={(value) => setForm((current) => ({ ...current, path_pattern: value }))}
        />
        <SwitchRow
          label={t('common.enabled')}
          checked={form.enabled}
          onChange={(enabled) => setForm((current) => ({ ...current, enabled }))}
        />
        {loadingBindings ? (
          <Box sx={{ py: 1, color: 'text.secondary', display: 'flex', gap: 1, alignItems: 'center' }}>
            <CircularProgress size={18} />
            {t('messages.loadingPermissions')}
          </Box>
        ) : (
          <ApiMenuSelect
            menus={menuItems.items}
            value={form.menu_item_ids}
            onChange={(menu_item_ids) => setForm((current) => ({ ...current, menu_item_ids }))}
          />
        )}
      </ManagementDialog>

      <ConfirmDialog
        open={!!deleteTarget}
        onClose={() => setDeleteTarget(null)}
        title={t('dialogs.deleteApi')}
        content={t('dialogs.deleteContent', { name: deleteTarget?.name ?? '' })}
        cancelText={t('common.cancel')}
        action={
          <Button variant="contained" color="error" onClick={handleDelete}>
            {t('common.delete')}
          </Button>
        }
      />
    </DashboardContent>
  );
}

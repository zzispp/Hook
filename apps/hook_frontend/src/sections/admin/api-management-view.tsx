'use client';

import type { ApiPermission, ApiPermissionInput } from 'src/types/rbac';

import { useMemo, useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import MenuItem from '@mui/material/MenuItem';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import CircularProgress from '@mui/material/CircularProgress';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useApis, createApi, deleteApi, updateApi, getApiMenus, useMenuItems } from 'src/actions/rbac';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { ConfirmDialog } from 'src/components/custom-dialog';
import { useTable, TableNoData, TablePaginationCustom } from 'src/components/table';

import { apiTableHead } from './api-table-head';
import { ApiMenuSelect } from './api-menu-select';
import {
  toEnabledFilters,
  AdminFiltersToolbar,
  DEFAULT_ADMIN_FILTERS,
} from './admin-filters-toolbar';
import {
  AddButton,
  SwitchRow,
  MethodLabel,
  TextFieldRow,
  EnabledLabel,
  METHOD_OPTIONS,
  AdminBreadcrumbs,
  ManagementDialog,
  TableLoadingRows,
  translatedApiName,
  translatedApiGroup,
  ManagementTableHead,
} from './shared';

// ----------------------------------------------------------------------

const DEFAULT_FORM: ApiPermissionInput = {
  code: '',
  method: 'GET',
  path_pattern: '',
  name: '',
  group: '',
  enabled: true,
  menu_item_ids: [],
};

// ----------------------------------------------------------------------

export function ApiManagementView() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'name' });
  const [filters, setFilters] = useState(DEFAULT_ADMIN_FILTERS);
  const { items, total, isLoading } = useApis(
    table.page,
    table.rowsPerPage,
    toEnabledFilters(filters)
  );
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
      group: api.group,
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
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.apiManagement')}
        action={<AddButton onClick={handleOpenCreate}>{t('actions.addApi')}</AddButton>}
      />

      <Card>
        <AdminFiltersToolbar
          filters={filters}
          searchPlaceholder={t('filters.searchApis')}
          onChange={handleFiltersChange}
        />
        <Scrollbar>
          <Table sx={{ minWidth: 980 }}>
            <ManagementTableHead head={tableHead} />
            <TableBody>
              {isLoading ? (
                <TableLoadingRows head={tableHead} rows={table.rowsPerPage} />
              ) : (
                items.map((row) => (
                  <TableRow key={row.id} hover>
                    <TableCell>
                      <MethodLabel method={row.method} />
                    </TableCell>
                    <TableCell>{translatedApiName(row, t)}</TableCell>
                    <TableCell sx={{ fontFamily: 'monospace' }}>{row.code}</TableCell>
                    <TableCell sx={{ fontFamily: 'monospace' }}>{row.path_pattern}</TableCell>
                    <TableCell>{translatedApiGroup(row.group, t)}</TableCell>
                    <TableCell>
                      <EnabledLabel enabled={row.enabled} />
                    </TableCell>
                    <TableCell align="right">
                      <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
                        <Tooltip title={t('common.edit')}>
                          <IconButton onClick={() => handleOpenEdit(row)}>
                            <Iconify icon="solar:pen-bold" />
                          </IconButton>
                        </Tooltip>
                        <Tooltip title={t('common.delete')}>
                          <IconButton color="error" onClick={() => setDeleteTarget(row)}>
                            <Iconify icon="solar:trash-bin-trash-bold" />
                          </IconButton>
                        </Tooltip>
                      </Box>
                    </TableCell>
                  </TableRow>
                ))
              )}

              <TableNoData title={t('common.noData')} notFound={!isLoading && items.length === 0} />
            </TableBody>
          </Table>
        </Scrollbar>

        <TablePaginationCustom
          page={table.page}
          count={total}
          rowsPerPage={table.rowsPerPage}
          onPageChange={table.onChangePage}
          onRowsPerPageChange={table.onChangeRowsPerPage}
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
        <TextFieldRow
          label={t('common.group')}
          value={form.group}
          onChange={(value) => setForm((current) => ({ ...current, group: value }))}
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

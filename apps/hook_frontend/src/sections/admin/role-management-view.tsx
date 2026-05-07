'use client';

import type { TableHeadCellProps } from 'src/components/table';
import type {
  Role,
  RoleInput,
  ApiPermission,
  MenuItem as RbacMenuItem,
} from 'src/types/rbac';

import { useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import List from '@mui/material/List';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Tooltip from '@mui/material/Tooltip';
import Checkbox from '@mui/material/Checkbox';
import TableRow from '@mui/material/TableRow';
import ListItem from '@mui/material/ListItem';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import DialogTitle from '@mui/material/DialogTitle';
import ListItemText from '@mui/material/ListItemText';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import ListItemButton from '@mui/material/ListItemButton';

import { DashboardContent } from 'src/layouts/dashboard';
import {
  useApis,
  useRoles,
  createRole,
  updateRole,
  deleteRole,
  getRoleApis,
  useMenuItems,
  getRoleMenus,
  updateRoleApis,
  updateRoleMenus,
} from 'src/actions/rbac';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { ConfirmDialog } from 'src/components/custom-dialog';
import { useTable, TableNoData, TablePaginationCustom } from 'src/components/table';

import {
  AddButton,
  SwitchRow,
  MethodLabel,
  TextFieldRow,
  EnabledLabel,
  BooleanLabel,
  AdminBreadcrumbs,
  ManagementDialog,
  TableLoadingRows,
  ManagementTableHead,
} from './shared';

// ----------------------------------------------------------------------

const TABLE_HEAD: TableHeadCellProps[] = [
  { id: 'name', label: 'Role', width: 220 },
  { id: 'code', label: 'Code', width: 200 },
  { id: 'description', label: 'Description' },
  { id: 'sort_order', label: 'Sort', width: 100 },
  { id: 'enabled', label: 'Status', width: 120 },
  { id: 'system', label: 'Type', width: 120 },
  { id: '', width: 144 },
];

const DEFAULT_FORM: RoleInput = {
  code: '',
  name: '',
  description: '',
  enabled: true,
  sort_order: 0,
};

// ----------------------------------------------------------------------

export function RoleManagementView() {
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'sort_order' });
  const { items, total, isLoading } = useRoles(table.page, table.rowsPerPage);
  const apis = useApis(0, 100);
  const menuItems = useMenuItems(0, 100);

  const [form, setForm] = useState<RoleInput>(DEFAULT_FORM);
  const [editing, setEditing] = useState<Role | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Role | null>(null);
  const [bindingTarget, setBindingTarget] = useState<Role | null>(null);
  const [bindingTab, setBindingTab] = useState<'apis' | 'menus'>('apis');
  const [selectedApis, setSelectedApis] = useState<string[]>([]);
  const [selectedMenus, setSelectedMenus] = useState<string[]>([]);
  const [bindingLoading, setBindingLoading] = useState(false);

  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm({ ...DEFAULT_FORM });
  }, []);

  const openEdit = useCallback((role: Role) => {
    setEditing(role);
    setForm({
      code: role.code,
      name: role.name,
      description: role.description,
      enabled: role.enabled,
      sort_order: role.sort_order,
    });
  }, []);

  const closeDialog = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_FORM);
  }, []);

  const submitRole = useCallback(async () => {
    setSubmitting(true);
    try {
      if (editing) {
        await updateRole(editing.code, form);
        toast.success('Role updated');
      } else {
        await createRole(form);
        toast.success('Role created');
      }
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Save failed');
    } finally {
      setSubmitting(false);
    }
  }, [closeDialog, editing, form]);

  const confirmDelete = useCallback(async () => {
    if (!deleteTarget) return;

    try {
      await deleteRole(deleteTarget.code);
      toast.success('Role deleted');
      setDeleteTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Delete failed');
    }
  }, [deleteTarget]);

  const openBindings = useCallback(async (role: Role) => {
    setBindingTarget(role);
    setBindingLoading(true);
    setBindingTab('apis');
    try {
      const [apiBinding, menuBinding] = await Promise.all([
        getRoleApis(role.code),
        getRoleMenus(role.code),
      ]);
      setSelectedApis(apiBinding.api_permission_ids);
      setSelectedMenus(menuBinding.menu_item_ids);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Load bindings failed');
    } finally {
      setBindingLoading(false);
    }
  }, []);

  const saveBindings = useCallback(async () => {
    if (!bindingTarget) return;

    setSubmitting(true);
    try {
      await Promise.all([
        updateRoleApis(bindingTarget.code, selectedApis),
        updateRoleMenus(bindingTarget.code, selectedMenus),
      ]);
      toast.success('Role permissions updated');
      setBindingTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Save bindings failed');
    } finally {
      setSubmitting(false);
    }
  }, [bindingTarget, selectedApis, selectedMenus]);

  return (
    <DashboardContent>
      <AdminBreadcrumbs heading="Role Management" action={<AddButton onClick={openCreate}>Add role</AddButton>} />

      <Card>
        <Scrollbar>
          <Table sx={{ minWidth: 1050 }}>
            <ManagementTableHead head={TABLE_HEAD} />
            <TableBody>
              {isLoading ? (
                <TableLoadingRows head={TABLE_HEAD} rows={table.rowsPerPage} />
              ) : (
                items.map((row) => (
                  <TableRow key={row.code} hover>
                    <TableCell>{row.name}</TableCell>
                    <TableCell sx={{ fontFamily: 'monospace' }}>{row.code}</TableCell>
                    <TableCell>{row.description || '-'}</TableCell>
                    <TableCell>{row.sort_order}</TableCell>
                    <TableCell>
                      <EnabledLabel enabled={row.enabled} />
                    </TableCell>
                    <TableCell>
                      <BooleanLabel enabled={row.system} trueText="System" falseText="Custom" />
                    </TableCell>
                    <TableCell align="right">
                      <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
                        <Tooltip title="Permissions">
                          <IconButton onClick={() => openBindings(row)}>
                            <Iconify icon="solar:shield-keyhole-bold-duotone" />
                          </IconButton>
                        </Tooltip>
                        <Tooltip title="Edit">
                          <span>
                            <IconButton disabled={row.system} onClick={() => openEdit(row)}>
                              <Iconify icon="solar:pen-bold" />
                            </IconButton>
                          </span>
                        </Tooltip>
                        <Tooltip title="Delete">
                          <span>
                            <IconButton color="error" disabled={row.system} onClick={() => setDeleteTarget(row)}>
                              <Iconify icon="solar:trash-bin-trash-bold" />
                            </IconButton>
                          </span>
                        </Tooltip>
                      </Box>
                    </TableCell>
                  </TableRow>
                ))
              )}

              <TableNoData notFound={!isLoading && items.length === 0} />
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
        open={creating || !!editing}
        title={editing ? 'Edit role' : 'Create role'}
        submitting={submitting}
        onClose={closeDialog}
        onSubmit={submitRole}
      >
        <TextFieldRow
          required
          disabled={!!editing}
          label="Code"
          value={form.code}
          onChange={(value) => setForm((current) => ({ ...current, code: value }))}
        />
        <TextFieldRow
          required
          label="Name"
          value={form.name}
          onChange={(value) => setForm((current) => ({ ...current, name: value }))}
        />
        <TextFieldRow
          label="Description"
          value={form.description}
          onChange={(value) => setForm((current) => ({ ...current, description: value }))}
        />
        <TextFieldRow
          type="number"
          label="Sort order"
          value={form.sort_order}
          onChange={(value) => setForm((current) => ({ ...current, sort_order: Number(value) }))}
        />
        <SwitchRow
          label="Enabled"
          checked={form.enabled}
          onChange={(enabled) => setForm((current) => ({ ...current, enabled }))}
        />
      </ManagementDialog>

      <BindingDialog
        role={bindingTarget}
        tab={bindingTab}
        loading={bindingLoading}
        submitting={submitting}
        apis={apis.items}
        menus={menuItems.items}
        selectedApis={selectedApis}
        selectedMenus={selectedMenus}
        onTabChange={setBindingTab}
        onSelectedApisChange={setSelectedApis}
        onSelectedMenusChange={setSelectedMenus}
        onClose={() => setBindingTarget(null)}
        onSubmit={saveBindings}
      />

      <ConfirmDialog
        open={!!deleteTarget}
        onClose={() => setDeleteTarget(null)}
        title="Delete role"
        content={`Delete ${deleteTarget?.name ?? ''}?`}
        action={
          <Button variant="contained" color="error" onClick={confirmDelete}>
            Delete
          </Button>
        }
      />
    </DashboardContent>
  );
}

function BindingDialog({
  role,
  tab,
  loading,
  submitting,
  apis,
  menus,
  selectedApis,
  selectedMenus,
  onTabChange,
  onSelectedApisChange,
  onSelectedMenusChange,
  onClose,
  onSubmit,
}: {
  role: Role | null;
  tab: 'apis' | 'menus';
  loading: boolean;
  submitting: boolean;
  apis: ApiPermission[];
  menus: RbacMenuItem[];
  selectedApis: string[];
  selectedMenus: string[];
  onTabChange: (value: 'apis' | 'menus') => void;
  onSelectedApisChange: (value: string[]) => void;
  onSelectedMenusChange: (value: string[]) => void;
  onClose: () => void;
  onSubmit: () => void;
}) {
  const toggleApi = (id: string) => {
    onSelectedApisChange(toggleValue(selectedApis, id));
  };

  const toggleMenu = (id: string) => {
    onSelectedMenusChange(toggleValue(selectedMenus, id));
  };

  return (
    <Dialog fullWidth maxWidth="md" open={!!role} onClose={onClose}>
      <DialogTitle>Role permissions: {role?.name}</DialogTitle>
      <DialogContent>
        <Box sx={{ display: 'flex', gap: 1, mb: 2 }}>
          <Button
            variant={tab === 'apis' ? 'contained' : 'outlined'}
            onClick={() => onTabChange('apis')}
          >
            API permissions
          </Button>
          <Button
            variant={tab === 'menus' ? 'contained' : 'outlined'}
            onClick={() => onTabChange('menus')}
          >
            Menu permissions
          </Button>
        </Box>

        {loading ? (
          <Box sx={{ py: 4, color: 'text.secondary' }}>Loading permissions...</Box>
        ) : (
          <Scrollbar sx={{ maxHeight: 520 }}>
            {tab === 'apis' ? (
              <List disablePadding>
                {apis.map((api) => (
                  <ListItem key={api.id} disablePadding>
                    <ListItemButton onClick={() => toggleApi(api.id)}>
                      <Checkbox edge="start" checked={selectedApis.includes(api.id)} tabIndex={-1} />
                      <ListItemText
                        primary={
                          <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
                            <MethodLabel method={api.method} />
                            <span>{api.name}</span>
                          </Box>
                        }
                        secondary={`${api.group || 'Ungrouped'} · ${api.path_pattern}`}
                      />
                    </ListItemButton>
                  </ListItem>
                ))}
              </List>
            ) : (
              <List disablePadding>
                {menus.map((menu) => (
                  <ListItem key={menu.id} disablePadding>
                    <ListItemButton onClick={() => toggleMenu(menu.id)}>
                      <Checkbox edge="start" checked={selectedMenus.includes(menu.id)} tabIndex={-1} />
                      <ListItemText primary={menu.title} secondary={`${menu.code} · ${menu.path}`} />
                    </ListItemButton>
                  </ListItem>
                ))}
              </List>
            )}
          </Scrollbar>
        )}
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onClose}>
          Cancel
        </Button>
        <Button variant="contained" loading={submitting} onClick={onSubmit}>
          Save permissions
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function toggleValue(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}

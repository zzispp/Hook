'use client';

import type { TableHeadCellProps } from 'src/components/table';
import type { Role, UserInput, SystemUser } from 'src/types/rbac';

import { useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import MenuItem from '@mui/material/MenuItem';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';

import { DashboardContent } from 'src/layouts/dashboard';
import { useRoles, useUsers, createUser, updateUser, deleteUser } from 'src/actions/rbac';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { ConfirmDialog } from 'src/components/custom-dialog';
import { useTable, TableNoData, TablePaginationCustom } from 'src/components/table';

import {
  AddButton,
  SwitchRow,
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
  { id: 'username', label: 'Username', width: 220 },
  { id: 'email', label: 'Email' },
  { id: 'role', label: 'Role', width: 160 },
  { id: 'auth_source', label: 'Source', width: 130 },
  { id: 'is_active', label: 'Status', width: 120 },
  { id: 'system', label: 'Type', width: 120 },
  { id: '', width: 96 },
];

const DEFAULT_FORM: UserInput = {
  username: '',
  password: '',
  email: '',
  role: '',
  is_active: true,
};

// ----------------------------------------------------------------------

export function UserManagementView() {
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'username' });
  const { items, total, isLoading } = useUsers(table.page, table.rowsPerPage);
  const roles = useRoles(0, 100);

  const [form, setForm] = useState<UserInput>(DEFAULT_FORM);
  const [editing, setEditing] = useState<SystemUser | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<SystemUser | null>(null);

  const roleOptions = roles.items.filter((role) => role.enabled);

  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm({
      ...DEFAULT_FORM,
      role: roleOptions[0]?.code ?? '',
    });
  }, [roleOptions]);

  const openEdit = useCallback((user: SystemUser) => {
    setEditing(user);
    setForm({
      username: user.username,
      password: '',
      email: user.email,
      role: user.role,
      is_active: user.is_active,
    });
  }, []);

  const closeDialog = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_FORM);
  }, []);

  const submitUser = useCallback(async () => {
    setSubmitting(true);
    try {
      if (editing) {
        await updateUser(editing.id, form);
        toast.success('User updated');
      } else {
        await createUser(form);
        toast.success('User created');
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
      await deleteUser(deleteTarget.id);
      toast.success('User deleted');
      setDeleteTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Delete failed');
    }
  }, [deleteTarget]);

  return (
    <DashboardContent>
      <AdminBreadcrumbs heading="User Management" action={<AddButton onClick={openCreate}>Add user</AddButton>} />

      <Card>
        <Scrollbar>
          <Table sx={{ minWidth: 980 }}>
            <ManagementTableHead head={TABLE_HEAD} />
            <TableBody>
              {isLoading ? (
                <TableLoadingRows head={TABLE_HEAD} rows={table.rowsPerPage} />
              ) : (
                items.map((row) => (
                  <TableRow key={row.id} hover>
                    <TableCell>{row.username}</TableCell>
                    <TableCell>{row.email}</TableCell>
                    <TableCell>{displayRole(row.role, roles.items)}</TableCell>
                    <TableCell>{row.auth_source}</TableCell>
                    <TableCell>
                      <EnabledLabel enabled={row.is_active} />
                    </TableCell>
                    <TableCell>
                      <BooleanLabel enabled={row.system} trueText="System" falseText="Local" />
                    </TableCell>
                    <TableCell align="right">
                      <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
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
        title={editing ? 'Edit user' : 'Create user'}
        submitting={submitting}
        onClose={closeDialog}
        onSubmit={submitUser}
      >
        <TextFieldRow
          required
          label="Username"
          value={form.username}
          onChange={(value) => setForm((current) => ({ ...current, username: value }))}
        />
        <TextFieldRow
          required
          label="Email"
          value={form.email}
          onChange={(value) => setForm((current) => ({ ...current, email: value }))}
        />
        <TextFieldRow
          required
          select
          label="Role"
          value={form.role}
          onChange={(value) => setForm((current) => ({ ...current, role: value }))}
        >
          {roleOptions.map((role) => (
            <MenuItem key={role.code} value={role.code}>
              {role.name} ({role.code})
            </MenuItem>
          ))}
        </TextFieldRow>
        <TextFieldRow
          required
          type="password"
          label={editing ? 'New password' : 'Password'}
          value={form.password}
          helperText={editing ? 'Backend currently requires a password on update.' : undefined}
          onChange={(value) => setForm((current) => ({ ...current, password: value }))}
        />
        <SwitchRow
          label="Active"
          checked={form.is_active}
          onChange={(isActive) => setForm((current) => ({ ...current, is_active: isActive }))}
        />
      </ManagementDialog>

      <ConfirmDialog
        open={!!deleteTarget}
        onClose={() => setDeleteTarget(null)}
        title="Delete user"
        content={`Delete ${deleteTarget?.username ?? ''}?`}
        action={
          <Button variant="contained" color="error" onClick={confirmDelete}>
            Delete
          </Button>
        }
      />
    </DashboardContent>
  );
}

function displayRole(code: string, roles: Role[]) {
  return roles.find((role) => role.code === code)?.name ?? code;
}

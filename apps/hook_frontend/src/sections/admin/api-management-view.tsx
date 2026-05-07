'use client';

import type { TableHeadCellProps } from 'src/components/table';
import type { ApiPermission, ApiPermissionInput } from 'src/types/rbac';

import { useState, useCallback } from 'react';

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

import { DashboardContent } from 'src/layouts/dashboard';
import { useApis, createApi, updateApi, deleteApi } from 'src/actions/rbac';

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
  METHOD_OPTIONS,
  AdminBreadcrumbs,
  ManagementDialog,
  TableLoadingRows,
  ManagementTableHead,
} from './shared';

// ----------------------------------------------------------------------

const TABLE_HEAD: TableHeadCellProps[] = [
  { id: 'method', label: 'Method', width: 110 },
  { id: 'name', label: 'Name', width: 220 },
  { id: 'code', label: 'Code', width: 220 },
  { id: 'path_pattern', label: 'Path pattern' },
  { id: 'group', label: 'Group', width: 160 },
  { id: 'enabled', label: 'Status', width: 120 },
  { id: '', width: 96 },
];

const DEFAULT_FORM: ApiPermissionInput = {
  code: '',
  method: 'GET',
  path_pattern: '',
  name: '',
  group: '',
  enabled: true,
};

// ----------------------------------------------------------------------

export function ApiManagementView() {
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'name' });
  const { items, total, isLoading } = useApis(table.page, table.rowsPerPage);

  const [form, setForm] = useState<ApiPermissionInput>(DEFAULT_FORM);
  const [editing, setEditing] = useState<ApiPermission | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<ApiPermission | null>(null);

  const dialogOpen = creating || !!editing;

  const handleOpenCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm({ ...DEFAULT_FORM });
  }, []);

  const handleOpenEdit = useCallback((api: ApiPermission) => {
    setEditing(api);
    setForm({
      code: api.code,
      method: api.method,
      path_pattern: api.path_pattern,
      name: api.name,
      group: api.group,
      enabled: api.enabled,
    });
  }, []);

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
        toast.success('API permission updated');
      } else {
        await createApi(form);
        toast.success('API permission created');
      }
      handleCloseDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Save failed');
    } finally {
      setSubmitting(false);
    }
  }, [editing, form, handleCloseDialog]);

  const handleDelete = useCallback(async () => {
    if (!deleteTarget) return;

    try {
      await deleteApi(deleteTarget.id);
      toast.success('API permission deleted');
      setDeleteTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Delete failed');
    }
  }, [deleteTarget]);

  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading="API Management"
        action={<AddButton onClick={handleOpenCreate}>Add API</AddButton>}
      />

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
                    <TableCell>
                      <MethodLabel method={row.method} />
                    </TableCell>
                    <TableCell>{row.name}</TableCell>
                    <TableCell sx={{ fontFamily: 'monospace' }}>{row.code}</TableCell>
                    <TableCell sx={{ fontFamily: 'monospace' }}>{row.path_pattern}</TableCell>
                    <TableCell>{row.group || '-'}</TableCell>
                    <TableCell>
                      <EnabledLabel enabled={row.enabled} />
                    </TableCell>
                    <TableCell align="right">
                      <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
                        <Tooltip title="Edit">
                          <IconButton onClick={() => handleOpenEdit(row)}>
                            <Iconify icon="solar:pen-bold" />
                          </IconButton>
                        </Tooltip>
                        <Tooltip title="Delete">
                          <IconButton color="error" onClick={() => setDeleteTarget(row)}>
                            <Iconify icon="solar:trash-bin-trash-bold" />
                          </IconButton>
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
        open={dialogOpen}
        title={editing ? 'Edit API permission' : 'Create API permission'}
        submitting={submitting}
        onClose={handleCloseDialog}
        onSubmit={handleSubmit}
      >
        <TextFieldRow
          required
          label="Name"
          value={form.name}
          onChange={(value) => setForm((current) => ({ ...current, name: value }))}
        />
        <TextFieldRow
          required
          label="Code"
          value={form.code}
          onChange={(value) => setForm((current) => ({ ...current, code: value }))}
        />
        <TextFieldRow
          required
          select
          label="Method"
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
          label="Path pattern"
          value={form.path_pattern}
          helperText="Example: /api/users/{id}"
          onChange={(value) => setForm((current) => ({ ...current, path_pattern: value }))}
        />
        <TextFieldRow
          label="Group"
          value={form.group}
          onChange={(value) => setForm((current) => ({ ...current, group: value }))}
        />
        <SwitchRow
          label="Enabled"
          checked={form.enabled}
          onChange={(enabled) => setForm((current) => ({ ...current, enabled }))}
        />
      </ManagementDialog>

      <ConfirmDialog
        open={!!deleteTarget}
        onClose={() => setDeleteTarget(null)}
        title="Delete API permission"
        content={`Delete ${deleteTarget?.name ?? ''}?`}
        action={
          <Button variant="contained" color="error" onClick={handleDelete}>
            Delete
          </Button>
        }
      />
    </DashboardContent>
  );
}

'use client';

import type { Announcement, AnnouncementInput } from 'src/types/operations';

import { useState, useCallback } from 'react';

import Card from '@mui/material/Card';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import TableContainer from '@mui/material/TableContainer';

import { fDateTime } from 'src/utils/format-time';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';
import {
  useAnnouncements,
  createAnnouncement,
  deleteAnnouncement,
  updateAnnouncement,
} from 'src/actions/operations';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { ConfirmDialog } from 'src/components/custom-dialog';
import {
  useTable,
  TableHeadCustom,
  TablePaginationCustom,
  tableStickyActionCellSx,
  withStickyActionHeadCell,
} from 'src/components/table';

import { AddButton, EnabledLabel, AdminBreadcrumbs } from 'src/sections/admin/shared';

import { AnnouncementTypeLabel } from './operation-labels';
import { AnnouncementFormDialog } from './announcement-form-dialog';

export function AnnouncementManagementView() {
  const state = useAnnouncementManagementState();

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.announcementManagement}
        action={
          <AddButton onClick={state.openCreate}>
            {state.t('operations.announcement.create')}
          </AddButton>
        }
      />
      <Card>
        <TextField
          fullWidth
          value={state.search}
          onChange={state.handleSearch}
          placeholder={state.t('operations.announcement.searchPlaceholder')}
          sx={{ p: 2.5 }}
        />
        <AnnouncementTable state={state} />
      </Card>
      <AnnouncementFormDialog
        open={state.formOpen}
        editing={state.editing}
        submitting={state.submitting}
        onClose={state.closeForm}
        onSubmit={state.submitForm}
      />
      <ConfirmDialog
        open={!!state.deleting}
        title={state.t('operations.announcement.deleteTitle')}
        content={state.deleting?.title}
        onClose={() => state.setDeleting(null)}
        action={
          <Button color="error" variant="contained" onClick={state.confirmDelete}>
            {state.t('common.delete')}
          </Button>
        }
      />
    </DashboardContent>
  );
}

function AnnouncementTable({
  state,
}: {
  state: ReturnType<typeof useAnnouncementManagementState>;
}) {
  const tableHead = [
    { id: 'title', label: state.t('operations.announcement.table.title') },
    { id: 'type', label: state.t('operations.announcement.table.type'), width: 120 },
    { id: 'enabled', label: state.t('operations.announcement.table.status'), width: 100 },
    { id: 'updated_at', label: state.t('operations.announcement.table.updated'), width: 180 },
    withStickyActionHeadCell({
      id: 'actions',
      label: state.t('common.actions'),
      width: 96,
      align: 'left',
    }),
  ];

  return (
    <>
      <TableContainer>
        <Scrollbar>
          <Table>
            <TableHeadCustom headCells={tableHead} />
            <TableBody>
              {state.announcements.items.map((row) => (
                <TableRow key={row.id}>
                  <TableCell>{row.title}</TableCell>
                  <TableCell>
                    <AnnouncementTypeLabel value={row.announcement_type} />
                  </TableCell>
                  <TableCell>
                    <EnabledLabel enabled={row.enabled} />
                  </TableCell>
                  <TableCell>{fDateTime(row.updated_at)}</TableCell>
                  <TableCell align="left" sx={tableStickyActionCellSx}>
                    <IconButton onClick={() => state.openEdit(row)}>
                      <Iconify icon="solar:pen-bold" />
                    </IconButton>
                    <IconButton color="error" onClick={() => state.setDeleting(row)}>
                      <Iconify icon="solar:trash-bin-trash-bold" />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </Scrollbar>
      </TableContainer>
      <TablePaginationCustom
        page={state.table.page}
        count={state.announcements.total}
        rowsPerPage={state.table.rowsPerPage}
        rowsPerPageOptions={[10, 25, 50]}
        onPageChange={state.table.onChangePage}
        onRowsPerPageChange={state.table.onChangeRowsPerPage}
      />
    </>
  );
}

function useAnnouncementManagementState() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10 });
  const [search, setSearch] = useState('');
  const announcements = useAnnouncements(table.page, table.rowsPerPage, { search }, true);
  const form = useAnnouncementFormState(t);
  const deletion = useAnnouncementDeleteState(t);
  const handleSearch = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      table.onResetPage();
      setSearch(event.target.value);
    },
    [table]
  );

  return {
    t,
    table,
    search,
    announcements,
    handleSearch,
    ...form,
    ...deletion,
  };
}

function useAnnouncementFormState(t: ReturnType<typeof useTranslate>['t']) {
  const [editing, setEditing] = useState<Announcement | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  const closeForm = useCallback(() => {
    setCreating(false);
    setEditing(null);
  }, []);

  const submitForm = useCallback(
    async (form: AnnouncementInput) => {
      setSubmitting(true);
      try {
        if (editing) {
          await updateAnnouncement(editing.id, form);
        } else {
          await createAnnouncement(form);
        }
        toast.success(t('operations.messages.saved'));
        closeForm();
      } catch (error) {
        toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
      } finally {
        setSubmitting(false);
      }
    },
    [closeForm, editing, t]
  );

  return {
    editing,
    submitting,
    formOpen: creating || !!editing,
    closeForm,
    submitForm,
    openCreate: () => setCreating(true),
    openEdit: (value: Announcement) => setEditing(value),
  };
}

function useAnnouncementDeleteState(t: ReturnType<typeof useTranslate>['t']) {
  const [deleting, setDeleting] = useState<Announcement | null>(null);

  const confirmDelete = useCallback(async () => {
    if (!deleting) return;
    try {
      await deleteAnnouncement(deleting.id);
      toast.success(t('operations.messages.deleted'));
      setDeleting(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleting, t]);

  return {
    deleting,
    setDeleting,
    confirmDelete,
  };
}

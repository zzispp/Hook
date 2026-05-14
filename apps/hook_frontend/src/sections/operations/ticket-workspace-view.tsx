'use client';

import type { SupportTicket } from 'src/types/operations';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';

import { fToNow } from 'src/utils/format-time';

import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { Iconify } from 'src/components/iconify';
import { TablePaginationCustom } from 'src/components/table';

import { AddButton, AdminBreadcrumbs } from 'src/sections/admin/shared';

import { TicketConversation } from './ticket-conversation';
import { TicketCreateDialog } from './ticket-create-dialog';
import { useTicketWorkspaceState } from './ticket-workspace-state';
import { TicketStatusLabel, TicketPriorityLabel } from './operation-labels';

const ticketPanelSx = {
  p: 2,
  width: { xs: 1, md: 360 },
  flexShrink: 0,
  display: 'flex',
  overflow: 'hidden',
  minHeight: { xs: 560, md: 0 },
};

const ticketListSx = {
  flex: '1 1 auto',
  minHeight: 0,
  overflowY: 'auto',
  overflowX: 'hidden',
  pr: 0.5,
};

const ticketPaginationSx = {
  flexShrink: 0,
  overflow: 'hidden',
  '& .MuiTablePagination-toolbar': {
    minHeight: 40,
    px: 0,
    gap: 0.5,
  },
  '& .MuiTablePagination-spacer': {
    display: 'none',
  },
  '& .MuiTablePagination-displayedRows': {
    m: 0,
    flex: '1 1 auto',
    fontSize: '0.75rem',
    whiteSpace: 'normal',
  },
  '& .MuiTablePagination-actions': {
    ml: 0,
    flexShrink: 0,
  },
};

export function TicketWorkspaceView({ admin = false }: { admin?: boolean }) {
  const state = useTicketWorkspaceState(admin);

  return (
    <DashboardContent
      maxWidth={false}
      sx={{ display: 'flex', flex: '1 1 auto', flexDirection: 'column' }}
    >
      <AdminBreadcrumbs
        headingCode={admin ? DASHBOARD_MENU_CODES.ticketManagement : DASHBOARD_MENU_CODES.tickets}
        action={
          admin ? null : (
            <AddButton onClick={() => state.setCreating(true)}>
              {state.t('operations.ticket.create')}
            </AddButton>
          )
        }
      />
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={3} sx={{ flex: '1 1 0', minHeight: 0 }}>
        <TicketListPanel state={state} />
        <Stack sx={{ flex: '1 1 auto', minWidth: 0, minHeight: 0 }}>
          <TicketConversation
            admin={admin}
            detail={state.detail.data}
            submitting={state.submitting}
            onReply={state.submitReply}
            onUpdate={admin ? state.submitUpdate : undefined}
          />
        </Stack>
      </Stack>
      <TicketCreateDialog
        open={state.creating}
        userEmail={state.userEmail}
        submitting={state.submitting}
        onClose={() => state.setCreating(false)}
        onSubmit={state.submitCreate}
      />
    </DashboardContent>
  );
}

function TicketListPanel({ state }: { state: ReturnType<typeof useTicketWorkspaceState> }) {
  return (
    <Card sx={ticketPanelSx}>
      <Stack spacing={2} sx={{ width: 1, minHeight: 0 }}>
        <TextField
          value={state.search}
          onChange={state.handleSearch}
          placeholder={state.t('operations.ticket.searchPlaceholder')}
        />
        <Stack spacing={1.5} sx={ticketListSx}>
          {state.tickets.items.map((ticket) => (
            <TicketListItem
              key={ticket.id}
              ticket={ticket}
              selected={ticket.id === state.selectedId}
              onClick={() => state.selectTicket(ticket.id)}
            />
          ))}
        </Stack>
        <TablePaginationCustom
          page={state.table.page}
          count={state.tickets.total}
          rowsPerPage={state.table.rowsPerPage}
          rowsPerPageOptions={[]}
          sx={ticketPaginationSx}
          labelDisplayedRows={({ from, to, count }) => `${from}-${to} / ${count}`}
          onPageChange={state.table.onChangePage}
        />
      </Stack>
    </Card>
  );
}

function TicketListItem({
  ticket,
  selected,
  onClick,
}: {
  ticket: SupportTicket;
  selected: boolean;
  onClick: () => void;
}) {
  return (
    <Button
      fullWidth
      color="inherit"
      onClick={onClick}
      sx={{
        p: 1.5,
        justifyContent: 'flex-start',
        bgcolor: selected ? 'action.selected' : 'transparent',
      }}
    >
      <Stack spacing={1} sx={{ width: 1, alignItems: 'flex-start' }}>
        <Typography variant="subtitle2" noWrap sx={{ maxWidth: 1 }}>
          {ticket.subject}
        </Typography>
        <Stack direction="row" spacing={1} sx={{ alignItems: 'center', flexWrap: 'wrap' }}>
          <TicketStatusLabel value={ticket.status} />
          <TicketPriorityLabel value={ticket.priority} />
        </Stack>
        <Typography variant="caption" color="text.disabled">
          <Iconify icon="solar:clock-circle-bold" width={14} /> {fToNow(ticket.last_message_at)}
        </Typography>
      </Stack>
    </Button>
  );
}

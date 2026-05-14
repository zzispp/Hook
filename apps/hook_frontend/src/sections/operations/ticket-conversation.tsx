'use client';

import type { TicketStatus, TicketPriority, SupportTicketDetail } from 'src/types/operations';

import { useState, useEffect } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';

import { fToNow } from 'src/utils/format-time';

import { useTranslate } from 'src/locales/use-locales';

import { Markdown } from 'src/components/markdown';

import { TicketStatusLabel, TicketPriorityLabel } from './operation-labels';

const STATUSES: TicketStatus[] = ['open', 'in_progress', 'waiting_user', 'resolved', 'closed'];
const PRIORITIES: TicketPriority[] = ['normal', 'high', 'urgent'];

const ticketConversationSx = {
  p: 3,
  display: 'flex',
  overflow: 'hidden',
  flex: '1 1 auto',
  minHeight: { xs: 560, md: 0 },
};

const ticketConversationBodySx = {
  width: 1,
  minHeight: 0,
};

const ticketMessagesSx = {
  flex: '1 1 auto',
  minHeight: 0,
  overflowY: 'auto',
  overflowX: 'hidden',
  pr: 1,
};

const ticketReplySx = {
  flexShrink: 0,
};

type Props = {
  admin: boolean;
  detail?: SupportTicketDetail;
  submitting: boolean;
  onReply: (message: string) => void;
  onUpdate?: (patch: { status?: TicketStatus; priority?: TicketPriority }) => void;
};

export function TicketConversation({ admin, detail, submitting, onReply, onUpdate }: Props) {
  const [message, setMessage] = useState('');
  const [status, setStatus] = useState<TicketStatus>('open');
  const [priority, setPriority] = useState<TicketPriority>('normal');

  useEffect(() => {
    if (detail?.ticket) {
      setStatus(detail.ticket.status);
      setPriority(detail.ticket.priority);
    }
  }, [detail]);

  if (!detail) {
    return <EmptyTicket />;
  }

  return (
    <Card sx={ticketConversationSx}>
      <Stack spacing={3} sx={ticketConversationBodySx}>
        <TicketHeader detail={detail} />
        {admin ? (
          <AdminTicketControls
            status={status}
            priority={priority}
            onStatus={setStatus}
            onPriority={setPriority}
            onSubmit={() => onUpdate?.({ status, priority })}
          />
        ) : null}
        <Stack spacing={2} sx={ticketMessagesSx}>
          {detail.messages.map((item) => (
            <MessageBubble
              key={item.id}
              mine={admin ? item.sender_role === 'admin' : item.sender_role === 'user'}
              detail={item}
            />
          ))}
        </Stack>
        <Stack spacing={1.5} sx={ticketReplySx}>
          <TicketReplyBox
            message={message}
            submitting={submitting}
            onChange={setMessage}
            onSubmit={() => {
              onReply(message);
              setMessage('');
            }}
          />
        </Stack>
      </Stack>
    </Card>
  );
}

function TicketReplyBox({
  message,
  submitting,
  onChange,
  onSubmit,
}: {
  message: string;
  submitting: boolean;
  onChange: (value: string) => void;
  onSubmit: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <>
      <TextField
        multiline
        minRows={4}
        value={message}
        label={t('operations.ticket.reply')}
        onChange={(event) => onChange(event.target.value)}
      />
      <Button
        variant="contained"
        loading={submitting}
        disabled={!message.trim()}
        onClick={onSubmit}
      >
        {t('operations.ticket.sendReply')}
      </Button>
    </>
  );
}

function TicketHeader({ detail }: { detail: SupportTicketDetail }) {
  return (
    <Stack spacing={1}>
      <Typography variant="h5">{detail.ticket.subject}</Typography>
      <Stack direction="row" spacing={1} sx={{ alignItems: 'center', flexWrap: 'wrap' }}>
        <TicketStatusLabel value={detail.ticket.status} />
        <TicketPriorityLabel value={detail.ticket.priority} />
        <Typography variant="caption" color="text.disabled">
          {detail.ticket.contact_email}
        </Typography>
      </Stack>
    </Stack>
  );
}

function AdminTicketControls({
  status,
  priority,
  onStatus,
  onPriority,
  onSubmit,
}: {
  status: TicketStatus;
  priority: TicketPriority;
  onStatus: (value: TicketStatus) => void;
  onPriority: (value: TicketPriority) => void;
  onSubmit: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}>
      <TextField
        select
        label={t('common.status')}
        value={status}
        onChange={(event) => onStatus(event.target.value as TicketStatus)}
      >
        {STATUSES.map((value) => (
          <MenuItem key={value} value={value}>
            {t(`operations.ticket.status.${value}`)}
          </MenuItem>
        ))}
      </TextField>
      <TextField
        select
        label={t('operations.ticket.priorityLabel')}
        value={priority}
        onChange={(event) => onPriority(event.target.value as TicketPriority)}
      >
        {PRIORITIES.map((value) => (
          <MenuItem key={value} value={value}>
            {t(`operations.ticket.priority.${value}`)}
          </MenuItem>
        ))}
      </TextField>
      <Button variant="outlined" onClick={onSubmit}>
        {t('operations.ticket.updateStatus')}
      </Button>
    </Stack>
  );
}

function MessageBubble({
  mine,
  detail,
}: {
  mine: boolean;
  detail: SupportTicketDetail['messages'][number];
}) {
  const { t } = useTranslate('admin');

  return (
    <Box sx={{ display: 'flex', justifyContent: mine ? 'flex-end' : 'flex-start' }}>
      <Box
        sx={{
          p: 2,
          maxWidth: 680,
          borderRadius: 2,
          bgcolor: mine ? 'primary.lighter' : 'background.neutral',
        }}
      >
        <Typography variant="caption" color="text.disabled">
          {t(`operations.ticket.sender.${detail.sender_role}`)} · {fToNow(detail.created_at)}
        </Typography>
        <Markdown sx={{ mt: 1, '& p': { typography: 'body2' } }}>{detail.body_markdown}</Markdown>
      </Box>
    </Box>
  );
}

function EmptyTicket() {
  const { t } = useTranslate('admin');
  return (
    <Card sx={{ ...ticketConversationSx, p: 4, display: 'grid', placeItems: 'center' }}>
      <Typography color="text.secondary">{t('operations.ticket.selectEmpty')}</Typography>
    </Card>
  );
}

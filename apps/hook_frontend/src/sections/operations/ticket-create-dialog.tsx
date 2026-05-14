'use client';

import { useState, useEffect } from 'react';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import TextField from '@mui/material/TextField';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { useTranslate } from 'src/locales/use-locales';

type Props = {
  open: boolean;
  userEmail: string;
  submitting: boolean;
  onClose: () => void;
  onSubmit: (form: { subject: string; body_markdown: string; contact_email?: string }) => void;
};

export function TicketCreateDialog({ open, userEmail, submitting, onClose, onSubmit }: Props) {
  const { t } = useTranslate('admin');
  const [editableEmail, setEditableEmail] = useState(false);
  const [form, setForm] = useState({ subject: '', body_markdown: '', contact_email: userEmail });

  useEffect(() => {
    if (open) {
      setEditableEmail(false);
      setForm({ subject: '', body_markdown: '', contact_email: userEmail });
    }
  }, [open, userEmail]);

  return (
    <Dialog fullWidth maxWidth="sm" open={open} onClose={onClose}>
      <DialogTitle>{t('operations.ticket.createTitle')}</DialogTitle>
      <DialogContent>
        <Stack spacing={2.5} sx={{ pt: 1 }}>
          <TextField
            required
            label={t('operations.ticket.subject')}
            value={form.subject}
            onChange={(event) => setForm((current) => ({ ...current, subject: event.target.value }))}
          />
          <Stack direction="row" spacing={1}>
            <TextField
              fullWidth
              disabled={!editableEmail}
              label={t('common.email')}
              value={form.contact_email}
              onChange={(event) =>
                setForm((current) => ({ ...current, contact_email: event.target.value }))
              }
            />
            <Button variant="outlined" onClick={() => setEditableEmail(true)}>
              {t('common.edit')}
            </Button>
          </Stack>
          <TextField
            multiline
            required
            minRows={7}
            label={t('operations.ticket.message')}
            value={form.body_markdown}
            onChange={(event) =>
              setForm((current) => ({ ...current, body_markdown: event.target.value }))
            }
          />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>{t('common.cancel')}</Button>
        <Button variant="contained" loading={submitting} onClick={() => onSubmit(form)}>
          {t('common.create')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

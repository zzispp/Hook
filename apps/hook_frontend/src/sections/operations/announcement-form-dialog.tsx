'use client';

import type { Announcement, AnnouncementType, AnnouncementInput } from 'src/types/operations';

import { useState, useEffect } from 'react';

import Stack from '@mui/material/Stack';
import Switch from '@mui/material/Switch';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import FormControlLabel from '@mui/material/FormControlLabel';

import { useTranslate } from 'src/locales/use-locales';

const ANNOUNCEMENT_TYPES: AnnouncementType[] = ['info', 'warning', 'maintenance', 'important'];

const DEFAULT_FORM: AnnouncementInput = {
  title: '',
  content_markdown: '',
  announcement_type: 'info',
  pinned: false,
  enabled: true,
};

type Props = {
  open: boolean;
  editing: Announcement | null;
  submitting: boolean;
  onClose: () => void;
  onSubmit: (form: AnnouncementInput) => void;
};

export function AnnouncementFormDialog({ open, editing, submitting, onClose, onSubmit }: Props) {
  const { t } = useTranslate('admin');
  const [form, setForm] = useState(DEFAULT_FORM);

  useEffect(() => {
    setForm(editing ? formFromAnnouncement(editing) : DEFAULT_FORM);
  }, [editing, open]);

  return (
    <Dialog fullWidth maxWidth="md" open={open} onClose={onClose}>
      <DialogTitle>
        {editing ? t('operations.announcement.editTitle') : t('operations.announcement.createTitle')}
      </DialogTitle>
      <DialogContent>
        <Stack spacing={2.5} sx={{ pt: 1 }}>
          <TextField
            required
            label={t('common.title')}
            value={form.title}
            onChange={(event) => setForm((current) => ({ ...current, title: event.target.value }))}
          />
          <TextField
            select
            label={t('common.type')}
            value={form.announcement_type}
            onChange={(event) =>
              setForm((current) => ({
                ...current,
                announcement_type: event.target.value as AnnouncementType,
              }))
            }
          >
            {ANNOUNCEMENT_TYPES.map((type) => (
              <MenuItem key={type} value={type}>
                {t(`operations.announcement.types.${type}`)}
              </MenuItem>
            ))}
          </TextField>
          <TextField
            multiline
            required
            minRows={8}
            label={t('operations.announcement.contentMarkdown')}
            value={form.content_markdown}
            onChange={(event) =>
              setForm((current) => ({ ...current, content_markdown: event.target.value }))
            }
          />
          <FormControlLabel
            control={
              <Switch
                checked={form.pinned}
                onChange={(event) =>
                  setForm((current) => ({ ...current, pinned: event.target.checked }))
                }
              />
            }
            label={t('operations.announcement.pinned')}
          />
          <FormControlLabel
            control={
              <Switch
                checked={form.enabled}
                onChange={(event) =>
                  setForm((current) => ({ ...current, enabled: event.target.checked }))
                }
              />
            }
            label={t('common.enabled')}
          />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>{t('common.cancel')}</Button>
        <Button variant="contained" loading={submitting} onClick={() => onSubmit(form)}>
          {t('common.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function formFromAnnouncement(value: Announcement): AnnouncementInput {
  return {
    title: value.title,
    content_markdown: value.content_markdown,
    announcement_type: value.announcement_type,
    pinned: value.pinned,
    enabled: value.enabled,
  };
}

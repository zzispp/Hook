'use client';

import type { TFunction } from 'i18next';

import { useState, useEffect } from 'react';

import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import TextField from '@mui/material/TextField';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { useTranslate } from 'src/locales/use-locales';

import { AuthCaptcha } from 'src/auth/components/cap-widget';

type Props = {
  open: boolean;
  userEmail: string;
  captchaConfig: TicketCaptchaConfig;
  submitting: boolean;
  onClose: () => void;
  onSubmit: (form: {
    subject: string;
    body_markdown: string;
    contact_email?: string;
    captcha_token?: string;
  }) => void;
};

type TicketCaptchaConfig = {
  enabled?: boolean;
  loading: boolean;
  errorMessage?: string;
};

export function TicketCreateDialog({
  open,
  userEmail,
  captchaConfig,
  submitting,
  onClose,
  onSubmit,
}: Props) {
  const { t } = useTranslate('admin');
  const [editableEmail, setEditableEmail] = useState(false);
  const [captchaToken, setCaptchaToken] = useState<string | null>(null);
  const [captchaResetKey, setCaptchaResetKey] = useState(0);
  const [form, setForm] = useState({ subject: '', body_markdown: '', contact_email: userEmail });

  useEffect(() => {
    if (open) {
      setEditableEmail(false);
      setCaptchaToken(null);
      setCaptchaResetKey((value) => value + 1);
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
          <AuthCaptcha
            enabled={captchaConfig.enabled === true}
            resetKey={captchaResetKey}
            labels={adminCaptchaLabels(t)}
            onTokenChange={setCaptchaToken}
          />
          {captchaConfig.errorMessage ? (
            <Alert severity="error">{captchaConfig.errorMessage}</Alert>
          ) : null}
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>{t('common.cancel')}</Button>
        <Button
          variant="contained"
          loading={submitting || captchaConfig.loading}
          disabled={captchaDisabled(captchaConfig, captchaToken)}
          onClick={() => onSubmit(ticketPayload(form, captchaConfig, captchaToken))}
        >
          {t('common.create')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function captchaDisabled(config: TicketCaptchaConfig, token: string | null) {
  if (config.loading || config.errorMessage || config.enabled === undefined) {
    return true;
  }
  return config.enabled && !token;
}

function ticketPayload(
  form: { subject: string; body_markdown: string; contact_email?: string },
  config: TicketCaptchaConfig,
  captchaToken: string | null
) {
  return {
    ...form,
    ...(config.enabled && captchaToken ? { captcha_token: captchaToken } : {}),
  };
}

function adminCaptchaLabels(t: TFunction<'admin'>) {
  return {
    initial: t('captcha.initial'),
    verifying: t('captcha.verifying'),
    solved: t('captcha.solved'),
    error: t('captcha.error'),
  };
}

'use client';

import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import TextField from '@mui/material/TextField';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { copyText } from 'src/utils/browser-compat';

import { useTranslate } from 'src/locales/use-locales';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

export function ApiTokenCreatedDialog({
  rawToken,
  onClose,
}: {
  rawToken: string | null;
  onClose: VoidFunction;
}) {
  const { t } = useTranslate('admin');

  return (
    <Dialog fullWidth maxWidth="sm" open={!!rawToken} onClose={onClose}>
      <DialogTitle>{t('dialogs.apiTokenCreated')}</DialogTitle>
      <DialogContent>
        <Alert severity="success" sx={{ mb: 2 }}>{t('messages.apiTokenCreated')}</Alert>
        <TextField fullWidth value={rawToken ?? ''} InputProps={{ readOnly: true }} />
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onClose}>{t('common.close')}</Button>
        <Button
          variant="contained"
          startIcon={<Iconify icon="solar:copy-bold" />}
          onClick={() => void copyToken(rawToken, t)}
        >
          {t('actions.copyApiKey')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

async function copyToken(rawToken: string | null, t: (key: string) => string) {
  if (!rawToken) return;
  try {
    await copyText(rawToken);
    toast.success(t('messages.apiKeyCopied'));
  } catch (error) {
    toast.error(error instanceof Error ? error.message : t('messages.copyFailed'));
  }
}

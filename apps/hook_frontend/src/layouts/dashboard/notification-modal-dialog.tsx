'use client';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import Dialog from '@mui/material/Dialog';
import IconButton from '@mui/material/IconButton';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';
import CircularProgress from '@mui/material/CircularProgress';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

import { QuickImportEventContent } from './quick-import-event-content';
import { type NotificationModalDialogProps } from './notification-modal-types';

const DIALOG_CONTENT_SX = {
  px: 3,
  pb: 3,
  pt: 2,
  maxHeight: 'min(72vh, 760px)',
  overflowY: 'auto',
};

export function NotificationModalDialog({
  state,
  announcementContent,
  quickImportEvent,
  loading,
  error,
  onClose,
}: NotificationModalDialogProps) {
  const { t } = useTranslate('admin');
  const title =
    state.kind === 'quickImport'
      ? t('providers.quickImportResolutionTitle')
      : t('operations.notifications.categories.announcement');

  return (
    <Dialog
      fullWidth
      maxWidth="md"
      open={state.kind !== 'closed'}
      onClose={() => void onClose()}
    >
      <DialogTitle sx={{ display: 'flex', alignItems: 'center', pr: 6 }}>
        <Box sx={{ flexGrow: 1 }}>{title}</Box>
        <IconButton
          aria-label={t('common.close')}
          onClick={() => void onClose()}
          sx={{ position: 'absolute', right: 16, top: 12 }}
        >
          <Iconify icon="mingcute:close-line" />
        </IconButton>
      </DialogTitle>
      <DialogContent dividers sx={DIALOG_CONTENT_SX}>
        {loading ? (
          <Box sx={{ py: 8, display: 'flex', justifyContent: 'center' }}>
            <CircularProgress />
          </Box>
        ) : null}
        {!loading && error ? <Alert severity="error">{error.message}</Alert> : null}
        {!loading && !error && state.kind === 'announcement' ? announcementContent : null}
        {!loading && !error && state.kind === 'quickImport' && quickImportEvent ? (
          <QuickImportEventContent event={quickImportEvent} />
        ) : null}
      </DialogContent>
    </Dialog>
  );
}

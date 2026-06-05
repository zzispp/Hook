'use client';

import type { TFunction } from 'i18next';
import type { RelationDialogState } from './admin-affiliate-state';

import { useState, useEffect } from 'react';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import TextField from '@mui/material/TextField';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { AffiliateReferrerSelector } from './admin-affiliate-referrer-selector';

export function AffiliateRelationDialog({
  t,
  state,
  submitting,
  onChange,
  onClose,
  onSubmit,
}: {
  t: TFunction<'admin'>;
  state: RelationDialogState | null;
  submitting: boolean;
  onChange: (patch: Partial<Pick<RelationDialogState, 'referrerAffCode' | 'reason'>>) => void;
  onClose: VoidFunction;
  onSubmit: VoidFunction;
}) {
  const open = Boolean(state);
  const [renderState, setRenderState] = useState<RelationDialogState | null>(state);

  useEffect(() => {
    if (state) {
      setRenderState(state);
    }
  }, [state]);

  const displayState = state ?? renderState;
  const clearMode = displayState?.mode === 'clear';

  return (
    <Dialog fullWidth maxWidth="sm" open={open} onClose={onClose} TransitionProps={{ onExited: () => setRenderState(null) }}>
      <DialogTitle>
        {clearMode ? t('adminAffiliates.dialogs.clearTitle') : t('adminAffiliates.dialogs.rebindTitle')}
      </DialogTitle>
      <DialogContent>
        <Stack spacing={2.5} sx={{ pt: 1 }}>
          {displayState && !clearMode ? (
            <AffiliateReferrerSelector
              key={displayState.relation.user.id}
              t={t}
              excludedUserId={displayState.relation.user.id}
              value={displayState.referrerAffCode}
              active={state?.mode === 'rebind'}
              onChange={(referrerAffCode) => onChange({ referrerAffCode })}
            />
          ) : null}
          <TextField
            required
            fullWidth
            multiline
            minRows={3}
            label={t('adminAffiliates.fields.reason')}
            value={displayState?.reason ?? ''}
            onChange={(event) => onChange({ reason: event.target.value })}
          />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button color="inherit" onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button loading={submitting} variant="contained" color={clearMode ? 'warning' : 'primary'} onClick={onSubmit}>
          {clearMode ? t('adminAffiliates.dialogs.clearConfirm') : t('adminAffiliates.dialogs.rebindConfirm')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

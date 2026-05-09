'use client';

import type { Role } from 'src/types/rbac';

import Button from '@mui/material/Button';

import { useTranslate } from 'src/locales/use-locales';

import { ConfirmDialog } from 'src/components/custom-dialog';

export function RoleDeleteDialog({
  target,
  onClose,
  onConfirm,
}: {
  target: Role | null;
  onClose: () => void;
  onConfirm: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <ConfirmDialog
      open={!!target}
      onClose={onClose}
      title={t('dialogs.deleteRole')}
      content={t('dialogs.deleteContent', { name: target?.name ?? '' })}
      cancelText={t('common.cancel')}
      action={
        <Button variant="contained" color="error" onClick={onConfirm}>
          {t('common.delete')}
        </Button>
      }
    />
  );
}

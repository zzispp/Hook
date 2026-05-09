'use client';

import type { MenuSection, MenuItem as RbacMenuItem } from 'src/types/rbac';

import Button from '@mui/material/Button';

import { useTranslate } from 'src/locales/use-locales';

import { ConfirmDialog } from 'src/components/custom-dialog';

export function DeleteSectionDialog({
  target,
  onClose,
  onConfirm,
}: {
  target: MenuSection | null;
  onClose: () => void;
  onConfirm: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <ConfirmDialog
      open={!!target}
      onClose={onClose}
      title={t('dialogs.deleteMenuSection')}
      content={t('dialogs.deleteContent', { name: target?.subheader ?? '' })}
      cancelText={t('common.cancel')}
      action={
        <Button variant="contained" color="error" onClick={onConfirm}>
          {t('common.delete')}
        </Button>
      }
    />
  );
}

export function DeleteItemDialog({
  target,
  onClose,
  onConfirm,
}: {
  target: RbacMenuItem | null;
  onClose: () => void;
  onConfirm: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <ConfirmDialog
      open={!!target}
      onClose={onClose}
      title={t('dialogs.deleteMenuItem')}
      content={t('dialogs.deleteContent', { name: target?.title ?? '' })}
      cancelText={t('common.cancel')}
      action={
        <Button variant="contained" color="error" onClick={onConfirm}>
          {t('common.delete')}
        </Button>
      }
    />
  );
}

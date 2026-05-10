'use client';

import type { TranslationLanguage } from 'src/types/i18n';
import type { TranslationValueRow } from './translation-management-utils';

import Button from '@mui/material/Button';

import { useTranslate } from 'src/locales/use-locales';

import { ConfirmDialog } from 'src/components/custom-dialog';

export function DeleteTranslationValueDialog({
  target,
  onClose,
  onConfirm,
}: {
  target: TranslationValueRow | null;
  onClose: () => void;
  onConfirm: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <ConfirmDialog
      open={!!target}
      onClose={onClose}
      title={t('translations.dialogs.deleteValue')}
      content={t('dialogs.deleteContent', { name: target ? `${target.group_key}.${target.item_key}` : '' })}
      cancelText={t('common.cancel')}
      action={
        <Button variant="contained" color="error" onClick={onConfirm}>
          {t('common.delete')}
        </Button>
      }
    />
  );
}

export function DeleteTranslationLanguageDialog({
  target,
  onClose,
  onConfirm,
}: {
  target: TranslationLanguage | null;
  onClose: () => void;
  onConfirm: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <ConfirmDialog
      open={!!target}
      onClose={onClose}
      title={t('translations.dialogs.deleteLanguage')}
      content={t('dialogs.deleteContent', { name: target?.native_name ?? '' })}
      cancelText={t('common.cancel')}
      action={
        <Button variant="contained" color="error" onClick={onConfirm}>
          {t('common.delete')}
        </Button>
      }
    />
  );
}


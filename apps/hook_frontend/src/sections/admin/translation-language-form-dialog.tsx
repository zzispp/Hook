'use client';

import type { TranslationLanguage } from 'src/types/i18n';
import type { TranslationLanguageForm } from './translation-management-utils';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow, ManagementDialog } from './shared';

type Props = {
  editing: TranslationLanguage | null;
  form: TranslationLanguageForm;
  open: boolean;
  submitting: boolean;
  onClose: () => void;
  onFormChange: (value: TranslationLanguageForm) => void;
  onSubmit: () => void;
};

export function TranslationLanguageFormDialog({
  editing,
  form,
  open,
  submitting,
  onClose,
  onFormChange,
  onSubmit,
}: Props) {
  const { t } = useTranslate('admin');
  const title = editing ? t('translations.dialogs.editLanguage') : t('translations.dialogs.createLanguage');

  return (
    <ManagementDialog open={open} title={title} submitting={submitting} onClose={onClose} onSubmit={onSubmit}>
      <TextFieldRow
        required
        disabled={!!editing}
        label={t('translations.fields.langCode')}
        value={form.code}
        onChange={(value) => onFormChange({ ...form, code: value })}
      />
      <TextFieldRow
        required
        label={t('common.name')}
        value={form.name}
        onChange={(value) => onFormChange({ ...form, name: value })}
      />
      <TextFieldRow
        required
        label={t('translations.fields.nativeName')}
        value={form.native_name}
        onChange={(value) => onFormChange({ ...form, native_name: value })}
      />
      <TextFieldRow
        type="number"
        label={t('common.sortOrder')}
        value={form.sort_order}
        onChange={(value) => onFormChange({ ...form, sort_order: Number(value) })}
      />
      <SwitchRow
        label={t('common.enabled')}
        checked={form.enabled}
        onChange={(enabled) => onFormChange({ ...form, enabled })}
      />
    </ManagementDialog>
  );
}


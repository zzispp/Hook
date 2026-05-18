'use client';

import type { TranslationLanguage } from 'src/types/i18n';
import type { TranslationValueRow, TranslationValueForm } from './translation-management-utils';

import { useTranslate } from 'src/locales/use-locales';

import { TRANSLATION_NAMESPACES } from './translation-management-utils';
import { SwitchRow, SelectOption, TextFieldRow, ManagementDialog } from './shared';

type Props = {
  editing: TranslationValueRow | null;
  form: TranslationValueForm;
  languages: TranslationLanguage[];
  open: boolean;
  submitting: boolean;
  onClose: () => void;
  onFormChange: (value: TranslationValueForm) => void;
  onSubmit: () => void;
};

export function TranslationValueFormDialog({
  editing,
  form,
  languages,
  open,
  submitting,
  onClose,
  onFormChange,
  onSubmit,
}: Props) {
  const { t } = useTranslate('admin');
  const title = editing ? t('translations.dialogs.editValue') : t('translations.dialogs.createValue');

  return (
    <ManagementDialog open={open} title={title} submitting={submitting} onClose={onClose} onSubmit={onSubmit}>
      <TextFieldRow
        select
        required
        disabled={!!editing}
        label={t('translations.fields.namespace')}
        value={form.namespace}
        onChange={(value) => onFormChange({ ...form, namespace: value })}
      >
        {TRANSLATION_NAMESPACES.map((value) => (
          <SelectOption key={value} value={value} label={t(`translations.namespaces.${value}`)} />
        ))}
      </TextFieldRow>
      <TextFieldRow
        required
        disabled={!!editing}
        label={t('translations.fields.groupKey')}
        value={form.group_key}
        onChange={(value) => onFormChange({ ...form, group_key: value })}
      />
      <TextFieldRow
        required
        disabled={!!editing}
        label={t('translations.fields.itemKey')}
        value={form.item_key}
        onChange={(value) => onFormChange({ ...form, item_key: value })}
      />
      {languages.map((language) => (
        <TextFieldRow
          key={language.code}
          required
          label={`${language.native_name} (${language.code})`}
          value={form.values[language.code] ?? ''}
          onChange={(value) =>
            onFormChange({ ...form, values: { ...form.values, [language.code]: value } })
          }
        />
      ))}
      <SwitchRow
        label={t('common.enabled')}
        checked={form.enabled}
        onChange={(enabled) => onFormChange({ ...form, enabled })}
      />
    </ManagementDialog>
  );
}

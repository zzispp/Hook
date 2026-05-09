'use client';

import type { MenuSection, MenuSectionInput } from 'src/types/rbac';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow, ManagementDialog } from './shared';

type Props = {
  editing: MenuSection | null;
  form: MenuSectionInput;
  open: boolean;
  submitting: boolean;
  onClose: () => void;
  onFormChange: (value: MenuSectionInput) => void;
  onSubmit: () => void;
};

export const DEFAULT_MENU_SECTION_FORM: MenuSectionInput = {
  code: '',
  subheader: '',
  sort_order: 0,
  enabled: true,
};

export function MenuSectionFormDialog({
  editing,
  form,
  open,
  submitting,
  onClose,
  onFormChange,
  onSubmit,
}: Props) {
  const { t } = useTranslate('admin');
  const title = editing ? t('dialogs.editMenuSection') : t('dialogs.createMenuSection');

  return (
    <ManagementDialog open={open} title={title} submitting={submitting} onClose={onClose} onSubmit={onSubmit}>
      <TextFieldRow
        required
        label={t('fields.subheader')}
        value={form.subheader}
        onChange={(value) => onFormChange({ ...form, subheader: value })}
      />
      <TextFieldRow
        required
        label={t('common.code')}
        value={form.code}
        onChange={(value) => onFormChange({ ...form, code: value })}
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

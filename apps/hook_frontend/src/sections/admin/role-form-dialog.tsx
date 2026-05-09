'use client';

import type { Role, RoleInput } from 'src/types/rbac';

import { useTranslate } from 'src/locales/use-locales';

import {
  SwitchRow,
  TextFieldRow,
  ManagementDialog,
} from './shared';

type Props = {
  editing: Role | null;
  form: RoleInput;
  open: boolean;
  submitting: boolean;
  title: string;
  onClose: () => void;
  onFormChange: (value: RoleInput) => void;
  onSubmit: () => void;
};

export function RoleFormDialog({
  editing,
  form,
  open,
  submitting,
  title,
  onClose,
  onFormChange,
  onSubmit,
}: Props) {
  const { t } = useTranslate('admin');

  return (
    <ManagementDialog open={open} title={title} submitting={submitting} onClose={onClose} onSubmit={onSubmit}>
      <TextFieldRow
        required
        disabled={!!editing}
        label={t('common.code')}
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
        label={t('common.description')}
        value={form.description}
        onChange={(value) => onFormChange({ ...form, description: value })}
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

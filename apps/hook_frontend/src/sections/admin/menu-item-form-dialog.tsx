'use client';

import type { MenuSection, MenuItemInput, MenuItem as RbacMenuItem } from 'src/types/rbac';

import Box from '@mui/material/Box';
import MenuItem from '@mui/material/MenuItem';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';

import {
  SwitchRow,
  TextFieldRow,
  NAV_ICON_OPTIONS,
  ManagementDialog,
  translatedMenuItem,
  translatedMenuSection,
} from './shared';

type Props = {
  allItems: RbacMenuItem[];
  allSections: MenuSection[];
  editing: RbacMenuItem | null;
  form: MenuItemInput;
  open: boolean;
  submitting: boolean;
  onClose: () => void;
  onFormChange: (value: MenuItemInput) => void;
  onSubmit: () => void;
};

export const DEFAULT_MENU_ITEM_FORM: MenuItemInput = {
  section_id: '',
  parent_id: null,
  code: '',
  title: '',
  path: '',
  icon: 'icon.menu',
  caption: null,
  deep_match: true,
  sort_order: 0,
  enabled: true,
};

export function MenuItemFormDialog({
  allItems,
  allSections,
  editing,
  form,
  open,
  submitting,
  onClose,
  onFormChange,
  onSubmit,
}: Props) {
  const { t } = useTranslate('admin');
  const title = editing ? t('dialogs.editMenuItem') : t('dialogs.createMenuItem');

  return (
    <ManagementDialog open={open} title={title} submitting={submitting} onClose={onClose} onSubmit={onSubmit}>
      <TextFieldRow
        required
        select
        label={t('common.section')}
        value={form.section_id}
        onChange={(value) => onFormChange({ ...form, section_id: value })}
      >
        {allSections.map((section) => (
          <MenuItem key={section.id} value={section.id}>
            {translatedMenuSection(section, t)}
          </MenuItem>
        ))}
      </TextFieldRow>
      <TextFieldRow
        select
        label={t('fields.parentItem')}
        value={form.parent_id ?? ''}
        onChange={(value) => onFormChange({ ...form, parent_id: value || null })}
      >
        <MenuItem value="">{t('common.none')}</MenuItem>
        {allItems
          .filter((item) => item.id !== editing?.id)
          .map((item) => (
            <MenuItem key={item.id} value={item.id}>
              {translatedMenuItem(item, t)}
            </MenuItem>
          ))}
      </TextFieldRow>
      <MenuItemTextFields form={form} onFormChange={onFormChange} />
      <Box>
        <Label color="info" variant="soft" sx={{ mr: 1 }}>
          {t('helper.deepMatch')}
        </Label>
      </Box>
      <SwitchRow
        label={t('fields.deepMatch')}
        checked={form.deep_match}
        onChange={(deepMatch) => onFormChange({ ...form, deep_match: deepMatch })}
      />
      <SwitchRow
        label={t('common.enabled')}
        checked={form.enabled}
        onChange={(enabled) => onFormChange({ ...form, enabled })}
      />
    </ManagementDialog>
  );
}

function MenuItemTextFields({
  form,
  onFormChange,
}: {
  form: MenuItemInput;
  onFormChange: (value: MenuItemInput) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <>
      <TextFieldRow
        required
        label={t('common.title')}
        value={form.title}
        onChange={(value) => onFormChange({ ...form, title: value })}
      />
      <TextFieldRow
        required
        label={t('common.code')}
        value={form.code}
        onChange={(value) => onFormChange({ ...form, code: value })}
      />
      <TextFieldRow
        required
        label={t('common.path')}
        value={form.path}
        onChange={(value) => onFormChange({ ...form, path: value })}
      />
      <IconField form={form} onFormChange={onFormChange} />
      <TextFieldRow
        label={t('common.caption')}
        value={form.caption ?? ''}
        onChange={(value) => onFormChange({ ...form, caption: value || null })}
      />
      <TextFieldRow
        type="number"
        label={t('common.sortOrder')}
        value={form.sort_order}
        onChange={(value) => onFormChange({ ...form, sort_order: Number(value) })}
      />
    </>
  );
}

function IconField({
  form,
  onFormChange,
}: {
  form: MenuItemInput;
  onFormChange: (value: MenuItemInput) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TextFieldRow
      select
      label={t('common.icon')}
      value={form.icon ?? ''}
      onChange={(value) => onFormChange({ ...form, icon: value || null })}
    >
      <MenuItem value="">{t('common.none')}</MenuItem>
      {NAV_ICON_OPTIONS.map((option) => (
        <MenuItem key={option} value={option}>
          {option}
        </MenuItem>
      ))}
    </TextFieldRow>
  );
}

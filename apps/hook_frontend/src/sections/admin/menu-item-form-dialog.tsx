'use client';

import type { IconifyName } from 'src/components/iconify';
import type { MenuSection, MenuItemInput, MenuItem as RbacMenuItem } from 'src/types/rbac';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import Autocomplete from '@mui/material/Autocomplete';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';

import {
  SwitchRow,
  TextFieldRow,
  NAV_ICON_OPTIONS,
  ManagementDialog,
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
  icon: 'solar:list-bold',
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
  const parentItems = allItems.filter(
    (item) => item.id !== editing?.id && item.section_id === form.section_id
  );

  return (
    <ManagementDialog open={open} title={title} submitting={submitting} onClose={onClose} onSubmit={onSubmit}>
      <TextFieldRow
        required
        select
        label={t('common.section')}
        value={form.section_id}
        onChange={(value) => onFormChange({ ...form, section_id: value, parent_id: null })}
      >
        {allSections.map((section) => (
          <MenuItem key={section.id} value={section.id}>
            {section.subheader}
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
        {parentItems.map((item) => (
          <MenuItem key={item.id} value={item.id}>
            {item.title}
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
    <Autocomplete
      fullWidth
      options={NAV_ICON_OPTIONS}
      value={form.icon ?? null}
      onChange={(_event, value) => onFormChange({ ...form, icon: value || null })}
      renderOption={(props, option) => (
        <MenuItem {...props} key={option} value={option}>
          <Stack direction="row" spacing={1.25} alignItems="center">
            <Iconify icon={option as IconifyName} />
            <Typography variant="body2">{option}</Typography>
          </Stack>
        </MenuItem>
      )}
      renderInput={(params) => (
        <TextField
          {...params}
          label={t('common.icon')}
          slotProps={{
            input: {
              ...params.InputProps,
              startAdornment: form.icon ? (
                <Iconify icon={form.icon as IconifyName} sx={{ mr: 1, color: 'text.secondary' }} />
              ) : (
                params.InputProps.startAdornment
              ),
            },
          }}
        />
      )}
    />
  );
}

'use client';

import type { MenuItem as RbacMenuItem } from 'src/types/rbac';

import Checkbox from '@mui/material/Checkbox';
import MenuItem from '@mui/material/MenuItem';
import ListItemText from '@mui/material/ListItemText';

import { useTranslate } from 'src/locales/use-locales';

import { TextFieldRow, translatedMenuItem } from './shared';

type Props = {
  menus: RbacMenuItem[];
  value: string[];
  onChange: (value: string[]) => void;
};

export function ApiMenuSelect({ menus, value, onChange }: Props) {
  const { t } = useTranslate('admin');

  return (
    <TextFieldRow
      select
      label={t('common.menus')}
      value={value}
      onChange={(selected) => onChange(selected.split(',').filter(Boolean))}
      SelectProps={{
        multiple: true,
        renderValue: (selected) => selectedMenuLabels(menus, selected as string[], t),
      }}
    >
      {menus.map((menu) => (
        <MenuItem key={menu.id} value={menu.id}>
          <Checkbox checked={value.includes(menu.id)} />
          <ListItemText primary={translatedMenuItem(menu, t)} secondary={menu.path} />
        </MenuItem>
      ))}
    </TextFieldRow>
  );
}

function selectedMenuLabels(
  menus: RbacMenuItem[],
  selected: string[],
  t: ReturnType<typeof useTranslate>['t']
) {
  return selected
    .map((id) => menus.find((menu) => menu.id === id))
    .filter((menu): menu is RbacMenuItem => Boolean(menu))
    .map((menu) => translatedMenuItem(menu, t))
    .join(', ');
}

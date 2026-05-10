'use client';

import type { MenuItem as RbacMenuItem } from 'src/types/rbac';

import { useMemo } from 'react';

import Box from '@mui/material/Box';
import MenuItem from '@mui/material/MenuItem';
import Checkbox from '@mui/material/Checkbox';
import ListItemText from '@mui/material/ListItemText';

import { useTranslate } from 'src/locales/use-locales';

import { TextFieldRow } from './shared';

type Props = {
  menus: RbacMenuItem[];
  value: string[];
  onChange: (value: string[]) => void;
};

type MenuTreeOption = {
  menu: RbacMenuItem;
  depth: number;
};

const MENU_INDENT_SPACING = 2;

export function ApiMenuSelect({ menus, value, onChange }: Props) {
  const { t } = useTranslate('admin');
  const options = useMemo(() => menuTreeOptions(menus), [menus]);

  return (
    <TextFieldRow
      select
      label={t('common.menus')}
      value={value}
      onChange={(selected) => onChange(selected.split(',').filter(Boolean))}
      SelectProps={{
        multiple: true,
        renderValue: (selected) => selectedMenuLabels(menus, selected as string[]),
      }}
    >
      {options.map(({ menu, depth }) => (
        <MenuItem key={menu.id} value={menu.id}>
          <Box sx={{ display: 'flex', alignItems: 'center', pl: depth * MENU_INDENT_SPACING, width: 1 }}>
            <Checkbox checked={value.includes(menu.id)} />
            <ListItemText primary={menu.title} secondary={menu.path} />
          </Box>
        </MenuItem>
      ))}
    </TextFieldRow>
  );
}

function selectedMenuLabels(menus: RbacMenuItem[], selected: string[]) {
  return selected
    .map((id) => menus.find((menu) => menu.id === id))
    .filter((menu): menu is RbacMenuItem => Boolean(menu))
    .map((menu) => menu.title)
    .join(', ');
}

function menuTreeOptions(menus: RbacMenuItem[]): MenuTreeOption[] {
  const childrenByParent = new Map<string, RbacMenuItem[]>();
  const menuIds = new Set(menus.map((menu) => menu.id));

  for (const menu of sortedMenus(menus)) {
    const parentId = menu.parent_id && menuIds.has(menu.parent_id) ? menu.parent_id : '';
    childrenByParent.set(parentId, [...(childrenByParent.get(parentId) ?? []), menu]);
  }

  return flattenMenuTree(childrenByParent, '', 0);
}

function flattenMenuTree(
  childrenByParent: Map<string, RbacMenuItem[]>,
  parentId: string,
  depth: number
): MenuTreeOption[] {
  return (childrenByParent.get(parentId) ?? []).flatMap((menu) => [
    { menu, depth },
    ...flattenMenuTree(childrenByParent, menu.id, depth + 1),
  ]);
}

function sortedMenus(menus: RbacMenuItem[]) {
  return [...menus].sort((left, right) => left.sort_order - right.sort_order || left.title.localeCompare(right.title));
}

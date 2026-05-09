'use client';

import type {
  MenuSection,
  MenuItemInput,
  MenuSectionInput,
  MenuItem as RbacMenuItem,
} from 'src/types/rbac';

import { useState, useCallback } from 'react';

import { useTranslate } from 'src/locales/use-locales';
import {
  getMenuApis,
  createMenuItem,
  updateMenuItem,
  createMenuSection,
  updateMenuSection,
} from 'src/actions/rbac';

import { toast } from 'src/components/snackbar';

import { DEFAULT_MENU_ITEM_FORM } from './menu-item-form-dialog';
import { DEFAULT_MENU_SECTION_FORM } from './menu-section-form-dialog';

export function useSectionFormState() {
  const [form, setForm] = useState<MenuSectionInput>(DEFAULT_MENU_SECTION_FORM);
  const [editing, setEditing] = useState<MenuSection | null>(null);
  const [creating, setCreating] = useState(false);

  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm({ ...DEFAULT_MENU_SECTION_FORM });
  }, []);

  const openEdit = useCallback((section: MenuSection) => {
    setEditing(section);
    setForm(sectionFormFromRecord(section));
  }, []);

  const close = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_MENU_SECTION_FORM);
  }, []);

  return { close, editing, form, open: creating || !!editing, openCreate, openEdit, setForm };
}

export function useMenuItemFormState(allSections: MenuSection[]) {
  const [form, setForm] = useState<MenuItemInput>(DEFAULT_MENU_ITEM_FORM);
  const [editing, setEditing] = useState<RbacMenuItem | null>(null);
  const [creating, setCreating] = useState(false);

  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm({ ...DEFAULT_MENU_ITEM_FORM, section_id: allSections[0]?.id ?? '' });
  }, [allSections]);

  const openEdit = useCallback((item: RbacMenuItem) => {
    setEditing(item);
    setForm(menuItemFormFromRecord(item));
  }, []);

  const close = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_MENU_ITEM_FORM);
  }, []);

  return { close, editing, form, open: creating || !!editing, openCreate, openEdit, setForm };
}

export function useMenuApiBindingState() {
  const { t } = useTranslate('admin');
  const [target, setTarget] = useState<RbacMenuItem | null>(null);
  const [loading, setLoading] = useState(false);
  const [selectedApiIds, setSelectedApiIds] = useState<string[]>([]);

  const open = useCallback(async (item: RbacMenuItem) => {
    setTarget(item);
    setLoading(true);
    try {
      const binding = await getMenuApis(item.id);
      setSelectedApiIds(binding.api_permission_ids);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.loadBindingsFailed'));
    } finally {
      setLoading(false);
    }
  }, [t]);

  const close = useCallback(() => {
    setTarget(null);
    setSelectedApiIds([]);
  }, []);

  return { close, loading, open, selectedApiIds, setSelectedApiIds, target };
}

export function useMenuManagementForms(allSections: MenuSection[]) {
  const section = useSectionFormState();
  const item = useMenuItemFormState(allSections);
  const apiBinding = useMenuApiBindingState();

  return { apiBinding, item, section };
}

export function useMenuDeleteState() {
  const [sectionTarget, setSectionTarget] = useState<MenuSection | null>(null);
  const [itemTarget, setItemTarget] = useState<RbacMenuItem | null>(null);

  return { itemTarget, sectionTarget, setItemTarget, setSectionTarget };
}

export async function saveMenuSection(editing: MenuSection | null, form: MenuSectionInput) {
  if (editing) {
    await updateMenuSection(editing.id, form);
    return;
  }

  await createMenuSection(form);
}

export async function saveMenuItem(editing: RbacMenuItem | null, form: MenuItemInput) {
  if (editing) {
    await updateMenuItem(editing.id, form);
    return;
  }

  await createMenuItem(form);
}

function sectionFormFromRecord(section: MenuSection): MenuSectionInput {
  return {
    code: section.code,
    subheader: section.subheader,
    sort_order: section.sort_order,
    enabled: section.enabled,
  };
}

function menuItemFormFromRecord(item: RbacMenuItem): MenuItemInput {
  return {
    section_id: item.section_id,
    parent_id: item.parent_id,
    code: item.code,
    title: item.title,
    path: item.path,
    icon: item.icon,
    caption: item.caption,
    deep_match: item.deep_match,
    sort_order: item.sort_order,
    enabled: item.enabled,
  };
}

export type MenuDeleteState = ReturnType<typeof useMenuDeleteState>;
export type MenuManagementForms = ReturnType<typeof useMenuManagementForms>;

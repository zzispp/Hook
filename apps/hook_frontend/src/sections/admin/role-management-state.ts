'use client';

import type { Role, RoleInput } from 'src/types/rbac';

import { useState, useCallback } from 'react';

import { useTranslate } from 'src/locales/use-locales';
import { createRole, updateRole, getRolePermissions } from 'src/actions/rbac';

import { toast } from 'src/components/snackbar';

export const DEFAULT_ROLE_FORM: RoleInput = {
  code: '',
  name: '',
  description: '',
  enabled: true,
  sort_order: 0,
};

export function useRoleFormState() {
  const [form, setForm] = useState<RoleInput>(DEFAULT_ROLE_FORM);
  const [editing, setEditing] = useState<Role | null>(null);
  const [creating, setCreating] = useState(false);

  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm({ ...DEFAULT_ROLE_FORM });
  }, []);

  const openEdit = useCallback((role: Role) => {
    setEditing(role);
    setForm(roleFormFromRecord(role));
  }, []);

  const close = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_ROLE_FORM);
  }, []);

  return { close, editing, form, open: creating || !!editing, openCreate, openEdit, setForm };
}

export function useRolePermissionState() {
  const { t } = useTranslate('admin');
  const [target, setTarget] = useState<Role | null>(null);
  const [selectedMenus, setSelectedMenus] = useState<string[]>([]);
  const [selectedApis, setSelectedApis] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);

  const open = useCallback(async (role: Role) => {
    setTarget(role);
    setLoading(true);
    try {
      const permissions = await getRolePermissions(role.code);
      setSelectedMenus(permissions.menu_item_ids);
      setSelectedApis(permissions.api_permission_ids);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.loadBindingsFailed'));
    } finally {
      setLoading(false);
    }
  }, [t]);

  const close = useCallback(() => {
    setTarget(null);
    setSelectedMenus([]);
    setSelectedApis([]);
  }, []);

  return {
    close,
    loading,
    open,
    selectedApis,
    selectedMenus,
    setSelectedApis,
    setSelectedMenus,
    target,
  };
}

export function useRoleDeleteState() {
  const [target, setTarget] = useState<Role | null>(null);

  return { setTarget, target };
}

export async function saveRole(editing: Role | null, form: RoleInput) {
  if (editing) {
    await updateRole(editing.code, form);
    return;
  }

  await createRole(form);
}

function roleFormFromRecord(role: Role): RoleInput {
  return {
    code: role.code,
    name: role.name,
    description: role.description,
    enabled: role.enabled,
    sort_order: role.sort_order,
  };
}

export type RoleFormState = ReturnType<typeof useRoleFormState>;
export type RoleDeleteState = ReturnType<typeof useRoleDeleteState>;
export type RolePermissionState = ReturnType<typeof useRolePermissionState>;

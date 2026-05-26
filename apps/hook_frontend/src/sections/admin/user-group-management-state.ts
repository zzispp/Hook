'use client';

import type { UserGroup } from 'src/types/user-group';
import type { useTranslate } from 'src/locales/use-locales';

import { useState, useCallback } from 'react';

import {
  createUserGroup,
  deleteUserGroup,
  updateUserGroup,
} from 'src/actions/user-groups';

import { toast } from 'src/components/snackbar';

import {
  isDefaultUserGroup,
  userGroupUpdatePayload,
  userGroupCreatePayload,
  DEFAULT_USER_GROUP_FORM,
  userGroupFormFromRecord,
} from './user-group-management-utils';

export function useUserGroupDialog(t: ReturnType<typeof useTranslate>['t']) {
  const [form, setForm] = useState({ ...DEFAULT_USER_GROUP_FORM });
  const [editing, setEditing] = useState<UserGroup | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  const close = useCallback(() => {
    setCreating(false);
    setEditing(null);
    setForm({ ...DEFAULT_USER_GROUP_FORM });
  }, []);

  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm({ ...DEFAULT_USER_GROUP_FORM });
  }, []);

  const openEdit = useCallback((group: UserGroup) => {
    setCreating(false);
    setEditing(group);
    setForm(userGroupFormFromRecord(group));
  }, []);

  const submit = useCallback(async () => {
    setSubmitting(true);
    try {
      await saveUserGroup(form, editing);
      toast.success(editing ? t('messages.userGroupUpdated') : t('messages.userGroupCreated'));
      close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [close, editing, form, t]);

  return { close, editing, form, open: creating || !!editing, openCreate, openEdit, setForm, submit, submitting };
}

export function useUserGroupDeleteDialog(t: ReturnType<typeof useTranslate>['t']) {
  const [target, setTarget] = useState<UserGroup | null>(null);

  const confirmDelete = useCallback(async () => {
    if (!target || isDefaultUserGroup(target)) return;
    try {
      await deleteUserGroup(target.code);
      toast.success(t('messages.userGroupDeleted'));
      setTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [target, t]);

  return { confirmDelete, setTarget, target };
}

async function saveUserGroup(form: typeof DEFAULT_USER_GROUP_FORM, editing: UserGroup | null) {
  if (!editing) {
    await createUserGroup(userGroupCreatePayload(form));
    return;
  }
  await updateUserGroup(editing.code, userGroupUpdatePayload(form));
}

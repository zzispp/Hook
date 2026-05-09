'use client';

import type { BillingGroup } from 'src/types/group';
import type { useTranslate } from 'src/locales/use-locales';

import { useState, useCallback } from 'react';

import { createBillingGroup, deleteBillingGroup, updateBillingGroup } from 'src/actions/groups';

import { toast } from 'src/components/snackbar';

import {
  groupPayload,
  formFromGroup,
  DEFAULT_GROUP_FORM,
} from './billing-group-management-utils';

export function useGroupDialog(t: ReturnType<typeof useTranslate>['t']) {
  const [form, setForm] = useState({ ...DEFAULT_GROUP_FORM });
  const [editing, setEditing] = useState<BillingGroup | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  const closeDialog = useCallback(() => {
    setCreating(false);
    setEditing(null);
    setForm({ ...DEFAULT_GROUP_FORM });
  }, []);

  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm({ ...DEFAULT_GROUP_FORM });
  }, []);

  const openEdit = useCallback((group: BillingGroup) => {
    setCreating(false);
    setEditing(group);
    setForm(formFromGroup(group));
  }, []);

  const submit = useCallback(async () => {
    setSubmitting(true);
    try {
      await saveGroup(form, editing);
      toast.success(editing ? t('messages.billingGroupUpdated') : t('messages.billingGroupCreated'));
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [closeDialog, editing, form, t]);

  return { closeDialog, creating, editing, form, open: creating || !!editing, openCreate, openEdit, setForm, submit, submitting };
}

export function useDeleteGroupDialog(t: ReturnType<typeof useTranslate>['t']) {
  const [deleteTarget, setDeleteTarget] = useState<BillingGroup | null>(null);

  const confirmDelete = useCallback(async () => {
    if (!deleteTarget) return;
    try {
      await deleteBillingGroup(deleteTarget.id);
      toast.success(t('messages.billingGroupDeleted'));
      setDeleteTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteTarget, t]);

  return { deleteTarget, setDeleteTarget, confirmDelete };
}

async function saveGroup(form: typeof DEFAULT_GROUP_FORM, editing: BillingGroup | null) {
  const payload = groupPayload(form);
  if (!editing) {
    await createBillingGroup({ ...payload, code: form.code });
    return;
  }
  await updateBillingGroup(editing.id, payload);
}
